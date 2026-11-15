//! Notification-area icon and menu.

use std::time::Duration;

use anyhow::{Context, Result};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::i18n::{self, Text};
use crate::model::Language;
use crate::platform::APP_TITLE;
use crate::worker::{Event, EventSender, TrayAction};

pub struct Tray {
    _icon: TrayIcon,
    _thread: std::thread::JoinHandle<()>,
    items: TrayItems,
}

/// Menu-item handles, kept so the labels can follow a language change.
struct TrayItems {
    show: MenuItem,
    hide: MenuItem,
    sync: MenuItem,
    config: MenuItem,
    quit: MenuItem,
}

impl Tray {
    /// Relabel the menu in place; the icon and menu identity stay the same.
    pub fn set_language(&self, language: Language) {
        self.items.show.set_text(i18n::t(language, Text::TrayShow));
        self.items.hide.set_text(i18n::t(language, Text::TrayHide));
        self.items.sync.set_text(i18n::t(language, Text::TraySync));
        self.items
            .config
            .set_text(i18n::t(language, Text::TrayConfig));
        self.items.quit.set_text(i18n::t(language, Text::TrayQuit));
    }
}

pub fn create(events: EventSender, language: Language) -> Result<Tray> {
    let menu = Menu::new();
    let show = MenuItem::with_id("show", i18n::t(language, Text::TrayShow), true, None);
    let hide = MenuItem::with_id("hide", i18n::t(language, Text::TrayHide), true, None);
    let sync = MenuItem::with_id("sync", i18n::t(language, Text::TraySync), true, None);
    let config = MenuItem::with_id("config", i18n::t(language, Text::TrayConfig), true, None);
    let separator = PredefinedMenuItem::separator();
    let quit = MenuItem::with_id("quit", i18n::t(language, Text::TrayQuit), true, None);

    menu.append_items(&[&show, &hide, &separator, &sync, &config, &separator, &quit])
        .context("build tray menu")?;

    let icon = build_icon(32).context("build tray icon")?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(true)
        .with_tooltip(APP_TITLE)
        .with_icon(icon)
        .build()
        .context("create tray icon")?;

    let thread = std::thread::spawn(move || {
        loop {
            let mut activity = false;
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                activity = true;
                if let Some(action) = action_for(&event.id.0)
                    && events.send(Event::Tray(action)).is_err()
                {
                    return;
                }
            }
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                activity = true;
                if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                    let _ = events.send(Event::Tray(TrayAction::ShowWindow));
                }
            }
            if !activity {
                std::thread::sleep(Duration::from_millis(120));
            }
        }
    });

    Ok(Tray {
        _icon: tray,
        _thread: thread,
        items: TrayItems {
            show,
            hide,
            sync,
            config,
            quit,
        },
    })
}

fn action_for(id: &str) -> Option<TrayAction> {
    match id {
        "show" => Some(TrayAction::ShowWindow),
        "hide" => Some(TrayAction::HideWindow),
        "sync" => Some(TrayAction::SyncNow),
        "config" => Some(TrayAction::OpenConfigDir),
        "quit" => Some(TrayAction::Quit),
        _ => None,
    }
}

/// Draw a small blue shield with a white check mark.
fn build_icon(size: u32) -> Result<Icon> {
    let size_f = size as f32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = size_f / 2.0;
    let check = [
        (0.32 * size_f, 0.52 * size_f),
        (0.46 * size_f, 0.66 * size_f),
        (0.70 * size_f, 0.36 * size_f),
    ];

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let dx = (px - center) / (size_f * 0.38);
            let dy = (py - center) / (size_f * 0.44);
            let inside = dx * dx + dy * dy <= 1.0;
            if !inside {
                continue;
            }

            let t = (py / size_f).clamp(0.0, 1.0);
            let r = lerp(37.0, 29.0, t) as u8;
            let g = lerp(99.0, 64.0, t) as u8;
            let b = lerp(235.0, 175.0, t) as u8;

            let stroke = (size_f * 0.055).max(1.6);
            let on_check = (segment_distance(px, py, check[0], check[1]) < stroke
                || segment_distance(px, py, check[1], check[2]) < stroke)
                && (dy > -0.75);

            let (r, g, b) = if on_check { (255, 255, 255) } else { (r, g, b) };
            let ix = ((y * size + x) * 4) as usize;
            rgba[ix] = r;
            rgba[ix + 1] = g;
            rgba[ix + 2] = b;
            rgba[ix + 3] = 255;
        }
    }

    Icon::from_rgba(rgba, size, size).context("convert tray icon pixels")
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn segment_distance(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (ax, ay) = a;
    let (bx, by) = b;
    let vx = bx - ax;
    let vy = by - ay;
    let wx = px - ax;
    let wy = py - ay;
    let length_sq = vx * vx + vy * vy;
    let t = if length_sq <= f32::EPSILON {
        0.0
    } else {
        ((wx * vx + wy * wy) / length_sq).clamp(0.0, 1.0)
    };
    let cx = ax + t * vx;
    let cy = ay + t * vy;
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}
