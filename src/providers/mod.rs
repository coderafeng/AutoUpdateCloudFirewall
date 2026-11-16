//! Cloud provider clients.
//!
//! Only the security-group ingress APIs needed to keep an allow-list in sync
//! with the local public IP are implemented, so the app has no dependency on a
//! vendor SDK.

pub mod aliyun;
pub mod tencent;

use anyhow::Result;
use reqwest::Client;

use crate::model::ProviderKind;
use crate::planner::{DesiredRule, RemoteRule, RuleUpdate};

/// Access-key credential loaded from the OS credential store.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub access_key_id: String,
    pub secret: String,
}

/// What a security group currently contains.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// Tencent Cloud policy version; `None` for Aliyun.
    pub version: Option<String>,
    pub rules: Vec<RemoteRule>,
}

pub enum ApiClient {
    Aliyun(aliyun::Aliyun),
    Tencent(tencent::Tencent),
}

impl ApiClient {
    pub fn new(provider: ProviderKind, http: Client, credentials: Credentials) -> Self {
        match provider {
            ProviderKind::Aliyun => ApiClient::Aliyun(aliyun::Aliyun::new(http, credentials)),
            ProviderKind::Tencent => ApiClient::Tencent(tencent::Tencent::new(http, credentials)),
        }
    }

    pub fn provider(&self) -> ProviderKind {
        match self {
            ApiClient::Aliyun(_) => ProviderKind::Aliyun,
            ApiClient::Tencent(_) => ProviderKind::Tencent,
        }
    }

    /// Verify that the credential works and ECS access is granted.
    pub async fn check(&self, region: &str) -> Result<()> {
        match self {
            ApiClient::Aliyun(client) => client.check(region).await,
            ApiClient::Tencent(client) => client.check(region).await,
        }
    }

    /// Read the ingress rules of a security group.
    pub async fn list_ingress(&self, region: &str, group_id: &str) -> Result<Snapshot> {
        match self {
            ApiClient::Aliyun(client) => client.list_ingress(region, group_id).await,
            ApiClient::Tencent(client) => client.list_ingress(region, group_id).await,
        }
    }

    pub async fn authorize(
        &self,
        region: &str,
        group_id: &str,
        version: Option<&str>,
        rule: &DesiredRule,
        priority: i32,
    ) -> Result<()> {
        match self {
            ApiClient::Aliyun(client) => client.authorize(region, group_id, rule, priority).await,
            ApiClient::Tencent(client) => client.authorize(region, group_id, version, rule).await,
        }
    }

    pub async fn revoke(
        &self,
        region: &str,
        group_id: &str,
        version: Option<&str>,
        rule: &RemoteRule,
    ) -> Result<()> {
        match self {
            ApiClient::Aliyun(client) => client.revoke(region, group_id, rule).await,
            ApiClient::Tencent(client) => client.revoke(region, group_id, version, rule).await,
        }
    }

    /// Move an existing rule to the source address of `update.desired`.
    ///
    /// Aliyun modifies the rule in place; Tencent Cloud has no per-rule update
    /// call, so the rule is revoked and re-created with the same description.
    pub async fn update_cidr(
        &self,
        region: &str,
        group_id: &str,
        version: Option<&str>,
        update: &RuleUpdate,
    ) -> Result<()> {
        match self {
            ApiClient::Aliyun(client) => client.modify_cidr(region, group_id, update).await,
            ApiClient::Tencent(client) => {
                client.replace_cidr(region, group_id, version, update).await
            }
        }
    }
}

/// Extract `code: message` from a provider error body when possible.
///
/// The raw body is long — Aliyun embeds a whole `AccessDeniedDetail` object —
/// and a plain truncation used to cut away exactly the part that names the
/// missing permission. The fields that matter are extracted first, so the
/// message stays short and still tells the operator what to fix.
pub(crate) fn api_error(provider: &str, status: u16, body: &str) -> anyhow::Error {
    let trimmed = body.trim();
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed)
        && let Some(detail) = summarise(&value)
    {
        return anyhow::anyhow!("{provider} API error (HTTP {status}): {detail}");
    }
    anyhow::anyhow!(
        "{provider} API error (HTTP {status}): {}",
        clip(trimmed, 400)
    )
}

/// `Code: Message`, plus the denied RAM action and the request id when present.
fn summarise(value: &serde_json::Value) -> Option<String> {
    use serde_json::Value;

    // Aliyun keeps the fields at the top level, Tencent Cloud nests them in
    // `Response.Error`.
    let error = match value.get("Response").and_then(|r| r.get("Error")) {
        Some(error) => error,
        None => value,
    };
    let code = error.get("Code").and_then(Value::as_str)?;
    let message = error.get("Message").and_then(Value::as_str).unwrap_or("");
    let mut text = if message.is_empty() {
        code.to_string()
    } else {
        format!("{code}: {}", clip(message, 200))
    };

    if let Some(action) = value
        .pointer("/AccessDeniedDetail/AuthAction")
        .and_then(Value::as_str)
    {
        text.push_str(&format!(" [needs RAM action {action}]"));
    }

    let request = error
        .get("RequestId")
        .or_else(|| value.pointer("/Response/RequestId"))
        .or_else(|| value.get("RequestId"))
        .and_then(Value::as_str);
    if let Some(request) = request {
        text.push_str(&format!(" (request {request})"));
    }

    Some(text)
}

/// Truncate to `max` characters, appending an ellipsis when anything was cut.
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliyun_denied_error_names_the_missing_action() {
        let body = r#"{"RequestId":"01A1E2F9-05AF","HostId":"ecs.aliyuncs.com",
            "Code":"Forbidden.RAM","Message":"User not authorized to operate on the specified resource.",
            "AccessDeniedDetail":{"PolicyType":"ResourceGroupLevelIdentityBasedPolicy",
            "NoPermissionType":"ImplicitDeny","AuthAction":"ecs:RevokeSecurityGroup"}}"#;
        let text = format!("{:#}", api_error("Aliyun", 403, body));
        assert!(text.contains("Forbidden.RAM"));
        assert!(text.contains("needs RAM action ecs:RevokeSecurityGroup"));
        assert!(text.contains("request 01A1E2F9-05AF"));
        assert!(!text.contains("ResourceGroupLevelIdentityBasedPolicy"));
    }

    #[test]
    fn tencent_nested_error_is_unwrapped() {
        let body = r#"{"Response":{"Error":{"Code":"InvalidParameter",
            "Message":"bad rule"},"RequestId":"req-1"}}"#;
        let text = format!("{:#}", api_error("Tencent Cloud", 400, body));
        assert_eq!(
            text,
            "Tencent Cloud API error (HTTP 400): InvalidParameter: bad rule (request req-1)"
        );
    }

    #[test]
    fn non_json_body_is_clipped() {
        let text = format!("{:#}", api_error("Aliyun", 502, &"x".repeat(600)));
        assert!(text.ends_with('…'));
        assert_eq!(text.chars().filter(|c| *c == 'x').count(), 400);
    }
}
