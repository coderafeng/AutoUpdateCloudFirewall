//! Pure rule planning: compare desired rules with the rules that exist in a
//! cloud security group, restricted to rules this app manages.

use crate::model::{PortRange, Protocol, normalize_api_port};

/// A rule this app wants to exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesiredRule {
    pub protocol: Protocol,
    pub start: u16,
    pub end: u16,
    pub cidr: String,
    pub description: String,
}

impl DesiredRule {
    pub fn port_range(&self) -> PortRange {
        PortRange {
            protocol: self.protocol,
            start: self.start,
            end: self.end,
        }
    }

    pub fn port_text(&self) -> String {
        self.port_range().compact()
    }

    fn identity(&self) -> (String, u16, u16, String, String) {
        (
            self.protocol.as_lower().to_string(),
            self.start,
            self.end,
            self.cidr.to_ascii_lowercase(),
            self.description.trim().to_string(),
        )
    }
}

/// A rule currently present in the cloud security group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteRule {
    pub protocol: String,
    pub port: String,
    pub cidr: String,
    pub description: String,
    /// Tencent Cloud policy index (needed to delete a policy).
    pub policy_index: Option<u64>,
}

impl RemoteRule {
    fn identity(&self) -> Option<(String, u16, u16, String, String)> {
        let (start, end) = normalize_api_port(&self.port)?;
        Some((
            self.protocol.to_ascii_lowercase(),
            start,
            end,
            self.cidr.to_ascii_lowercase(),
            self.description.trim().to_string(),
        ))
    }
}

#[derive(Debug, Default, Clone)]
pub struct SyncPlan {
    pub delete: Vec<RemoteRule>,
    pub create: Vec<DesiredRule>,
    pub keep: usize,
}

impl SyncPlan {
    pub fn is_empty(&self) -> bool {
        self.delete.is_empty() && self.create.is_empty()
    }
}

/// Rules are managed by this app only when their description starts with the
/// configured prefix. Everything else in the group is left untouched.
pub fn is_managed(description: &str, prefix: &str) -> bool {
    let description = description.trim_start();
    !prefix.is_empty() && description.starts_with(prefix)
}

/// Build the desired ingress rules for a target: every configured port opened
/// to the current public IP only.
pub fn desired_rules(rules: &[PortRange], ip: &str, prefix: &str) -> Vec<DesiredRule> {
    let cidr = format!("{ip}/32");
    rules
        .iter()
        .map(|rule| DesiredRule {
            protocol: rule.protocol,
            start: rule.start,
            end: rule.end,
            cidr: cidr.clone(),
            description: format!("{} {} {}", prefix, rule.protocol.as_lower(), rule.compact()),
        })
        .collect()
}

/// Compare the managed remote rules against the desired rules.
pub fn plan(remote: &[RemoteRule], desired: &[DesiredRule], prefix: &str) -> SyncPlan {
    let managed: Vec<&RemoteRule> = remote
        .iter()
        .filter(|rule| is_managed(&rule.description, prefix))
        .collect();

    let mut plan = SyncPlan::default();
    let mut matched: Vec<bool> = vec![false; desired.len()];

    for rule in &managed {
        let identity = rule.identity();
        let position = identity.as_ref().and_then(|identity| {
            desired
                .iter()
                .enumerate()
                .position(|(ix, candidate)| !matched[ix] && &candidate.identity() == identity)
        });

        match position {
            Some(ix) => {
                matched[ix] = true;
                plan.keep += 1;
            }
            None => plan.delete.push((*rule).clone()),
        }
    }

    for (ix, rule) in desired.iter().enumerate() {
        if !matched[ix] {
            plan.create.push(rule.clone());
        }
    }

    plan
}

/// All managed rules currently in the group (useful for status reporting).
pub fn managed_rules<'a>(remote: &'a [RemoteRule], prefix: &str) -> Vec<&'a RemoteRule> {
    remote
        .iter()
        .filter(|rule| is_managed(&rule.description, prefix))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(protocol: &str, port: &str, cidr: &str, desc: &str) -> RemoteRule {
        RemoteRule {
            protocol: protocol.to_string(),
            port: port.to_string(),
            cidr: cidr.to_string(),
            description: desc.to_string(),
            policy_index: None,
        }
    }

    fn desired(rules: &[PortRange], ip: &str) -> Vec<DesiredRule> {
        desired_rules(rules, ip, "AutoFirewall")
    }

    #[test]
    fn creates_missing_rules_and_keeps_existing() {
        let ports = [
            PortRange::single(Protocol::Tcp, 80),
            PortRange::single(Protocol::Tcp, 443),
        ];
        let want = desired(&ports, "1.2.3.4");
        let existing = vec![
            remote("tcp", "80/80", "1.2.3.4/32", "AutoFirewall tcp 80"),
            remote("tcp", "22/22", "0.0.0.0/0", "ssh"),
        ];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.keep, 1);
        assert_eq!(plan.create.len(), 1);
        assert_eq!(plan.create[0].start, 443);
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn removes_stale_ip_rules_before_recreating() {
        let ports = [PortRange::single(Protocol::Tcp, 22)];
        let want = desired(&ports, "5.6.7.8");
        let existing = vec![remote("tcp", "22", "1.2.3.4/32", "AutoFirewall tcp 22")];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.delete.len(), 1);
        assert_eq!(plan.create.len(), 1);
        assert_eq!(plan.keep, 0);
    }

    #[test]
    fn ignores_unmanaged_rules() {
        let ports = [PortRange::single(Protocol::Tcp, 80)];
        let want = desired(&ports, "1.2.3.4");
        let existing = vec![
            remote("tcp", "80", "9.9.9.9/32", "legacy"),
            remote("tcp", "80", "1.2.3.4/32", "hand written"),
        ];
        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.create.len(), 1);
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn removes_managed_rules_with_all_traffic() {
        let want: Vec<DesiredRule> = Vec::new();
        let existing = vec![remote("tcp", "ALL", "0.0.0.0/0", "AutoFirewall tcp all")];
        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.delete.len(), 1);
    }

    #[test]
    fn empty_desired_clears_managed_rules_only() {
        let existing = vec![
            remote("tcp", "80", "1.2.3.4/32", "AutoFirewall tcp 80"),
            remote("tcp", "80", "1.2.3.4/32", "other tool"),
        ];
        let plan = plan(&existing, &[], "AutoFirewall");
        assert_eq!(plan.delete.len(), 1);
        assert_eq!(plan.keep, 0);
    }
}
