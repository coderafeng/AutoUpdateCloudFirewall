//! Application artwork.
//!
//! The PNGs and the `.ico` are generated from `assets/logo.svg` and
//! `assets/logo-mono.svg` by `tools/generate-icons.py`; the files are committed
//! so an ordinary build needs no rasteriser.

use std::borrow::Cow;
use std::sync::Arc;

use anyhow::{Context, Result};

/// 32 px brand mark: the Windows notification-area size.
pub const TRAY: &[u8] = include_bytes!("../assets/icon-32.png");

/// 64 px brand mark: window and taskbar icon.
pub const WINDOW: &[u8] = include_bytes!("../assets/icon-64.png");

/// 64 px brand mark exposed to the UI through the asset source.
pub const LOGO: &[u8] = include_bytes!("../assets/icon-64.png");

/// Asset path the UI uses for the sidebar logo.
pub const LOGO_PATH: &str = "logo/icon.png";

/// Notification-area bitmaps, one per shell icon size, so the tray icon stays
/// crisp at 100 %, 125 %, 150 % and 200 % display scaling.
const TRAY_SIZES: [(u32, &[u8]); 6] = [
    (16, include_bytes!("../assets/icon-16.png")),
    (20, include_bytes!("../assets/icon-20.png")),
    (24, include_bytes!("../assets/icon-24.png")),
    (32, TRAY),
    (40, include_bytes!("../assets/icon-40.png")),
    (48, include_bytes!("../assets/icon-48.png")),
];

/// Decode one embedded PNG into straight RGBA pixels.
fn rgba(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32)> {
    let image = image::load_from_memory(bytes)
        .context("decode icon artwork")?
        .into_rgba8();
    let (width, height) = image.dimensions();
    Ok((image.into_raw(), width, height))
}

/// Window and taskbar icon for [`gpui_kit::WindowOptions`].
pub fn window_icon() -> Result<Arc<image::RgbaImage>> {
    let image = image::load_from_memory(WINDOW)
        .context("decode window icon")?
        .into_rgba8();
    Ok(Arc::new(image))
}

/// Notification-area bitmap ready for `tray_icon::Icon::from_rgba`, matched to
/// the current display scaling so the mark is not resampled by the shell.
pub fn tray_pixels() -> Result<(Vec<u8>, u32, u32)> {
    let wanted = tray_size_px();
    let (_, bytes) = TRAY_SIZES
        .iter()
        .find(|(size, _)| *size >= wanted)
        .unwrap_or_else(|| TRAY_SIZES.last().expect("tray sizes are not empty"));
    rgba(bytes)
}

/// Size the shell asks for: the 16 px tray icon scaled by the system DPI.
fn tray_size_px() -> u32 {
    #[cfg(windows)]
    {
        // GetDpiForSystem returns 0 on systems older than Windows 10 1607.
        let dpi = unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForSystem() };
        if dpi != 0 {
            return ((16 * dpi) as f32 / 96.0).round().max(16.0) as u32;
        }
    }
    32
}

/// Serve the artwork to the UI alongside the bundled icon font.
pub struct AppAssets;

impl gpui_kit::AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui_kit::Result<Option<Cow<'static, [u8]>>> {
        if path == LOGO_PATH {
            return Ok(Some(Cow::Borrowed(LOGO)));
        }
        gpui_kit::assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> gpui_kit::Result<Vec<gpui_kit::SharedString>> {
        let mut paths = gpui_kit::assets::Assets.list(path)?;
        paths.push(LOGO_PATH.into());
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
