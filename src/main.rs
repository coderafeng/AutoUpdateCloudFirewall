#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Auto Update Cloud Firewall — keeps cloud security-group ingress rules in
//! sync with the current public IPv4 address.

mod i18n;
mod icon;
mod ip;
mod logging;
mod model;
mod planner;
mod platform;
mod providers;
mod schedule;
mod storage;
mod tray;
mod ui;
mod worker;

use gpui_kit::*;

use crate::platform::APP_TITLE;
use crate::ui::{AppView, Boot};

fn main() {
    // Only one copy may run: a second double click surfaces the running window
    // instead of starting another process (and another hidden tray icon).
    let Some(_instance) = platform::single_instance::acquire() else {
        platform::activate_existing_instance();
        return;
    };

    let mut config = storage::load_config();
    let _ = storage::ensure_dirs();

    // Reflect the real registry state instead of trusting stale config.
    let autostart_enabled = platform::autostart::is_enabled();
    if autostart_enabled {
        // Re-register on every start: keeps the executable path current after an
        // upgrade and syncs the `--minimized` switch with the setting.
        let _ = platform::autostart::set_enabled(true, config.settings.start_minimized);
    }
    config.settings.autostart = autostart_enabled;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("create background runtime");

    let (commands, command_rx) = tokio::sync::mpsc::unbounded_channel();
    let (events, event_rx) = tokio::sync::mpsc::unbounded_channel();

    // Snapshot the log before spawning the worker so live events (which are
    // also written to the file) are not shown twice.
    let initial_logs = logging::Logger::new().read_recent(200);

    worker::spawn(runtime.handle(), config.clone(), command_rx, events.clone());

    let tray = match tray::create(events, config.settings.language) {
        Ok(tray) => Some(tray),
        Err(err) => {
            eprintln!("tray icon unavailable: {err:#}");
            None
        }
    };

    let start_hidden = config.settings.start_minimized && platform::launched_by_autostart();
    let window_icon = icon::window_icon().ok();

    // A manual launch must always end up with a visible window. gpui asks the
    // platform to show it, but some machines (RDP sessions, delayed desktop
    // compositing) leave it created-but-hidden, which looks like "nothing ran".
    if !start_hidden {
        std::thread::spawn(|| {
            for _ in 0..100 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if platform::ensure_main_window_visible() {
                    return;
                }
            }
        });
    }

    let app = gpui_kit::application().with_assets(icon::AppAssets);
    app.run(move |cx| {
        gpui_kit::init(cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1180.), px(790.)), cx)),
            titlebar: Some(TitlebarOptions {
                title: Some(APP_TITLE.into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            icon: window_icon,
            show: !start_hidden,
            ..Default::default()
        };

        gpui_kit::open_window(options, cx, move |window, cx| {
            let boot = Boot {
                config,
                runtime,
                commands,
                events: event_rx,
                tray,
                logs: initial_logs,
            };
            cx.new(|cx| AppView::new(window, cx, boot))
        })
        .expect("failed to open the main window");
    });
}
