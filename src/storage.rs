//! Local persistence: JSON configuration plus secrets in the OS credential store.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::model::{AppConfig, MIN_INTERVAL_SECS};

const APP_DIR: &str = "AutoUpdateCloudFirewall";
const KEYRING_SERVICE: &str = "AutoUpdateCloudFirewall";

fn base_dir() -> PathBuf {
    dirs::config_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
}

pub fn config_dir() -> PathBuf {
    base_dir()
}

pub fn config_path() -> PathBuf {
    base_dir().join("config.json")
}

pub fn log_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(base_dir)
        .join(APP_DIR)
        .join("logs")
}

pub fn log_path() -> PathBuf {
    log_dir().join("app.log")
}

pub fn ensure_dirs() -> Result<()> {
    fs::create_dir_all(config_dir()).with_context(|| "create config directory")?;
    fs::create_dir_all(log_dir()).with_context(|| "create log directory")?;
    Ok(())
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<AppConfig>(&text) {
            Ok(mut config) => {
                // Configs written by older versions allowed 30s; raise them to
                // the current minimum so the settings page edits a valid value.
                config.settings.check_interval_secs =
                    config.settings.check_interval_secs.max(MIN_INTERVAL_SECS);
                config
            }
            Err(err) => {
                let backup = path.with_extension("json.broken");
                let _ = fs::copy(&path, &backup);
                eprintln!(
                    "failed to parse {} ({err}); a copy was kept at {}",
                    path.display(),
                    backup.display()
                );
                AppConfig::default()
            }
        },
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    ensure_dirs()?;
    let path = config_path();
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(config).context("serialize configuration")?;
    fs::write(&tmp, text).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, &path).with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}

fn open_config_folder(path: &Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

pub fn open_config_dir() {
    open_config_folder(&config_dir());
}

pub fn open_log_dir() {
    open_config_folder(&log_dir());
}

pub fn save_secret(account_id: &str, secret: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, account_id)
        .context("open OS credential store entry")?;
    entry
        .set_password(secret)
        .context("store access-key secret in the OS credential store")
}

pub fn load_secret(account_id: &str) -> Result<String> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, account_id)
        .context("open OS credential store entry")?;
    entry
        .get_password()
        .context("read access-key secret from the OS credential store")
}

pub fn delete_secret(account_id: &str) {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, account_id) {
        let _ = entry.delete_credential();
    }
}
