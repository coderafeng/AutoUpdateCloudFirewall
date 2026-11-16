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
mod storage;
mod tray;
mod ui;
mod worker;

use gpui_kit::*;

use crate::platform::APP_TITLE;
use crate::ui::{AppView, Boot};

fn main() {
    let mut config = storage::load_config();
    let _ = storage::ensure_dirs();

    // Reflect the real registry state instead of trusting stale config.
    config.settings.autostart = platform::autostart::is_enabled();

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

    let start_hidden = config.settings.start_minimized;
    let window_icon = icon::window_icon().ok();

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
