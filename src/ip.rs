//! Public IPv4 detection with a configurable list of fallback services.

use std::net::Ipv4Addr;

use anyhow::{Context, Result, bail};
use regex::Regex;
use reqwest::Client;

fn ipv4_regex() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").expect("valid ipv4 regex"))
}

/// True when the address is routable on the public internet.
pub fn is_public_ipv4(ip: &str) -> bool {
    let Ok(addr) = ip.trim().parse::<Ipv4Addr>() else {
        return false;
    };
    let [a, b, c, _d] = addr.octets();
    if addr.is_private()
        || addr.is_loopback()
        || addr.is_link_local()
        || addr.is_broadcast()
        || addr.is_multicast()
        || addr.is_unspecified()
    {
        return false;
    }
    // Carrier-grade NAT 100.64.0.0/10
    if a == 100 && (64..=127).contains(&b) {
        return false;
    }
    // Documentation / benchmark / reserved ranges.
    if a == 0 || a >= 240 {
        return false;
    }
    if a == 192 && b == 0 && (c == 0 || c == 2) {
        return false;
    }
    if a == 198 && (b == 18 || b == 19) {
        return false;
    }
    if a == 198 && b == 51 && c == 100 {
        return false;
    }
    if a == 203 && b == 0 && c == 113 {
        return false;
    }
    true
}

fn extract_ipv4(body: &str) -> Option<String> {
    ipv4_regex()
        .find_iter(body)
        .map(|m| m.as_str())
        .find(|ip| is_public_ipv4(ip))
        .map(str::to_string)
}

/// Try each service in order and return the first valid public IPv4 address.
pub async fn detect(http: &Client, services: &[String]) -> Result<String> {
    if services.is_empty() {
        bail!("no IP lookup services configured");
    }

    let mut errors: Vec<String> = Vec::new();
    for service in services {
        let service = service.trim();
        if service.is_empty() {
            continue;
        }
        match http
            .get(service)
            .timeout(std::time::Duration::from_secs(8))
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                match response.text().await {
                    Ok(body) => {
                        if !status.is_success() {
                            errors.push(format!("{service}: HTTP {status}"));
                            continue;
                        }
                        match extract_ipv4(&body) {
                            Some(ip) => return Ok(ip),
                            None => errors.push(format!("{service}: no public IPv4 in response")),
                        }
                    }
                    Err(err) => errors.push(format!("{service}: {err}")),
                }
            }
            Err(err) => errors.push(format!("{service}: {}", root_cause(&err))),
        }
    }

    Err(anyhow::anyhow!(
        "could not detect a public IPv4 address ({})",
        errors.join("; ")
    ))
}

fn root_cause(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "timed out".to_string()
    } else if err.is_connect() {
        "connection failed".to_string()
    } else {
        err.to_string()
    }
}

/// Fetch a public IP with default timeouts and context for the log.
pub async fn detect_logged(http: &Client, services: &[String]) -> Result<String> {
    detect(http, services)
        .await
        .context("public IP detection failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_addresses() {
        for ip in [
            "127.0.0.1",
            "10.0.0.8",
            "172.16.4.4",
            "192.168.1.10",
            "169.254.1.1",
            "100.64.3.2",
            "0.0.0.0",
            "255.255.255.255",
        ] {
            assert!(!is_public_ipv4(ip), "{ip} should not be public");
        }
    }

    #[test]
    fn accepts_public_addresses() {
        for ip in ["1.1.1.1", "8.8.8.8", "114.114.114.114", "47.96.12.34"] {
            assert!(is_public_ipv4(ip), "{ip} should be public");
        }
    }

    #[test]
    fn extracts_from_plain_and_json() {
        assert_eq!(extract_ipv4("8.8.8.8\n").as_deref(), Some("8.8.8.8"));
        assert_eq!(
            extract_ipv4(r#"{"ip":"1.2.3.4"}"#).as_deref(),
            Some("1.2.3.4")
        );
        assert_eq!(
            extract_ipv4("当前 IP： 47.96.12.34 来自").as_deref(),
            Some("47.96.12.34")
        );
        assert_eq!(extract_ipv4("192.168.1.5"), None);
        // Skip a private-looking value and keep scanning for a public one.
        assert_eq!(
            extract_ipv4("proxy 10.0.0.1, your ip 47.96.12.34").as_deref(),
            Some("47.96.12.34")
        );
    }
}
