//! Pure rule planning: compare desired rules with the rules that exist in a
//! cloud security group.
//!
//! A configured rule can name itself with a *keyword* that appears in the
//! description of an existing rule. Such a rule is adopted: the plan updates
//! its source address in place instead of creating a second rule next to it.

use crate::model::{PortRange, Protocol, RuleSpec, normalize_api_port};

/// A rule this app wants to exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesiredRule {
    pub protocol: Protocol,
    pub start: u16,
    pub end: u16,
    pub cidr: String,
    pub description: String,
    /// Description keyword that picks the existing rule; empty means the rule
    /// is owned through the configured prefix only.
    pub keyword: String,
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

    pub fn has_keyword(&self) -> bool {
        !self.keyword.trim().is_empty()
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

    /// Same protocol and the same inclusive port range.
    fn same_port(&self, remote: &RemoteRule) -> bool {
        remote.protocol.to_ascii_lowercase() == self.protocol.as_lower()
            && normalize_api_port(&remote.port) == Some((self.start, self.end))
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
    /// Aliyun security-group rule id (needed to modify a rule in place).
    pub rule_id: Option<String>,
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

    fn description_contains(&self, keyword: &str) -> bool {
        self.description
            .to_lowercase()
            .contains(&keyword.trim().to_lowercase())
    }
}

/// An adopted rule whose source address has to change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleUpdate {
    pub remote: RemoteRule,
    pub desired: DesiredRule,
}

/// A configured keyword that cannot be resolved to exactly one existing rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousMatch {
    pub desired: DesiredRule,
    /// How many existing rules matched the keyword.
    pub count: usize,
    /// True when the single match is already claimed by another configured rule.
    pub shared: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SyncPlan {
    pub delete: Vec<RemoteRule>,
    pub update: Vec<RuleUpdate>,
    pub create: Vec<DesiredRule>,
    pub keep: usize,
    pub ambiguous: Vec<AmbiguousMatch>,
}

impl SyncPlan {
    pub fn is_empty(&self) -> bool {
        self.delete.is_empty()
            && self.update.is_empty()
            && self.create.is_empty()
            && self.ambiguous.is_empty()
    }
}

/// Rules are managed by this app only when their description starts with the
/// configured prefix. Everything else in the group is left untouched.
pub fn is_managed(description: &str, prefix: &str) -> bool {
    let description = description.trim_start();
    !prefix.is_empty() && description.starts_with(prefix)
}

/// Build the desired ingress rules for a target: every configured rule opened
/// to the current public IP only.
pub fn desired_rules(rules: &[RuleSpec], ip: &str, prefix: &str) -> Vec<DesiredRule> {
    let cidr = format!("{ip}/32");
    rules
        .iter()
        .map(|rule| {
            let keyword = rule.keyword.trim().to_string();
            let mut description = format!(
                "{} {} {}",
                prefix,
                rule.protocol.as_lower(),
                rule.port_text()
            );
            if !keyword.is_empty() {
                description.push(' ');
                description.push_str(&keyword);
            }
            DesiredRule {
                protocol: rule.protocol,
                start: rule.start,
                end: rule.end,
                cidr: cidr.clone(),
                description,
                keyword,
            }
        })
        .collect()
}

/// Compare the remote rules against the desired rules.
///
/// Rules with a keyword adopt exactly one existing rule (matched by protocol,
/// port and description keyword) and are updated in place. Rules without a
/// keyword keep the prefix-based ownership: a managed rule that no longer
/// matches is deleted before the desired rule is created again.
pub fn plan(remote: &[RemoteRule], desired: &[DesiredRule], prefix: &str) -> SyncPlan {
    let mut plan = SyncPlan::default();
    let mut claimed: Vec<bool> = vec![false; remote.len()];
    let mut satisfied: Vec<bool> = vec![false; desired.len()];

    // 1. Keyword rules locate their rule by description.
    for (ix, want) in desired.iter().enumerate() {
        if !want.has_keyword() {
            continue;
        }
        let candidates: Vec<usize> = remote
            .iter()
            .enumerate()
            .filter(|(_, rule)| want.same_port(rule) && rule.description_contains(&want.keyword))
            .map(|(rix, _)| rix)
            .collect();

        match candidates.as_slice() {
            [] => {}
            [rix] => {
                if claimed[*rix] {
                    satisfied[ix] = true;
                    plan.ambiguous.push(AmbiguousMatch {
                        desired: want.clone(),
                        count: 1,
                        shared: true,
                    });
                    continue;
                }
                claimed[*rix] = true;
                satisfied[ix] = true;
                let rule = &remote[*rix];
                if rule.cidr.eq_ignore_ascii_case(&want.cidr) {
                    plan.keep += 1;
                } else {
                    plan.update.push(RuleUpdate {
                        remote: rule.clone(),
                        desired: want.clone(),
                    });
                }
            }
            _ => {
                satisfied[ix] = true;
                plan.ambiguous.push(AmbiguousMatch {
                    desired: want.clone(),
                    count: candidates.len(),
                    shared: false,
                });
            }
        }
    }

    // 2. Rules without a keyword follow the description prefix.
    for (rix, rule) in remote.iter().enumerate() {
        if claimed[rix] || !is_managed(&rule.description, prefix) {
            continue;
        }
        let identity = rule.identity();
        let position = identity.as_ref().and_then(|identity| {
            desired
                .iter()
                .enumerate()
                .position(|(dix, candidate)| !satisfied[dix] && &candidate.identity() == identity)
        });
        match position {
            Some(dix) => {
                satisfied[dix] = true;
                claimed[rix] = true;
                plan.keep += 1;
            }
            None => plan.delete.push(rule.clone()),
        }
    }

    // 3. Whatever is still missing gets created.
    for (dix, rule) in desired.iter().enumerate() {
        if !satisfied[dix] {
            plan.create.push(rule.clone());
        }
    }

    plan
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
            rule_id: None,
        }
    }

    fn spec(protocol: Protocol, port: u16, keyword: &str) -> RuleSpec {
        RuleSpec::new(protocol, port, port, keyword)
    }

    fn desired(rules: &[RuleSpec], ip: &str) -> Vec<DesiredRule> {
        desired_rules(rules, ip, "AutoFirewall")
    }

    #[test]
    fn creates_missing_rules_and_keeps_existing() {
        let rules = [spec(Protocol::Tcp, 80, ""), spec(Protocol::Tcp, 443, "")];
        let want = desired(&rules, "1.2.3.4");
        let existing = vec![
            remote("tcp", "80/80", "1.2.3.4/32", "AutoFirewall tcp 80"),
            remote("tcp", "22/22", "0.0.0.0/0", "ssh"),
        ];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.keep, 1);
        assert_eq!(plan.create.len(), 1);
        assert_eq!(plan.create[0].start, 443);
        assert!(plan.delete.is_empty());
        assert!(plan.update.is_empty());
        assert!(plan.ambiguous.is_empty());
    }

    #[test]
    fn removes_stale_ip_rules_before_recreating() {
        let want = desired(&[spec(Protocol::Tcp, 22, "")], "5.6.7.8");
        let existing = vec![remote("tcp", "22", "1.2.3.4/32", "AutoFirewall tcp 22")];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.delete.len(), 1);
        assert_eq!(plan.create.len(), 1);
        assert_eq!(plan.keep, 0);
    }

    #[test]
    fn ignores_unmanaged_rules() {
        let want = desired(&[spec(Protocol::Tcp, 80, "")], "1.2.3.4");
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

    #[test]
    fn keyword_rule_updates_the_matching_rule_in_place() {
        let want = desired(&[spec(Protocol::Tcp, 22, "home")], "5.6.7.8");
        let existing = vec![
            remote("tcp", "22", "1.2.3.4/32", "home ssh"),
            remote("tcp", "22", "2.2.2.2/32", "web ssh"),
        ];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.keep, 0);
        assert_eq!(plan.update.len(), 1);
        assert_eq!(plan.update[0].remote.description, "home ssh");
        assert_eq!(plan.update[0].desired.cidr, "5.6.7.8/32");
        assert!(plan.create.is_empty());
        assert!(plan.delete.is_empty());
        assert!(plan.ambiguous.is_empty());
    }

    #[test]
    fn keyword_rule_keeps_the_rule_when_the_address_already_matches() {
        let want = desired(&[spec(Protocol::Tcp, 22, "home")], "1.2.3.4");
        let existing = vec![remote("tcp", "22/22", "1.2.3.4/32", "home ssh")];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.keep, 1);
        assert!(plan.update.is_empty());
        assert!(plan.create.is_empty());
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn keyword_rule_is_created_when_nothing_matches() {
        let want = desired(&[spec(Protocol::Tcp, 22, "home")], "5.6.7.8");
        let existing = vec![remote("tcp", "22", "9.9.9.9/32", "web ssh")];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.create.len(), 1);
        assert_eq!(plan.create[0].description, "AutoFirewall tcp 22 home");
        assert!(plan.update.is_empty());
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn ambiguous_keyword_blocks_the_target() {
        let want = desired(&[spec(Protocol::Tcp, 22, "ssh")], "5.6.7.8");
        let existing = vec![
            remote("tcp", "22", "1.2.3.4/32", "home ssh"),
            remote("tcp", "22", "2.2.2.2/32", "web ssh"),
        ];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.ambiguous.len(), 1);
        assert_eq!(plan.ambiguous[0].count, 2);
        assert!(!plan.ambiguous[0].shared);
        assert!(plan.update.is_empty());
        assert!(plan.create.is_empty());
        assert!(plan.delete.is_empty());
    }

    #[test]
    fn keyword_matching_requires_the_same_protocol_and_port() {
        let want = desired(&[spec(Protocol::Tcp, 22, "home")], "5.6.7.8");
        let existing = vec![
            remote("udp", "22", "1.2.3.4/32", "home ssh"),
            remote("tcp", "2222", "1.2.3.4/32", "home ssh"),
            remote("tcp", "ALL", "1.2.3.4/32", "home all"),
        ];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.create.len(), 1);
        assert!(plan.update.is_empty());
    }

    #[test]
    fn keyword_rules_are_never_deleted() {
        let existing = vec![remote("tcp", "22", "1.2.3.4/32", "home ssh")];
        let plan = plan(&existing, &[], "AutoFirewall");
        assert!(plan.delete.is_empty());
        assert!(plan.keep == 0);
    }

    #[test]
    fn two_configured_rules_cannot_adopt_the_same_rule() {
        let want = desired(
            &[
                spec(Protocol::Tcp, 22, "home"),
                spec(Protocol::Tcp, 22, "ssh"),
            ],
            "5.6.7.8",
        );
        let existing = vec![remote("tcp", "22", "1.2.3.4/32", "home ssh")];

        let plan = plan(&existing, &want, "AutoFirewall");
        assert_eq!(plan.ambiguous.len(), 1);
        assert!(plan.ambiguous[0].shared);
        assert_eq!(plan.update.len(), 1);
        assert!(plan.create.is_empty());
    }

    #[test]
    fn already_updated_rules_leave_the_plan_empty() {
        let want = desired(&[spec(Protocol::Tcp, 22, "home")], "5.6.7.8");
        let existing = vec![remote("tcp", "22", "5.6.7.8/32", "home ssh")];
        assert!(plan(&existing, &want, "AutoFirewall").is_empty());
    }
}
