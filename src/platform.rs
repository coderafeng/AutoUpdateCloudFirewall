//! Platform helpers: show/hide the native window and manage auto-start.

pub const APP_TITLE: &str = "Auto Update Cloud Firewall";

/// Project pages reachable from the sidebar footer and the settings page.
pub const REPOSITORY_URL: &str = "https://github.com/coderafeng/AutoUpdateCloudFirewall";
pub const LICENSE_URL: &str =
    "https://github.com/coderafeng/AutoUpdateCloudFirewall/blob/main/LICENSE";
pub const HOMEPAGE_URL: &str = "https://rustsilk.com";

/// Bring the main window back from the tray (or from a minimized state).
pub fn show_main_window() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, IsIconic, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow,
        };

        let title = wide(APP_TITLE);
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            } else {
                ShowWindow(hwnd, SW_SHOW);
            }
            SetForegroundWindow(hwnd);
        }
    }
}

/// Hide the main window to the notification area.
pub fn hide_main_window() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{FindWindowW, SW_HIDE, ShowWindow};

        let title = wide(APP_TITLE);
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Windows auto-start registration under `HKCU\...\Run`.
pub mod autostart {
    use anyhow::Result;

    #[cfg(windows)]
    use anyhow::Context;

    #[cfg(windows)]
    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    #[cfg(windows)]
    const VALUE_NAME: &str = "AutoUpdateCloudFirewall";

    #[cfg(windows)]
    pub fn is_enabled() -> bool {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        current_user
            .open_subkey(RUN_KEY)
            .and_then(|key| key.get_value::<String, _>(VALUE_NAME))
            .is_ok()
    }

    #[cfg(not(windows))]
    pub fn is_enabled() -> bool {
        false
    }

    #[cfg(windows)]
    pub fn set_enabled(enabled: bool) -> Result<()> {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = current_user
            .create_subkey(RUN_KEY)
            .context("open Windows startup registry key")?;

        if enabled {
            let exe = std::env::current_exe().context("resolve current executable")?;
            let command = format!("\"{}\"", exe.display());
            key.set_value(VALUE_NAME, &command)
                .context("register auto-start")?;
        } else {
            match key.delete_value(VALUE_NAME) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err).context("remove auto-start registration"),
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_enabled(_enabled: bool) -> Result<()> {
        anyhow::bail!("auto-start is only implemented on Windows")
    }
}
