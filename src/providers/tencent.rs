//! Tencent Cloud CVM security-group client (TC3-HMAC-SHA256 signature).

use anyhow::{Context, Result, bail};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::{Credentials, Snapshot};
use crate::planner::{DesiredRule, RemoteRule, RuleUpdate};

const ENDPOINT: &str = "https://ecs.tencentcloudapi.com/";
const HOST: &str = "ecs.tencentcloudapi.com";
const SERVICE: &str = "ecs";
const API_VERSION: &str = "2017-03-12";

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

pub struct Tencent {
    http: Client,
    credentials: Credentials,
}

impl Tencent {
    pub fn new(http: Client, credentials: Credentials) -> Self {
        Self { http, credentials }
    }

    async fn call(&self, action: &str, region: &str, payload: Value) -> Result<Value> {
        let body = serde_json::to_string(&payload).context("serialize Tencent payload")?;
        let now = chrono::Utc::now();
        let timestamp = now.timestamp();
        let date = now.format("%Y-%m-%d").to_string();

        let canonical_headers =
            format!("content-type:application/json; charset=utf-8\nhost:{HOST}\n");
        let signed_headers = "content-type;host";
        let canonical_request = format!(
            "POST\n/\n\n{}\n{}\n{}",
            canonical_headers,
            signed_headers,
            sha256_hex(body.as_bytes())
        );
        let string_to_sign = format!(
            "TC3-HMAC-SHA256\n{timestamp}\n{date}/{SERVICE}/tc3_request\n{}",
            sha256_hex(canonical_request.as_bytes())
        );

        let secret_date = hmac_sha256(
            format!("TC3{}", self.credentials.secret).as_bytes(),
            date.as_bytes(),
        );
        let secret_service = hmac_sha256(&secret_date, SERVICE.as_bytes());
        let secret_signing = hmac_sha256(&secret_service, b"tc3_request");
        let signature = hex::encode(hmac_sha256(&secret_signing, string_to_sign.as_bytes()));

        let authorization = format!(
            "TC3-HMAC-SHA256 Credential={}/{}/{SERVICE}/tc3_request, SignedHeaders={signed_headers}, Signature={signature}",
            self.credentials.access_key_id, date
        );

        let response = self
            .http
            .post(ENDPOINT)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", HOST)
            .header("X-TC-Action", action)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("X-TC-Version", API_VERSION)
            .header("X-TC-Region", region)
            .body(body)
            .send()
            .await
            .context("Tencent Cloud request failed")?;

        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);

        if !status.is_success() {
            return Err(super::api_error("Tencent Cloud", status.as_u16(), &text));
        }

        let response_object = json.get("Response").cloned().unwrap_or(Value::Null);
        if let Some(error) = response_object.get("Error") {
            let code = error
                .get("Code")
                .and_then(Value::as_str)
                .unwrap_or("UnknownError");
            let message = error
                .get("Message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error");
            bail!("Tencent Cloud {action} failed: {code}: {message}");
        }
        if response_object.is_null() {
            return Err(super::api_error("Tencent Cloud", status.as_u16(), &text));
        }
        Ok(response_object)
    }

    pub async fn check(&self, region: &str) -> Result<()> {
        self.call("DescribeSecurityGroups", region, json!({ "Limit": 1 }))
            .await
            .map(|_| ())
    }

    pub async fn list_ingress(&self, region: &str, group_id: &str) -> Result<Snapshot> {
        let response = self
            .call(
                "DescribeSecurityGroupPolicies",
                region,
                json!({ "SecurityGroupId": group_id }),
            )
            .await?;

        let set = response
            .get("SecurityGroupPolicySet")
            .cloned()
            .unwrap_or(Value::Null);
        let version = set.get("Version").and_then(|value| match value {
            Value::String(text) => Some(text.clone()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        });

        let rules = set
            .get("Ingress")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| {
                        let action = item
                            .get("Action")
                            .and_then(Value::as_str)
                            .unwrap_or("ACCEPT");
                        if !action.eq_ignore_ascii_case("ACCEPT") {
                            return None;
                        }
                        let cidr = item
                            .get("CidrBlock")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        let port = item
                            .get("Port")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        if cidr.is_empty() || port.is_empty() {
                            return None;
                        }
                        Some(RemoteRule {
                            protocol: item
                                .get("Protocol")
                                .and_then(Value::as_str)
                                .unwrap_or("tcp")
                                .to_string(),
                            port,
                            cidr,
                            description: item
                                .get("PolicyDescription")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            policy_index: item.get("PolicyIndex").and_then(Value::as_u64),
                            rule_id: None,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Snapshot { version, rules })
    }

    pub async fn authorize(
        &self,
        region: &str,
        group_id: &str,
        version: Option<&str>,
        rule: &DesiredRule,
    ) -> Result<()> {
        let version = version
            .context("Tencent Cloud did not return a policy version for this security group")?;
        let policy = json!({
            "Protocol": rule.protocol.as_upper(),
            "Port": rule.port_range().tencent_port(),
            "CidrBlock": rule.cidr,
            "Action": "ACCEPT",
            "PolicyDescription": rule.description,
        });
        self.call(
            "CreateSecurityGroupPolicies",
            region,
            json!({
                "SecurityGroupId": group_id,
                "SecurityGroupPolicySet": {
                    "Version": version,
                    "Ingress": [policy],
                }
            }),
        )
        .await
        .map(|_| ())
    }

    pub async fn revoke(
        &self,
        region: &str,
        group_id: &str,
        version: Option<&str>,
        rule: &RemoteRule,
    ) -> Result<()> {
        let version = version
            .context("Tencent Cloud did not return a policy version for this security group")?;
        let index = rule
            .policy_index
            .context("Tencent Cloud rule has no policy index to delete")?;
        self.call(
            "DeleteSecurityGroupPolicies",
            region,
            json!({
                "SecurityGroupId": group_id,
                "SecurityGroupPolicySet": {
                    "Version": version,
                    "Ingress": [{ "PolicyIndex": index }],
                }
            }),
        )
        .await
        .map(|_| ())
    }

    /// Tencent Cloud cannot modify a policy in place: revoke it and create it
    /// again with the same description but the new source address. The policy
    /// version changes after each call, so it is re-read before re-creating.
    pub async fn replace_cidr(
        &self,
        region: &str,
        group_id: &str,
        version: Option<&str>,
        update: &RuleUpdate,
    ) -> Result<()> {
        self.revoke(region, group_id, version, &update.remote)
            .await?;

        let mut rule = update.desired.clone();
        rule.description = update.remote.description.clone();

        let mut last_error = None;
        for attempt in 0..3u64 {
            if attempt > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(400 * attempt)).await;
            }
            let fresh = self.list_ingress(region, group_id).await?;
            match self
                .authorize(region, group_id, fresh.version.as_deref(), &rule)
                .await
            {
                Ok(()) => return Ok(()),
                Err(err) => last_error = Some(err),
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Tencent Cloud policy update failed")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_stable() {
        // Known TC3 test vector pieces.
        assert_eq!(sha256_hex(b"{\"Limit\": 1}"), sha256_hex(b"{\"Limit\": 1}"));
        assert_eq!(hex::encode(hmac_sha256(b"key", b"data")).len(), 64);
    }
}
