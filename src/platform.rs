//! Platform helpers: show/hide the native window, keep a single running
//! instance alive and manage auto-start.

pub const APP_TITLE: &str = "Auto Update Cloud Firewall";

/// Project pages reachable from the sidebar footer and the settings page.
pub const REPOSITORY_URL: &str = "https://github.com/coderafeng/AutoUpdateCloudFirewall";
pub const LICENSE_URL: &str =
    "https://github.com/coderafeng/AutoUpdateCloudFirewall/blob/main/LICENSE";
pub const HOMEPAGE_URL: &str = "https://rustsilk.com";

/// Switch written into the auto-start registry command. A boot-time launch
/// carries it so it can start in the notification area, while a manual launch
/// (shortcut / double click) never does and therefore always shows the window.
pub const AUTOSTART_SWITCH: &str = "--minimized";

/// True when this process was started by the Windows auto-start entry.
pub fn launched_by_autostart() -> bool {
    std::env::args_os().any(|arg| arg.to_string_lossy().eq_ignore_ascii_case(AUTOSTART_SWITCH))
}

/// Find the main window without changing its state.
#[cfg(windows)]
fn find_main_window() -> windows_sys::Win32::Foundation::HWND {
    use windows_sys::Win32::UI::WindowsAndMessaging::FindWindowW;

    unsafe {
        let title = wide(APP_TITLE);
        FindWindowW(std::ptr::null(), title.as_ptr())
    }
}

/// Bring the main window back from the tray (or from a minimized state).
pub fn show_main_window() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IsIconic, SW_RESTORE, SW_SHOW, SetForegroundWindow, ShowWindow, SwitchToThisWindow,
        };

        let hwnd = find_main_window();
        if !hwnd.is_null() {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            } else {
                ShowWindow(hwnd, SW_SHOW);
            }
            // Windows refuses SetForegroundWindow when another process owns the
            // foreground; SwitchToThisWindow still raises the window.
            if SetForegroundWindow(hwnd) == 0 {
                SwitchToThisWindow(hwnd, 1);
            }
        }
    }
}

/// Make sure a freshly created window is not left invisible. Returns `true`
/// once the window exists, so callers can stop waiting for it.
pub fn ensure_main_window_visible() -> bool {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            IsIconic, IsWindowVisible, SW_RESTORE, SW_SHOW, ShowWindow,
        };

        let hwnd = find_main_window();
        if hwnd.is_null() {
            return false;
        }
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        } else if IsWindowVisible(hwnd) == 0 {
            ShowWindow(hwnd, SW_SHOW);
        }
        true
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Hide the main window to the notification area.
pub fn hide_main_window() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};

        let hwnd = find_main_window();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

/// Wait for the window of an already running instance and raise it. Called by a
/// second launch so a double click never leaves the user staring at nothing.
pub fn activate_existing_instance() {
    for _ in 0..100 {
        if ensure_main_window_visible() {
            show_main_window();
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[cfg(windows)]
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Process-wide guard that makes sure only one copy runs at a time. A second
/// launch would otherwise spawn another tray icon, another worker and another
/// window that stays hidden behind the first one.
pub mod single_instance {
    /// Owns the named mutex for as long as the process runs.
    pub struct Instance {
        #[cfg(windows)]
        handle: windows_sys::Win32::Foundation::HANDLE,
    }

    #[cfg(windows)]
    const MUTEX_NAME: &str = r"Local\AutoUpdateCloudFirewall.SingleInstance";

    /// Returns `None` when another instance already holds the guard.
    pub fn acquire() -> Option<Instance> {
        #[cfg(windows)]
        unsafe {
            use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
            use windows_sys::Win32::System::Threading::CreateMutexW;

            let name = super::wide(MUTEX_NAME);
            let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
            if handle.is_null() {
                // Could not create the mutex at all: run anyway rather than
                // refusing to start.
                return Some(Instance { handle });
            }
            if GetLastError() == ERROR_ALREADY_EXISTS {
                CloseHandle(handle);
                return None;
            }
            Some(Instance { handle })
        }
        #[cfg(not(windows))]
        {
            Some(Instance {})
        }
    }

    impl Drop for Instance {
        fn drop(&mut self) {
            #[cfg(windows)]
            unsafe {
                use windows_sys::Win32::Foundation::CloseHandle;

                if !self.handle.is_null() {
                    CloseHandle(self.handle);
                }
            }
        }
    }
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

    /// Register or remove the auto-start entry. `minimized` decides whether a
    /// boot-time launch starts in the notification area.
    #[cfg(windows)]
    pub fn set_enabled(enabled: bool, minimized: bool) -> Result<()> {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;

        let current_user = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = current_user
            .create_subkey(RUN_KEY)
            .context("open Windows startup registry key")?;

        if enabled {
            let exe = std::env::current_exe().context("resolve current executable")?;
            let mut command = format!("\"{}\"", exe.display());
            if minimized {
                command.push(' ');
                command.push_str(super::AUTOSTART_SWITCH);
            }
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
    pub fn set_enabled(_enabled: bool, _minimized: bool) -> Result<()> {
        anyhow::bail!("auto-start is only implemented on Windows")
    }
}
