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

    let icon = build_icon().context("build tray icon")?;
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

/// The packaged brand mark, rendered from `assets/logo.svg`.
fn build_icon() -> Result<Icon> {
    let (rgba, width, height) = crate::icon::tray_pixels()?;
    Icon::from_rgba(rgba, width, height).context("convert tray icon pixels")
}
