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
use crate::planner::{DesiredRule, RemoteRule};

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
}

/// Extract `code: message` from a provider error body when possible.
pub(crate) fn api_error(provider: &str, status: u16, body: &str) -> anyhow::Error {
    let trimmed = body.trim();
    let detail = if trimmed.chars().count() > 400 {
        let mut detail: String = trimmed.chars().take(400).collect();
        detail.push('…');
        detail
    } else {
        trimmed.to_string()
    };
    anyhow::anyhow!("{provider} API error (HTTP {status}): {detail}")
}
