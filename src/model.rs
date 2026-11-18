//! Configuration model: accounts, security-group targets, port rules and app settings.

use serde::{Deserialize, Serialize};

use crate::i18n::{self, Text};

pub const CONFIG_VERSION: u32 = 1;

/// Smallest accepted sync interval. Below this the cloud API calls and log
/// lines would pile up for no benefit, so both the UI validation and the
/// worker clamp to it.
pub const MIN_INTERVAL_SECS: u64 = 60;

/// Interval used when a config has no `check_interval_secs` yet.
pub const DEFAULT_INTERVAL_SECS: u64 = 300;

fn default_true() -> bool {
    true
}

fn default_interval() -> u64 {
    DEFAULT_INTERVAL_SECS
}

fn default_prefix() -> String {
    "AutoFirewall".to_string()
}

fn default_services() -> Vec<String> {
    vec![
        "https://api.ipify.org".to_string(),
        "https://ifconfig.me/ip".to_string(),
        "https://ipinfo.io/ip".to_string(),
        "https://myip.ipip.net".to_string(),
    ]
}

/// A supported cloud platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    #[default]
    Aliyun,
    Tencent,
}

/// Interface language. Stored in the config with a stable short code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    English,
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
}

impl Language {
    pub fn all() -> [Language; 4] {
        [
            Language::English,
            Language::SimplifiedChinese,
            Language::Japanese,
            Language::Korean,
        ]
    }

    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::SimplifiedChinese => "zh-CN",
            Language::Japanese => "ja",
            Language::Korean => "ko",
        }
    }

    /// Always shown in the language itself so users can find it.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::SimplifiedChinese => "简体中文",
            Language::Japanese => "日本語",
            Language::Korean => "한국어",
        }
    }

    /// Best-effort match of the operating-system UI language.
    pub fn from_system() -> Self {
        #[cfg(windows)]
        {
            let langid = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
            match langid & 0x3ff {
                0x04 => Language::SimplifiedChinese,
                0x11 => Language::Japanese,
                0x12 => Language::Korean,
                _ => Language::English,
            }
        }
        #[cfg(not(windows))]
        {
            let value = std::env::var("LANG")
                .unwrap_or_default()
                .to_ascii_lowercase();
            if value.starts_with("zh") {
                Language::SimplifiedChinese
            } else if value.starts_with("ja") {
                Language::Japanese
            } else if value.starts_with("ko") {
                Language::Korean
            } else {
                Language::English
            }
        }
    }
}

impl ProviderKind {
    pub fn label(self) -> &'static str {
        match self {
            ProviderKind::Aliyun => "Aliyun ECS",
            ProviderKind::Tencent => "Tencent Cloud CVM",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            ProviderKind::Aliyun => "Aliyun",
            ProviderKind::Tencent => "Tencent",
        }
    }

    pub fn default_region(self) -> &'static str {
        match self {
            ProviderKind::Aliyun => "cn-hangzhou",
            ProviderKind::Tencent => "ap-guangzhou",
        }
    }
}

/// Transport protocol of a firewall rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    pub fn as_lower(self) -> &'static str {
        match self {
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
        }
    }

    pub fn as_upper(self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        }
    }
}

/// A single port or inclusive port range for one protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortRange {
    pub protocol: Protocol,
    pub start: u16,
    pub end: u16,
}

impl PortRange {
    /// `80` for a single port, `8000-8010` for a range.
    pub fn compact(self) -> String {
        if self.start == self.end {
            self.start.to_string()
        } else {
            format!("{}-{}", self.start, self.end)
        }
    }

    /// Aliyun expects `80/80` or `8000/8010`.
    pub fn aliyun_port(self) -> String {
        format!("{}/{}", self.start, self.end)
    }

    /// Tencent expects `80` or `8000-8010`.
    pub fn tencent_port(self) -> String {
        self.compact()
    }
}

/// One configured ingress rule: a port range plus an optional keyword that
/// picks out the existing security-group rule by its description.
///
/// An empty keyword keeps the legacy behaviour: the rule is owned through the
/// configured description prefix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSpec {
    pub protocol: Protocol,
    pub start: u16,
    pub end: u16,
    #[serde(default)]
    pub keyword: String,
}

impl RuleSpec {
    pub fn new(protocol: Protocol, start: u16, end: u16, keyword: impl Into<String>) -> Self {
        Self {
            protocol,
            start,
            end,
            keyword: keyword.into(),
        }
    }

    pub fn port_range(&self) -> PortRange {
        PortRange {
            protocol: self.protocol,
            start: self.start,
            end: self.end,
        }
    }

    /// `80` for a single port, `8000-8010` for a range.
    pub fn port_text(&self) -> String {
        self.port_range().compact()
    }

    pub fn has_keyword(&self) -> bool {
        !self.keyword.trim().is_empty()
    }

    /// Same protocol, same port and the same keyword (case-insensitive).
    pub fn same_spec(&self, other: &Self) -> bool {
        self.protocol == other.protocol
            && self.start == other.start
            && self.end == other.end
            && self.keyword.trim().to_lowercase() == other.keyword.trim().to_lowercase()
    }
}

/// Normalize a cloud API port value (`80`, `80/80`, `8000-8010`, `ALL`, `-1/-1`)
/// into an inclusive range. Returns `None` for "all traffic" style values.
pub fn normalize_api_port(port: &str) -> Option<(u16, u16)> {
    let port = port.trim();
    if port.is_empty() {
        return None;
    }
    let lower = port.to_ascii_lowercase();
    if lower == "all" || lower == "-1/-1" || lower == "-1" || lower == "*" {
        return None;
    }
    let body = if let Some(rest) = lower.strip_prefix("tcp:") {
        rest
    } else if let Some(rest) = lower.strip_prefix("udp:") {
        rest
    } else {
        lower.as_str()
    };
    let (start, end) = if let Some((a, b)) = body.split_once('-') {
        (a.trim(), b.trim())
    } else if let Some((a, b)) = body.split_once('/') {
        (a.trim(), b.trim())
    } else {
        (body, body)
    };
    let start: u16 = start.parse().ok()?;
    let end: u16 = end.parse().ok()?;
    if start == 0 || end == 0 || start > end {
        return None;
    }
    Some((start, end))
}

/// A problem found while parsing the port text. The message is rendered in the
/// current interface language by [`PortError::message`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    Empty,
    Invalid(String),
    OutOfRange(String),
    Reversed(String),
}

impl PortError {
    pub fn message(&self, language: Language) -> String {
        match self {
            PortError::Empty => i18n::t(language, Text::ErrPortListEmpty).to_string(),
            PortError::Invalid(port) => i18n::tr(language, Text::ErrPortInvalid, &[("port", port)]),
            PortError::OutOfRange(port) => {
                i18n::tr(language, Text::ErrPortRange, &[("port", port)])
            }
            PortError::Reversed(port) => {
                i18n::tr(language, Text::ErrPortReversed, &[("port", port)])
            }
        }
    }
}

/// Parse one port field of the rule editor: a single port (`22`) or an
/// inclusive range (`8000-8010` / `8000/8010`).
pub fn parse_port_field(input: &str) -> Result<(u16, u16), PortError> {
    let item = input.trim();
    if item.is_empty() {
        return Err(PortError::Empty);
    }
    if item
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, ',' | ';' | ':'))
    {
        return Err(PortError::Invalid(item.to_string()));
    }
    let (start, end) = if let Some((a, b)) = item.split_once('-') {
        (a.trim(), b.trim())
    } else if let Some((a, b)) = item.split_once('/') {
        (a.trim(), b.trim())
    } else {
        (item, item)
    };
    let start: u16 = start
        .parse()
        .map_err(|_| PortError::Invalid(item.to_string()))?;
    let end: u16 = end
        .parse()
        .map_err(|_| PortError::Invalid(item.to_string()))?;
    if start == 0 || end == 0 {
        return Err(PortError::OutOfRange(item.to_string()));
    }
    if start > end {
        return Err(PortError::Reversed(item.to_string()));
    }
    Ok((start, end))
}

pub fn describe_port_rules(rules: &[RuleSpec], empty: &str) -> String {
    if rules.is_empty() {
        return empty.to_string();
    }
    let mut groups: Vec<(Protocol, Vec<String>)> = Vec::new();
    for rule in rules {
        let mut text = rule.port_text();
        if rule.has_keyword() {
            text.push_str(&format!(" ({})", rule.keyword.trim()));
        }
        match groups.iter_mut().find(|(p, _)| *p == rule.protocol) {
            Some((_, list)) => list.push(text),
            None => groups.push((rule.protocol, vec![text])),
        }
    }
    groups
        .into_iter()
        .map(|(protocol, ports)| format!("{} {}", protocol.label(), ports.join(", ")))
        .collect::<Vec<_>>()
        .join("  ·  ")
}

/// A cloud account with an access-key credential.
///
/// The access-key **secret** never lives in the config file; it is stored in the
/// operating system credential store, keyed by [`Account::id`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub provider: ProviderKind,
    #[serde(default)]
    pub access_key_id: String,
    #[serde(default)]
    pub region: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub remark: String,
}

impl Account {
    pub fn new(name: impl Into<String>, provider: ProviderKind) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            provider,
            access_key_id: String::new(),
            region: provider.default_region().to_string(),
            enabled: true,
            remark: String::new(),
        }
    }

    pub fn region_or_default(&self) -> &str {
        if self.region.trim().is_empty() {
            self.provider.default_region()
        } else {
            self.region.trim()
        }
    }

    /// Show only the head and tail of the access-key id.
    pub fn masked_key(&self) -> String {
        let key = self.access_key_id.trim();
        let chars: Vec<char> = key.chars().collect();
        if chars.len() <= 8 {
            if chars.is_empty() {
                "not set".to_string()
            } else {
                "••••••••".to_string()
            }
        } else {
            let head: String = chars.iter().take(4).collect();
            let tail: String = chars.iter().skip(chars.len() - 3).collect();
            format!("{head}••••{tail}")
        }
    }
}

/// A security group whose ingress rules are kept in sync with the current IP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub account_id: String,
    /// Optional region override; falls back to the account region.
    #[serde(default)]
    pub region: String,
    pub security_group_id: String,
    #[serde(default)]
    pub rules: Vec<RuleSpec>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Sync interval for this target only; `0` means "use the global value".
    #[serde(default)]
    pub interval_secs: u64,
    /// Cron expression for this target only; empty means "use the global one".
    #[serde(default)]
    pub schedule_cron: String,
}

impl Target {
    pub fn new(account_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            account_id: account_id.into(),
            region: String::new(),
            security_group_id: String::new(),
            rules: Vec::new(),
            enabled: true,
            interval_secs: 0,
            schedule_cron: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "default_language")]
    pub language: Language,
    #[serde(default = "default_interval")]
    pub check_interval_secs: u64,
    /// Global cron expression; when set it takes precedence over the interval.
    #[serde(default)]
    pub schedule_cron: String,
    #[serde(default = "default_services")]
    pub ip_services: Vec<String>,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default = "default_prefix")]
    pub rule_prefix: String,
    #[serde(default = "default_priority")]
    pub rule_priority: i32,
}

fn default_priority() -> i32 {
    1
}

fn default_language() -> Language {
    Language::from_system()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: Language::from_system(),
            check_interval_secs: default_interval(),
            schedule_cron: String::new(),
            ip_services: default_services(),
            dry_run: false,
            close_to_tray: true,
            start_minimized: false,
            autostart: false,
            rule_prefix: default_prefix(),
            rule_priority: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(default)]
    pub targets: Vec<Target>,
    #[serde(default)]
    pub settings: Settings,
}

fn default_version() -> u32 {
    CONFIG_VERSION
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION,
            accounts: Vec::new(),
            targets: Vec::new(),
            settings: Settings::default(),
        }
    }
}

impl AppConfig {
    pub fn account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|account| account.id == id)
    }

    pub fn targets_for(&self, account_id: &str) -> impl Iterator<Item = &Target> {
        self.targets
            .iter()
            .filter(move |target| target.account_id == account_id)
    }

    pub fn enabled_target_count(&self) -> usize {
        self.targets
            .iter()
            .filter(|target| {
                target.enabled
                    && self
                        .account(&target.account_id)
                        .map(|account| account.enabled)
                        .unwrap_or(false)
            })
            .count()
    }

    /// Settings that changed since the config was loaded by the worker.
    pub fn sync_interval_secs(&self) -> u64 {
        self.settings.check_interval_secs.max(MIN_INTERVAL_SECS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_single_port_field() {
        assert_eq!(parse_port_field("22"), Ok((22, 22)));
        assert_eq!(parse_port_field(" 8000-8010 "), Ok((8000, 8010)));
        assert_eq!(parse_port_field("8000/8010"), Ok((8000, 8010)));
    }

    #[test]
    fn rejects_bad_port_fields() {
        assert!(parse_port_field("").is_err());
        assert!(parse_port_field("0").is_err());
        assert!(parse_port_field("90-80").is_err());
        assert!(parse_port_field("abc").is_err());
        assert!(parse_port_field("22,80").is_err());
        assert!(parse_port_field("tcp:22").is_err());
    }

    #[test]
    fn rule_specs_round_trip_and_keep_old_configs_valid() {
        let spec = RuleSpec::new(Protocol::Tcp, 22, 22, "home");
        let json = serde_json::to_string(&spec).unwrap();
        assert_eq!(serde_json::from_str::<RuleSpec>(&json).unwrap(), spec);

        let old: RuleSpec =
            serde_json::from_str(r#"{"protocol":"tcp","start":80,"end":80}"#).unwrap();
        assert_eq!(old.keyword, "");
        assert!(!old.has_keyword());
    }

    #[test]
    fn duplicate_specs_compare_keywords_case_insensitively() {
        let a = RuleSpec::new(Protocol::Tcp, 22, 22, "Home");
        let b = RuleSpec::new(Protocol::Tcp, 22, 22, "home");
        let c = RuleSpec::new(Protocol::Tcp, 22, 22, "web");
        assert!(a.same_spec(&b));
        assert!(!a.same_spec(&c));
    }

    #[test]
    fn describes_rules_with_keywords() {
        let rules = [
            RuleSpec::new(Protocol::Tcp, 22, 22, "home"),
            RuleSpec::new(Protocol::Tcp, 443, 443, ""),
            RuleSpec::new(Protocol::Udp, 53, 53, "dns"),
        ];
        assert_eq!(
            describe_port_rules(&rules, "none"),
            "TCP 22 (home), 443  ·  UDP 53 (dns)"
        );
        assert_eq!(describe_port_rules(&[], "none"), "none");
    }

    #[test]
    fn normalizes_api_ports() {
        assert_eq!(normalize_api_port("80"), Some((80, 80)));
        assert_eq!(normalize_api_port("80/80"), Some((80, 80)));
        assert_eq!(normalize_api_port("8000-8010"), Some((8000, 8010)));
        assert_eq!(normalize_api_port("ALL"), None);
        assert_eq!(normalize_api_port("-1/-1"), None);
    }

    #[test]
    fn masks_access_keys() {
        let mut account = Account::new("demo", ProviderKind::Aliyun);
        assert_eq!(account.masked_key(), "not set");
        account.access_key_id = "LTAI5tABCDEFGH1234".to_string();
        assert_eq!(account.masked_key(), "LTAI••••234");
    }

    #[test]
    fn example_config_parses() {
        let text = include_str!("../config.json.example");
        let config: AppConfig = serde_json::from_str(text).expect("example config is valid");
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.accounts.len(), 2);
        assert_eq!(config.targets.len(), 1);
        assert_eq!(config.targets[0].rules.len(), 3);
        assert_eq!(config.settings.language, Language::English);
    }

    #[test]
    fn language_round_trips_through_json() {
        for language in Language::all() {
            let json = serde_json::to_string(&language).unwrap();
            let parsed: Language = serde_json::from_str(&json).unwrap();
            assert_eq!(language, parsed);
        }
    }

    #[test]
    fn missing_language_falls_back_to_system() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.language, Language::from_system());
    }

    #[test]
    fn interval_defaults_to_300_and_clamps_to_60() {
        let mut config = AppConfig::default();
        assert_eq!(config.settings.check_interval_secs, DEFAULT_INTERVAL_SECS);
        assert_eq!(config.sync_interval_secs(), 300);

        config.settings.check_interval_secs = 30;
        assert_eq!(config.sync_interval_secs(), MIN_INTERVAL_SECS);
    }
}
