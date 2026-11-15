//! Aliyun ECS security-group client (RPC style, HMAC-SHA1 signature).

use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::Client;
use serde_json::Value;
use sha1::Sha1;

use super::{Credentials, Snapshot};
use crate::planner::{DesiredRule, RemoteRule};

const ENDPOINT: &str = "https://ecs.aliyuncs.com/";
const API_VERSION: &str = "2014-05-26";

/// RFC3986 unreserved characters only, matching Aliyun's signing rules.
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

fn encode(value: &str) -> String {
    utf8_percent_encode(value, ENCODE_SET).to_string()
}

pub struct Aliyun {
    http: Client,
    credentials: Credentials,
}

impl Aliyun {
    pub fn new(http: Client, credentials: Credentials) -> Self {
        Self { http, credentials }
    }

    async fn call(&self, action: &str, region: &str, extra: Vec<(&str, String)>) -> Result<Value> {
        let mut params: Vec<(String, String)> = vec![
            ("AccessKeyId".into(), self.credentials.access_key_id.clone()),
            ("Action".into(), action.into()),
            ("Format".into(), "JSON".into()),
            ("RegionId".into(), region.into()),
            ("SignatureMethod".into(), "HMAC-SHA1".into()),
            ("SignatureNonce".into(), uuid::Uuid::new_v4().to_string()),
            ("SignatureVersion".into(), "1.0".into()),
            ("Timestamp".into(), timestamp()),
            ("Version".into(), API_VERSION.into()),
        ];
        for (key, value) in extra {
            params.push((key.to_string(), value));
        }
        params.sort_by(|a, b| a.0.cmp(&b.0));

        let canonical = params
            .iter()
            .map(|(key, value)| format!("{}={}", encode(key), encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        let string_to_sign = format!("GET&%2F&{}", encode(&canonical));

        let mut mac =
            Hmac::<Sha1>::new_from_slice(format!("{}&", self.credentials.secret).as_bytes())
                .context("build HMAC key")?;
        mac.update(string_to_sign.as_bytes());
        let signature = BASE64.encode(mac.finalize().into_bytes());

        let url = format!("{ENDPOINT}?{canonical}&Signature={}", encode(&signature));
        let response = self
            .http
            .get(url)
            .header("x-acs-version", API_VERSION)
            .send()
            .await
            .context("Aliyun request failed")?;
        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        let json: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
        if !status.is_success() {
            return Err(super::api_error("Aliyun", status.as_u16(), &body));
        }
        if let Some(code) = json.get("Code").and_then(Value::as_str) {
            let message = json
                .get("Message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            bail!("Aliyun {action} failed: {code}: {message}");
        }
        Ok(json)
    }

    pub async fn check(&self, region: &str) -> Result<()> {
        self.call(
            "DescribeSecurityGroups",
            region,
            vec![
                ("PageSize", "1".to_string()),
                ("PageNumber", "1".to_string()),
            ],
        )
        .await
        .map(|_| ())
    }

    pub async fn list_ingress(&self, region: &str, group_id: &str) -> Result<Snapshot> {
        let json = self
            .call(
                "DescribeSecurityGroupAttribute",
                region,
                vec![
                    ("SecurityGroupId", group_id.to_string()),
                    ("Direction", "ingress".to_string()),
                ],
            )
            .await?;

        let permissions = json
            .pointer("/Permissions/Permission")
            .cloned()
            .unwrap_or(Value::Null);
        let list = match permissions {
            Value::Array(items) => items,
            Value::Object(_) => vec![permissions],
            _ => Vec::new(),
        };

        let rules = list
            .iter()
            .filter_map(|item| {
                let protocol = item
                    .get("IpProtocol")
                    .and_then(Value::as_str)
                    .unwrap_or("tcp")
                    .to_string();
                let port = item
                    .get("PortRange")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let cidr = item
                    .get("SourceCidrIp")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                if cidr.is_empty() || port.is_empty() {
                    return None;
                }
                Some(RemoteRule {
                    protocol,
                    port,
                    cidr,
                    description: item
                        .get("Description")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    policy_index: None,
                })
            })
            .collect();

        Ok(Snapshot {
            version: None,
            rules,
        })
    }

    pub async fn authorize(
        &self,
        region: &str,
        group_id: &str,
        rule: &DesiredRule,
        priority: i32,
    ) -> Result<()> {
        self.call(
            "AuthorizeSecurityGroup",
            region,
            vec![
                ("SecurityGroupId", group_id.to_string()),
                ("IpProtocol", rule.protocol.as_lower().to_string()),
                ("PortRange", rule.port_range().aliyun_port()),
                ("SourceCidrIp", rule.cidr.clone()),
                ("Description", rule.description.clone()),
                ("Priority", priority.clamp(1, 100).to_string()),
            ],
        )
        .await
        .map(|_| ())
    }

    pub async fn revoke(&self, region: &str, group_id: &str, rule: &RemoteRule) -> Result<()> {
        self.call(
            "RevokeSecurityGroup",
            region,
            vec![
                ("SecurityGroupId", group_id.to_string()),
                ("IpProtocol", rule.protocol.to_ascii_lowercase()),
                ("PortRange", rule.port.clone()),
                ("SourceCidrIp", rule.cidr.clone()),
            ],
        )
        .await
        .map(|_| ())
    }
}

fn timestamp() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3986_encoding() {
        assert_eq!(encode("a b"), "a%20b");
        assert_eq!(encode("a+b"), "a%2Bb");
        assert_eq!(encode("a*b~-._"), "a%2Ab~-._");
        assert_eq!(encode("1.2.3.4/32"), "1.2.3.4%2F32");
    }
}
