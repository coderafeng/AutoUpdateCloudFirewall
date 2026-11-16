//! Application views: sidebar shell, overview, accounts, targets, logs, settings.

mod dialogs;
mod rules_editor;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::component::{
    ActiveTheme, Disableable as _, Icon, IconName, Sizable as _, StyledExt as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    scroll::ScrollableElement,
    sidebar::{
        Sidebar, SidebarCollapsible, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu,
        SidebarMenuItem,
    },
    switch::Switch,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use tokio::sync::mpsc::UnboundedSender;

use crate::i18n::{self, Text};
use crate::logging::{Level, LogEntry};
use crate::model::{AppConfig, Language, MIN_INTERVAL_SECS, ProviderKind, describe_port_rules};
use crate::platform::{self, autostart};
use crate::storage;
use crate::tray::Tray;
use crate::worker::{Command, Event, EventReceiver, TargetPhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Overview,
    Accounts,
    Targets,
    Logs,
    Settings,
}

impl Page {
    fn title(self, language: Language) -> &'static str {
        let text = match self {
            Page::Overview => Text::PageOverview,
            Page::Accounts => Text::PageAccounts,
            Page::Targets => Text::PageTargets,
            Page::Logs => Text::PageLogs,
            Page::Settings => Text::PageSettings,
        };
        i18n::t(language, text)
    }

    fn subtitle(self, language: Language) -> &'static str {
        let text = match self {
            Page::Overview => Text::PageOverviewSub,
            Page::Accounts => Text::PageAccountsSub,
            Page::Targets => Text::PageTargetsSub,
            Page::Logs => Text::PageLogsSub,
            Page::Settings => Text::PageSettingsSub,
        };
        i18n::t(language, text)
    }

    fn icon(self) -> IconName {
        match self {
            Page::Overview => IconName::LayoutDashboard,
            Page::Accounts => IconName::User,
            Page::Targets => IconName::Network,
            Page::Logs => IconName::FileText,
            Page::Settings => IconName::Settings,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
    Idle,
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeLevel {
    Info,
    Success,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notice {
    pub level: NoticeLevel,
    pub text: String,
}

pub struct Boot {
    pub config: AppConfig,
    pub runtime: tokio::runtime::Runtime,
    pub commands: UnboundedSender<Command>,
    pub events: EventReceiver,
    pub tray: Option<Tray>,
    /// Log entries from previous runs, snapshotted before the worker starts.
    pub logs: Vec<LogEntry>,
}

pub struct AppView {
    pub config: AppConfig,
    language: Language,
    page: Page,
    logs: Vec<LogEntry>,
    public_ip: Option<String>,
    last_ip_at: Option<String>,
    last_sync_at: Option<String>,
    sync_state: SyncState,
    target_status: HashMap<String, (TargetPhase, String)>,
    notice: Option<Notice>,

    pub commands: UnboundedSender<Command>,
    /// Kept alive for the lifetime of the window; dropping it stops the worker.
    _runtime: Option<tokio::runtime::Runtime>,
    /// Kept alive for the lifetime of the window; dropping it removes the tray icon.
    tray: Option<Tray>,
    allow_close: Arc<AtomicBool>,
    close_to_tray: Arc<AtomicBool>,

    account_name: Entity<InputState>,
    account_key: Entity<InputState>,
    account_secret: Entity<InputState>,
    account_region: Entity<InputState>,
    pub account_provider: ProviderKind,
    pub editing_account: Option<String>,

    target_name: Entity<InputState>,
    target_group: Entity<InputState>,
    target_region: Entity<InputState>,
    rules_editor: Entity<rules_editor::RulesEditor>,
    pub target_account: Option<String>,
    pub editing_target: Option<String>,

    settings_interval: Entity<InputState>,
    settings_services: Entity<InputState>,
    settings_prefix: Entity<InputState>,
}

impl AppView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>, mut boot: Boot) -> Self {
        let config = std::mem::take(&mut boot.config);
        let logs = std::mem::take(&mut boot.logs);
        let language = config.settings.language;

        let account_name = cx.new(|cx| {
            InputState::new(window, cx).placeholder(i18n::t(language, Text::PlaceholderAccountName))
        });
        let account_key = cx.new(|cx| InputState::new(window, cx).placeholder("LTAI5t…"));
        let account_secret = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(i18n::t(language, Text::AccessKeySecret))
                .masked(true)
        });
        let account_region = cx.new(|cx| InputState::new(window, cx).placeholder("cn-hangzhou"));

        let target_name = cx.new(|cx| {
            InputState::new(window, cx).placeholder(i18n::t(language, Text::PlaceholderTargetName))
        });
        let target_group = cx.new(|cx| InputState::new(window, cx).placeholder("sg-bp1…"));
        let target_region = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(i18n::t(language, Text::PlaceholderRegionInherit))
        });
        let rules_editor =
            cx.new(|cx| rules_editor::RulesEditor::with_defaults(language, window, cx));

        let settings_interval = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(config.settings.check_interval_secs.to_string())
        });
        let settings_services = cx.new(|cx| {
            InputState::new(window, cx).default_value(config.settings.ip_services.join(", "))
        });
        let settings_prefix = cx.new(|cx| {
            InputState::new(window, cx).default_value(config.settings.rule_prefix.clone())
        });

        let allow_close = Arc::new(AtomicBool::new(false));
        let close_to_tray = Arc::new(AtomicBool::new(config.settings.close_to_tray));
        {
            let allow = allow_close.clone();
            let close_flag = close_to_tray.clone();
            window.on_window_should_close(cx, move |_window, _cx| {
                if allow.load(Ordering::SeqCst) || !close_flag.load(Ordering::SeqCst) {
                    return true;
                }
                platform::hide_main_window();
                false
            });
        }

        let mut events = boot.events;
        cx.spawn(async move |this, cx| {
            while let Some(event) = events.recv().await {
                if this
                    .update(cx, |this, cx| {
                        this.handle_event(event, cx);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        let _ = boot.commands.send(Command::DetectIp);

        let mut view = Self {
            config,
            language,
            page: Page::Overview,
            logs,
            public_ip: None,
            last_ip_at: None,
            last_sync_at: None,
            sync_state: SyncState::Idle,
            target_status: HashMap::new(),
            notice: None,
            commands: boot.commands,
            _runtime: Some(boot.runtime),
            tray: boot.tray,
            allow_close,
            close_to_tray,
            account_name,
            account_key,
            account_secret,
            account_region,
            account_provider: ProviderKind::Aliyun,
            editing_account: None,
            target_name,
            target_group,
            target_region,
            rules_editor,
            target_account: None,
            editing_target: None,
            settings_interval,
            settings_services,
            settings_prefix,
        };

        view.set_notice(
            NoticeLevel::Info,
            view.tr(
                Text::ConfigNotice,
                &[("path", &storage::config_path().display().to_string())],
            ),
        );
        view
    }

    fn t(&self, text: Text) -> &'static str {
        i18n::t(self.language, text)
    }

    fn tr(&self, text: Text, args: &[(&str, &str)]) -> String {
        i18n::tr(self.language, text, args)
    }

    /// Switch the interface language immediately: UI labels, tray menu, and
    /// worker log messages all follow.
    fn set_language(&mut self, language: Language, window: &mut Window, cx: &mut Context<Self>) {
        if self.language == language {
            return;
        }
        self.language = language;
        self.config.settings.language = language;

        let placeholders = [
            (&self.account_name, Text::PlaceholderAccountName),
            (&self.account_secret, Text::AccessKeySecret),
            (&self.target_name, Text::PlaceholderTargetName),
            (&self.target_region, Text::PlaceholderRegionInherit),
        ];
        for (input, text) in placeholders {
            let value = i18n::t(language, text);
            input.update(cx, |state, cx| {
                state.set_placeholder(value, window, cx);
            });
        }

        if let Some(tray) = self.tray.as_ref() {
            tray.set_language(language);
        }

        self.persist();
        self.set_notice(NoticeLevel::Info, i18n::t(language, Text::LanguageChanged));
        cx.notify();
    }

    pub fn sync_now(&mut self, manual: bool) {
        self.sync_state = SyncState::Running;
        self.notice = None;
        if self.commands.send(Command::SyncNow { manual }).is_err() {
            self.sync_state = SyncState::Idle;
            let message = self.t(Text::WorkerNotRunning).to_string();
            self.set_notice(NoticeLevel::Error, message);
        }
    }

    pub fn detect_ip(&mut self) {
        let _ = self.commands.send(Command::DetectIp);
    }

    pub fn persist(&mut self) {
        if let Err(err) = storage::save_config(&self.config) {
            let message = self.tr(Text::SaveConfigFailed, &[("err", &format!("{err:#}"))]);
            self.set_notice(NoticeLevel::Error, message);
            return;
        }
        let _ = self
            .commands
            .send(Command::ConfigChanged(Box::new(self.config.clone())));
    }

    pub fn set_notice(&mut self, level: NoticeLevel, text: impl Into<String>) {
        self.notice = Some(Notice {
            level,
            text: text.into(),
        });
    }

    fn handle_event(&mut self, event: Event, cx: &mut Context<Self>) {
        match event {
            Event::Log(entry) => {
                self.logs.push(entry);
                if self.logs.len() > 500 {
                    let overflow = self.logs.len() - 500;
                    self.logs.drain(0..overflow);
                }
            }
            Event::IpDetected { ip } => {
                self.public_ip = Some(ip);
                self.last_ip_at = Some(now_text());
            }
            Event::IpFailed { message } => {
                let message = self.tr(Text::IpDetectionFailed, &[("message", &message)]);
                self.set_notice(NoticeLevel::Error, message);
            }
            Event::SyncStarted { manual } => {
                self.sync_state = SyncState::Running;
                if manual {
                    self.set_notice(NoticeLevel::Info, self.t(Text::ManualSyncStarted));
                }
            }
            Event::SyncFinished { ok, message } => {
                self.sync_state = SyncState::Idle;
                self.last_sync_at = Some(now_text());
                self.set_notice(
                    if ok {
                        NoticeLevel::Success
                    } else {
                        NoticeLevel::Warn
                    },
                    message,
                );
            }
            Event::TargetStatus {
                target_id,
                phase,
                message,
            } => {
                self.target_status.insert(target_id, (phase, message));
            }
            Event::AccountTested {
                account_id,
                ok,
                message,
            } => {
                let name = self
                    .config
                    .account(&account_id)
                    .map(|account| account.name.clone())
                    .unwrap_or_else(|| self.t(Text::AccountFallback).to_string());
                self.set_notice(
                    if ok {
                        NoticeLevel::Success
                    } else {
                        NoticeLevel::Error
                    },
                    format!("{name}: {message}"),
                );
            }
            Event::Tray(action) => match action {
                crate::worker::TrayAction::ShowWindow => platform::show_main_window(),
                crate::worker::TrayAction::HideWindow => platform::hide_main_window(),
                crate::worker::TrayAction::SyncNow => self.sync_now(true),
                crate::worker::TrayAction::OpenConfigDir => storage::open_config_dir(),
                crate::worker::TrayAction::Quit => {
                    let _ = self.commands.send(Command::Shutdown);
                    self.allow_close.store(true, Ordering::SeqCst);
                    cx.quit();
                }
            },
        }
    }

    // ---------------------------------------------------------------- layout

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let language = self.language;
        let item = |page: Page, cx: &mut Context<Self>| {
            SidebarMenuItem::new(page.title(language))
                .icon(page.icon())
                .active(self.page == page)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.page = page;
                    cx.notify();
                }))
        };

        Sidebar::new("app-sidebar")
            .collapsible(SidebarCollapsible::None)
            .w(px(236.))
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_2()
                        .child(
                            img(crate::icon::LOGO_PATH)
                                .w(px(32.))
                                .h(px(32.))
                                .flex_shrink_0(),
                        )
                        .child(
                            v_flex()
                                .overflow_hidden()
                                .child(div().font_semibold().child("Cloud Firewall"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(self.t(Text::AppTagline)),
                                ),
                        ),
                ),
            )
            .child(SidebarGroup::new(self.t(Text::NavWorkspace)).child(
                SidebarMenu::new().children([
                    item(Page::Overview, cx),
                    item(Page::Accounts, cx),
                    item(Page::Targets, cx),
                ]),
            ))
            .child(SidebarGroup::new(self.t(Text::NavSystem)).child(
                SidebarMenu::new().children([item(Page::Logs, cx), item(Page::Settings, cx)]),
            ))
            .footer(
                SidebarFooter::new().child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(div().size_2().rounded_full().bg(
                                    if self.sync_state == SyncState::Running {
                                        cx.theme().warning
                                    } else {
                                        cx.theme().success
                                    },
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(if self.sync_state == SyncState::Running {
                                            self.t(Text::WorkerSyncing)
                                        } else {
                                            self.t(Text::WorkerIdle)
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!(
                                    "v{} · {}",
                                    env!("CARGO_PKG_VERSION"),
                                    self.t(Text::LicenseMit)
                                )),
                        )
                        .child(
                            Button::new("footer-github")
                                .ghost()
                                .xsmall()
                                .icon(Icon::from(gpui_kit::assets::IconName::Github))
                                .label("GitHub")
                                .tooltip(platform::REPOSITORY_URL)
                                .on_click(|_, _, cx| cx.open_url(platform::REPOSITORY_URL)),
                        ),
                ),
            )
    }

    fn render_header(&self, cx: &mut Context<Self>) -> Div {
        let page = self.page;
        let language = self.language;
        h_flex()
            .justify_between()
            .flex_shrink_0()
            .px_6()
            .py_4()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                v_flex()
                    .gap_1()
                    .child(div().text_lg().font_semibold().child(page.title(language)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(page.subtitle(language)),
                    ),
            )
            .child(h_flex().gap_2().when(page == Page::Overview, |this| {
                this.child(
                    Button::new("detect-ip")
                        .outline()
                        .icon(IconName::Globe)
                        .label(self.t(Text::DetectIp))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.detect_ip();
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("sync-now")
                        .primary()
                        .icon(IconName::RefreshCw)
                        .label(self.t(Text::SyncNow))
                        .loading(self.sync_state == SyncState::Running)
                        .disabled(self.sync_state == SyncState::Running)
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sync_now(true);
                            cx.notify();
                        })),
                )
            }))
    }

    fn render_notice(&self, cx: &App) -> Option<Div> {
        let notice = self.notice.as_ref()?;
        let (color, icon) = match notice.level {
            NoticeLevel::Info => (cx.theme().info, IconName::Info),
            NoticeLevel::Success => (cx.theme().success, IconName::CircleCheck),
            NoticeLevel::Warn => (cx.theme().warning, IconName::TriangleAlert),
            NoticeLevel::Error => (cx.theme().danger, IconName::CircleX),
        };
        Some(
            h_flex()
                .gap_2()
                .items_start()
                .p_3()
                .rounded(cx.theme().radius)
                .bg(color.opacity(0.10))
                .border_1()
                .border_color(color.opacity(0.35))
                .child(Icon::new(icon).size_4().text_color(color))
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(color)
                        .child(notice.text.clone()),
                ),
        )
    }

    fn page_body(&mut self, cx: &mut Context<Self>) -> AnyElement {
        match self.page {
            Page::Overview => self.render_overview(cx),
            Page::Accounts => self.render_accounts(cx),
            Page::Targets => self.render_targets(cx),
            Page::Logs => self.render_logs(cx),
            Page::Settings => self.render_settings(cx),
        }
    }

    // -------------------------------------------------------------- overview

    fn stat_card(
        &self,
        cx: &App,
        icon: IconName,
        label: &str,
        value: String,
        hint: String,
        color: Hsla,
    ) -> Div {
        v_flex()
            .flex_1()
            .min_w(px(150.))
            .gap_2()
            .p_4()
            .rounded(cx.theme().radius_lg)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().popover)
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .font_medium()
                            .text_color(cx.theme().muted_foreground)
                            .child(label.to_string()),
                    )
                    .child(Icon::new(icon).size_4().text_color(color)),
            )
            .child(
                div()
                    .text_xl()
                    .font_semibold()
                    .whitespace_nowrap()
                    .child(value),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(hint),
            )
    }

    fn render_overview(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let enabled_accounts = self
            .config
            .accounts
            .iter()
            .filter(|account| account.enabled)
            .count();
        let enabled_targets = self.config.enabled_target_count();
        let dry_run = self.config.settings.dry_run;

        let last_ip_hint = self
            .last_ip_at
            .clone()
            .map(|at| self.tr(Text::CheckedAt, &[("time", &at)]))
            .unwrap_or_else(|| self.t(Text::NotCheckedYet).to_string());

        let stats = h_flex()
            .gap_3()
            .items_stretch()
            .child(self.stat_card(
                cx,
                IconName::Globe,
                self.t(Text::PublicIp),
                self.public_ip.clone().unwrap_or_else(|| "—".to_string()),
                last_ip_hint,
                cx.theme().primary,
            ))
            .child(
                self.stat_card(
                    cx,
                    IconName::RefreshCw,
                    self.t(Text::LastSync),
                    self.last_sync_at
                        .clone()
                        .unwrap_or_else(|| self.t(Text::Never).to_string()),
                    if dry_run {
                        self.t(Text::DryRunOnHint).to_string()
                    } else {
                        self.t(Text::AutoUpdatedHint).to_string()
                    },
                    cx.theme().success,
                ),
            )
            .child(self.stat_card(
                cx,
                IconName::User,
                self.t(Text::Accounts),
                format!("{} / {}", enabled_accounts, self.config.accounts.len()),
                self.t(Text::EnabledTotal).to_string(),
                cx.theme().info,
            ))
            .child(self.stat_card(
                cx,
                IconName::Network,
                self.t(Text::Targets),
                format!("{} / {}", enabled_targets, self.config.targets.len()),
                self.t(Text::EnabledTotal).to_string(),
                cx.theme().warning,
            ));

        let targets: Vec<(String, String, String)> = self
            .config
            .targets
            .iter()
            .filter(|target| target.enabled)
            .map(|target| {
                (
                    target.id.clone(),
                    target.name.clone(),
                    target.security_group_id.clone(),
                )
            })
            .collect();

        let status_rows: Vec<AnyElement> = if targets.is_empty() {
            vec![
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.t(Text::NoTargetsHint))
                    .into_any_element(),
            ]
        } else {
            targets
                .iter()
                .map(|(id, name, group)| {
                    let (phase, message) = self.target_status.get(id).cloned().unwrap_or((
                        TargetPhase::Idle,
                        self.t(Text::WaitingNextCheck).to_string(),
                    ));
                    let (label, color) = match phase {
                        TargetPhase::Idle => {
                            (self.t(Text::StatusIdle), cx.theme().muted_foreground)
                        }
                        TargetPhase::Running => (self.t(Text::StatusSyncing), cx.theme().warning),
                        TargetPhase::Ok => (self.t(Text::StatusInSync), cx.theme().success),
                        TargetPhase::Error => (self.t(Text::StatusError), cx.theme().danger),
                    };
                    h_flex()
                        .justify_between()
                        .gap_3()
                        .py_2()
                        .child(
                            v_flex()
                                .gap_1()
                                .min_w(px(120.))
                                .child(div().text_sm().font_medium().child(name.clone()))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(group.clone()),
                                ),
                        )
                        .child(
                            v_flex()
                                .items_end()
                                .gap_1()
                                .child(pill(label, color))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(message),
                                ),
                        )
                        .into_any_element()
                })
                .collect()
        };

        let status_card =
            card(cx)
                .child(
                    h_flex()
                        .justify_between()
                        .child(div().font_semibold().child(self.t(Text::TargetStatus)))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(self.tr(
                                    Text::ActiveCount,
                                    &[("count", &targets.len().to_string())],
                                )),
                        ),
                )
                .children(status_rows);

        let recent: Vec<Div> = self
            .logs
            .iter()
            .rev()
            .take(8)
            .map(|entry| log_row(entry, cx))
            .collect();

        let recent_card = card(cx)
            .child(
                h_flex()
                    .justify_between()
                    .child(div().font_semibold().child(self.t(Text::RecentActivity)))
                    .child(
                        Button::new("view-logs")
                            .ghost()
                            .small()
                            .label(self.t(Text::ViewAll))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page = Page::Logs;
                                cx.notify();
                            })),
                    ),
            )
            .when(recent.is_empty(), |this| {
                this.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.t(Text::NothingHappened)),
                )
            })
            .children(recent);

        v_flex()
            .gap_5()
            .child(stats)
            .child(
                h_flex()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("quick-sync")
                                    .primary()
                                    .icon(IconName::RefreshCw)
                                    .label(self.t(Text::SyncSecurityGroups))
                                    .loading(self.sync_state == SyncState::Running)
                                    .disabled(self.sync_state == SyncState::Running)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sync_now(true);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("dry-run-toggle")
                                    .outline()
                                    .icon(IconName::Eye)
                                    .label(if dry_run {
                                        self.t(Text::DryRunOn)
                                    } else {
                                        self.t(Text::DryRunOff)
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.config.settings.dry_run =
                                            !this.config.settings.dry_run;
                                        this.persist();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(self.tr(
                                Text::NextCheck,
                                &[("secs", &self.config.sync_interval_secs().to_string())],
                            )),
                    ),
            )
            .child(status_card)
            .child(recent_card)
            .into_any_element()
    }

    // -------------------------------------------------------------- accounts

    fn render_accounts(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let accounts = self.config.accounts.clone();

        let header = h_flex()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.tr(
                        Text::AccountsSummary,
                        &[("count", &accounts.len().to_string())],
                    )),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("add-aliyun")
                            .primary()
                            .icon(IconName::Plus)
                            .label(self.t(Text::AliyunAccount))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_account_dialog(ProviderKind::Aliyun, None, window, cx);
                            })),
                    )
                    .child(
                        Button::new("add-tencent")
                            .outline()
                            .icon(IconName::Plus)
                            .label(self.t(Text::TencentAccount))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_account_dialog(ProviderKind::Tencent, None, window, cx);
                            })),
                    ),
            );

        let mut list = v_flex().gap_3();
        if accounts.is_empty() {
            list = list.child(empty_state(
                cx,
                self.t(Text::NoAccounts),
                self.t(Text::NoAccountsHint),
            ));
        }

        for account in accounts {
            let id = account.id.clone();
            let provider = account.provider;
            let color = provider_color(provider, cx);
            let target_count = self.config.targets_for(&id).count();
            let key_label = if account.access_key_id.trim().is_empty() {
                self.t(Text::NotSet).to_string()
            } else {
                account.masked_key()
            };

            list = list.child(
                card(cx).child(
                    h_flex()
                        .justify_between()
                        .gap_3()
                        .child(
                            v_flex()
                                .gap_2()
                                .min_w(px(120.))
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_base()
                                                .font_semibold()
                                                .child(account.name.clone()),
                                        )
                                        .child(pill(provider.short_label(), color))
                                        .when(!account.enabled, |this| {
                                            this.child(pill(
                                                self.t(Text::Disabled),
                                                cx.theme().muted_foreground,
                                            ))
                                        }),
                                )
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(self.tr(
                                            Text::RegionLabel,
                                            &[("region", account.region_or_default())],
                                        ))
                                        .child(self.tr(Text::KeyLabel, &[("key", &key_label)]))
                                        .child(self.tr(
                                            Text::TargetCount,
                                            &[("count", &target_count.to_string())],
                                        )),
                                )
                                .when(!account.remark.trim().is_empty(), |this| {
                                    this.child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(account.remark.clone()),
                                    )
                                }),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Switch::new("account-enabled")
                                        .checked(account.enabled)
                                        .tooltip(self.t(Text::EnableAccountTooltip))
                                        .on_change(cx.listener({
                                            let id = id.clone();
                                            move |this, value, _, cx| {
                                                if let Some(account) = this
                                                    .config
                                                    .accounts
                                                    .iter_mut()
                                                    .find(|account| account.id == id)
                                                {
                                                    account.enabled = *value;
                                                }
                                                this.persist();
                                                cx.notify();
                                            }
                                        })),
                                )
                                .child(
                                    Button::new("test-account")
                                        .outline()
                                        .small()
                                        .icon(IconName::Play)
                                        .label(self.t(Text::Test))
                                        .on_click(cx.listener({
                                            let id = id.clone();
                                            move |this, _, _, cx| {
                                                this.set_notice(
                                                    NoticeLevel::Info,
                                                    this.t(Text::TestingCredential),
                                                );
                                                let _ = this.commands.send(Command::TestAccount {
                                                    account_id: id.clone(),
                                                });
                                                cx.notify();
                                            }
                                        })),
                                )
                                .child(
                                    Button::new("edit-account")
                                        .outline()
                                        .small()
                                        .icon(IconName::Settings2)
                                        .label(self.t(Text::Edit))
                                        .on_click(cx.listener({
                                            let id = id.clone();
                                            move |this, _, window, cx| {
                                                this.open_account_dialog(
                                                    ProviderKind::Aliyun,
                                                    Some(id.clone()),
                                                    window,
                                                    cx,
                                                );
                                            }
                                        })),
                                )
                                .child(
                                    Button::new("delete-account")
                                        .ghost()
                                        .small()
                                        .icon(IconName::Delete)
                                        .on_click(cx.listener({
                                            let id = id.clone();
                                            let name = account.name.clone();
                                            move |this, _, window, cx| {
                                                this.confirm_delete_account(
                                                    id.clone(),
                                                    name.clone(),
                                                    window,
                                                    cx,
                                                );
                                            }
                                        })),
                                ),
                        ),
                ),
            );
        }

        v_flex()
            .gap_4()
            .child(header)
            .child(list)
            .into_any_element()
    }

    // --------------------------------------------------------------- targets

    fn render_targets(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let accounts = self.config.accounts.clone();
        let targets = self.config.targets.clone();

        let mut content = v_flex().gap_4();

        if accounts.is_empty() {
            content = content.child(empty_state(
                cx,
                self.t(Text::AddAccountFirst),
                self.t(Text::AddAccountFirstHint),
            ));
        } else {
            content = content.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.t(Text::TargetsIntro)),
            );
        }

        for account in accounts {
            let account_id = account.id.clone();
            let owned: Vec<_> = targets
                .iter()
                .filter(|target| target.account_id == account_id)
                .cloned()
                .collect();
            let color = provider_color(account.provider, cx);

            let mut card_view = card(cx).child(
                h_flex()
                    .justify_between()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().font_semibold().child(account.name.clone()))
                            .child(pill(account.provider.short_label(), color))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(account.region_or_default().to_string()),
                            ),
                    )
                    .child(
                        Button::new("add-target")
                            .outline()
                            .small()
                            .icon(IconName::Plus)
                            .label(self.t(Text::AddTarget))
                            .disabled(!account.enabled)
                            .on_click(cx.listener({
                                let account_id = account_id.clone();
                                move |this, _, window, cx| {
                                    this.open_target_dialog(account_id.clone(), None, window, cx);
                                }
                            })),
                    ),
            );

            if owned.is_empty() {
                card_view = card_view.child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(self.t(Text::NoTargetsForAccount)),
                );
            }

            for target in owned {
                let target_id = target.id.clone();
                let target_name = target.name.clone();
                let rules_text = describe_port_rules(&target.rules, self.t(Text::NoPorts));
                let region_text = self.tr(
                    Text::RegionLower,
                    &[(
                        "region",
                        if target.region.trim().is_empty() {
                            account.region_or_default()
                        } else {
                            &target.region
                        },
                    )],
                );
                let (phase, message) = self
                    .target_status
                    .get(&target_id)
                    .cloned()
                    .unwrap_or((TargetPhase::Idle, String::new()));
                let (status_label, status_color) = match phase {
                    TargetPhase::Idle => (self.t(Text::StatusIdle), cx.theme().muted_foreground),
                    TargetPhase::Running => (self.t(Text::StatusSyncing), cx.theme().warning),
                    TargetPhase::Ok => (self.t(Text::StatusInSync), cx.theme().success),
                    TargetPhase::Error => (self.t(Text::StatusError), cx.theme().danger),
                };

                card_view = card_view.child(
                    v_flex()
                        .gap_2()
                        .p_3()
                        .rounded(cx.theme().radius)
                        .bg(cx.theme().muted.opacity(0.35))
                        .child(
                            h_flex()
                                .justify_between()
                                .gap_3()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .min_w(px(120.))
                                        .child(
                                            h_flex()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_medium()
                                                        .child(target_name.clone()),
                                                )
                                                .child(pill(status_label, status_color))
                                                .when(!target.enabled, |this| {
                                                    this.child(pill(
                                                        self.t(Text::Paused),
                                                        cx.theme().muted_foreground,
                                                    ))
                                                }),
                                        )
                                        .child(
                                            h_flex()
                                                .gap_3()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(target.security_group_id.clone())
                                                .child(region_text)
                                                .child(rules_text),
                                        )
                                        .when(!message.is_empty(), |this| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(status_color)
                                                    .child(message.clone()),
                                            )
                                        }),
                                )
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Switch::new("target-enabled")
                                                .checked(target.enabled)
                                                .on_change(cx.listener({
                                                    let id = target_id.clone();
                                                    move |this, value, _, cx| {
                                                        if let Some(target) = this
                                                            .config
                                                            .targets
                                                            .iter_mut()
                                                            .find(|target| target.id == id)
                                                        {
                                                            target.enabled = *value;
                                                        }
                                                        this.persist();
                                                        cx.notify();
                                                    }
                                                })),
                                        )
                                        .child(
                                            Button::new("edit-target")
                                                .outline()
                                                .small()
                                                .icon(IconName::Settings2)
                                                .label(self.t(Text::Edit))
                                                .on_click(cx.listener({
                                                    let account_id = account_id.clone();
                                                    let id = target_id.clone();
                                                    move |this, _, window, cx| {
                                                        this.open_target_dialog(
                                                            account_id.clone(),
                                                            Some(id.clone()),
                                                            window,
                                                            cx,
                                                        );
                                                    }
                                                })),
                                        )
                                        .child(
                                            Button::new("delete-target")
                                                .ghost()
                                                .small()
                                                .icon(IconName::Delete)
                                                .on_click(cx.listener({
                                                    let id = target_id.clone();
                                                    let name = target_name.clone();
                                                    move |this, _, window, cx| {
                                                        this.confirm_delete_target(
                                                            id.clone(),
                                                            name.clone(),
                                                            window,
                                                            cx,
                                                        );
                                                    }
                                                })),
                                        ),
                                ),
                        ),
                );
            }

            content = content.child(card_view);
        }

        content.into_any_element()
    }

    // ------------------------------------------------------------------ logs

    fn render_logs(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let rows: Vec<Div> = self.logs.iter().map(|entry| log_row(entry, cx)).collect();

        let toolbar = h_flex()
            .justify_between()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.tr(
                        Text::LogEntries,
                        &[
                            ("count", &self.logs.len().to_string()),
                            ("path", &storage::log_path().display().to_string()),
                        ],
                    )),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("open-log-folder")
                            .outline()
                            .small()
                            .icon(IconName::Folder)
                            .label(self.t(Text::OpenFolder))
                            .on_click(|_, _, _| storage::open_log_dir()),
                    )
                    .child(
                        Button::new("clear-logs")
                            .ghost()
                            .small()
                            .icon(IconName::Delete)
                            .label(self.t(Text::ClearView))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.logs.clear();
                                cx.notify();
                            })),
                    ),
            );

        let body = card(cx).when(rows.is_empty(), |this| {
            this.child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.t(Text::LogEmpty)),
            )
        });

        v_flex()
            .gap_3()
            .child(toolbar)
            .child(body.children(rows))
            .into_any_element()
    }

    // -------------------------------------------------------------- settings

    fn render_settings(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let dry_run = self.config.settings.dry_run;
        let close_to_tray = self.config.settings.close_to_tray;
        let autostart_on = self.config.settings.autostart;
        let start_minimized = self.config.settings.start_minimized;
        let language = self.language;

        let language_card = card(cx)
            .child(section_title(
                self.t(Text::SectionLanguage),
                self.t(Text::SectionLanguageHint),
                cx,
            ))
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children(Language::all().into_iter().map(|candidate| {
                        let button =
                            Button::new(SharedString::from(format!("lang-{}", candidate.code())))
                                .small()
                                .label(candidate.native_name());
                        let button = if candidate == language {
                            button.primary()
                        } else {
                            button.outline()
                        };
                        button.on_click(cx.listener(move |this, _, window, cx| {
                            this.set_language(candidate, window, cx);
                        }))
                    })),
            );

        let sync_card = card(cx)
            .child(section_title(
                self.t(Text::SectionSync),
                self.t(Text::SectionSyncHint),
                cx,
            ))
            .child(field(
                self.t(Text::CheckInterval),
                Input::new(&self.settings_interval),
                Some(self.t(Text::CheckIntervalHint)),
                cx,
            ))
            .child(field(
                self.t(Text::ManagedPrefix),
                Input::new(&self.settings_prefix),
                Some(self.t(Text::ManagedPrefixHint)),
                cx,
            ))
            .child(field(
                self.t(Text::IpServices),
                Input::new(&self.settings_services),
                Some(self.t(Text::IpServicesHint)),
                cx,
            ));

        let behavior_card = card(cx)
            .child(section_title(
                self.t(Text::SectionBehaviour),
                self.t(Text::SectionBehaviourHint),
                cx,
            ))
            .child(setting_row(
                self.t(Text::DryRun),
                self.t(Text::DryRunHint),
                Switch::new("setting-dry-run")
                    .checked(dry_run)
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.config.settings.dry_run = *value;
                        this.persist();
                        cx.notify();
                    })),
                cx,
            ))
            .child(setting_row(
                self.t(Text::CloseToTray),
                self.t(Text::CloseToTrayHint),
                Switch::new("setting-close-to-tray")
                    .checked(close_to_tray)
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.config.settings.close_to_tray = *value;
                        this.close_to_tray.store(*value, Ordering::SeqCst);
                        this.persist();
                        cx.notify();
                    })),
                cx,
            ))
            .child(setting_row(
                self.t(Text::StartWithWindows),
                self.t(Text::StartWithWindowsHint),
                Switch::new("setting-autostart")
                    .checked(autostart_on)
                    .on_change(cx.listener(|this, value, _, cx| {
                        match autostart::set_enabled(*value) {
                            Ok(()) => {
                                this.config.settings.autostart = *value;
                                this.persist();
                                this.set_notice(
                                    NoticeLevel::Success,
                                    if *value {
                                        this.t(Text::AutostartEnabled)
                                    } else {
                                        this.t(Text::AutostartDisabled)
                                    },
                                );
                            }
                            Err(err) => {
                                let message =
                                    this.tr(Text::AutostartFailed, &[("err", &format!("{err:#}"))]);
                                this.set_notice(NoticeLevel::Error, message);
                            }
                        }
                        cx.notify();
                    })),
                cx,
            ))
            .child(setting_row(
                self.t(Text::StartMinimized),
                self.t(Text::StartMinimizedHint),
                Switch::new("setting-start-min")
                    .checked(start_minimized)
                    .on_change(cx.listener(|this, value, _, cx| {
                        this.config.settings.start_minimized = *value;
                        this.persist();
                        cx.notify();
                    })),
                cx,
            ));

        let maintenance = card(cx)
            .child(section_title(
                self.t(Text::SectionFiles),
                self.t(Text::SectionFilesHint),
                cx,
            ))
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("open-config")
                            .outline()
                            .icon(IconName::Folder)
                            .label(self.t(Text::OpenConfigFolder))
                            .on_click(|_, _, _| storage::open_config_dir()),
                    )
                    .child(
                        Button::new("open-logs")
                            .outline()
                            .icon(IconName::FileText)
                            .label(self.t(Text::OpenLogFolder))
                            .on_click(|_, _, _| storage::open_log_dir()),
                    )
                    .child(
                        Button::new("reset-usage")
                            .ghost()
                            .icon(IconName::Eye)
                            .label(self.t(Text::PreviewPlan))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.sync_now(true);
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(self.tr(
                        Text::ConfigFile,
                        &[("path", &storage::config_path().display().to_string())],
                    )),
            );

        let version = env!("CARGO_PKG_VERSION");
        let about = card(cx)
            .child(section_title(
                self.t(Text::SectionAbout),
                &self.tr(Text::SectionAboutHint, &[("version", version)]),
                cx,
            ))
            .child(link_row(
                self.t(Text::SourceCode),
                self.t(Text::SourceCodeHint),
                platform::REPOSITORY_URL,
                gpui_kit::assets::IconName::Github,
                self.t(Text::OpenLink),
                cx,
            ))
            .child(link_row(
                self.t(Text::LicenseMit),
                self.t(Text::LicenseMitHint),
                platform::LICENSE_URL,
                gpui_kit::assets::IconName::BookOpen,
                self.t(Text::ReadLicense),
                cx,
            ))
            .child(link_row(
                self.t(Text::ProjectHomepage),
                "rustsilk.com",
                platform::HOMEPAGE_URL,
                gpui_kit::assets::IconName::Globe,
                self.t(Text::OpenLink),
                cx,
            ));

        v_flex()
            .gap_4()
            .child(language_card)
            .child(sync_card)
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("save-settings")
                            .primary()
                            .icon(IconName::Check)
                            .label(self.t(Text::SaveSettings))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_settings(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("reload-settings")
                            .outline()
                            .icon(IconName::RotateCw)
                            .label(self.t(Text::Revert))
                            .on_click(cx.listener(|this, _, window, cx| {
                                let interval = this.settings_interval.clone();
                                let services = this.settings_services.clone();
                                let prefix = this.settings_prefix.clone();
                                let interval_value =
                                    this.config.settings.check_interval_secs.to_string();
                                let services_value = this.config.settings.ip_services.join(", ");
                                let prefix_value = this.config.settings.rule_prefix.clone();
                                this.set_input(&interval, interval_value, window, cx);
                                this.set_input(&services, services_value, window, cx);
                                this.set_input(&prefix, prefix_value, window, cx);
                            })),
                    ),
            )
            .child(behavior_card)
            .child(maintenance)
            .child(about)
            .into_any_element()
    }

    fn set_input(
        &mut self,
        input: &Entity<InputState>,
        value: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let value = value.into();
        input.update(cx, |state, cx| {
            state.set_value(value, window, cx);
        });
    }

    fn save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let interval_text = self.settings_interval.read(cx).value().to_string();
        let services_text = self.settings_services.read(cx).value().to_string();
        let prefix_text = self.settings_prefix.read(cx).value().to_string();

        let interval = match interval_text.trim().parse::<u64>() {
            Ok(value) if value >= MIN_INTERVAL_SECS => value,
            _ => {
                let message = self.t(Text::IntervalInvalid).to_string();
                window.push_notification(message.clone(), cx);
                self.set_notice(NoticeLevel::Error, message);
                return;
            }
        };

        let services: Vec<String> = services_text
            .split([',', '\n'])
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect();
        if services.is_empty() {
            window.push_notification(self.t(Text::AddService), cx);
            return;
        }

        let prefix = prefix_text.trim().to_string();
        if prefix.is_empty() {
            window.push_notification(self.t(Text::PrefixEmpty), cx);
            return;
        }

        self.config.settings.check_interval_secs = interval;
        self.config.settings.ip_services = services;
        self.config.settings.rule_prefix = prefix;
        self.persist();
        self.set_notice(NoticeLevel::Success, self.t(Text::SettingsSaved));
        window.push_notification(self.t(Text::SettingsSavedToast), cx);
    }
}

// ------------------------------------------------------------------ helpers

fn card(cx: &App) -> Div {
    v_flex()
        .gap_3()
        .p_4()
        .rounded(cx.theme().radius_lg)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().popover)
}

fn section_title(title: &str, hint: &str, cx: &App) -> Div {
    v_flex()
        .gap_1()
        .child(div().font_semibold().child(title.to_string()))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(hint.to_string()),
        )
}

fn field(label: &str, input: impl IntoElement, hint: Option<&str>, cx: &App) -> Div {
    v_flex()
        .gap_1()
        .child(div().text_xs().font_medium().child(label.to_string()))
        .child(input)
        .when_some(hint, |this, hint| {
            this.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(hint.to_string()),
            )
        })
}

fn setting_row(label: &str, hint: &str, switch: Switch, cx: &App) -> Div {
    h_flex()
        .justify_between()
        .gap_4()
        .py_2()
        .child(
            v_flex()
                .gap_1()
                .child(div().text_sm().font_medium().child(label.to_string()))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(hint.to_string()),
                ),
        )
        .child(switch)
}

/// A settings row whose control is an external link button.
fn link_row(
    label: &str,
    hint: &str,
    url: &'static str,
    icon: gpui_kit::assets::IconName,
    action: &str,
    cx: &App,
) -> Div {
    h_flex()
        .justify_between()
        .items_center()
        .gap_4()
        .py_2()
        .child(
            v_flex()
                .gap_1()
                .min_w(px(0.))
                .child(div().text_sm().font_medium().child(label.to_string()))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(hint.to_string()),
                ),
        )
        .child(
            Button::new(SharedString::from(url))
                .outline()
                .small()
                .flex_shrink_0()
                .icon(Icon::from(icon))
                .label(action.to_string())
                .on_click(move |_, _, cx| cx.open_url(url)),
        )
}

fn pill(text: impl Into<SharedString>, color: Hsla) -> Div {
    div()
        .flex()
        .items_center()
        .px(px(7.))
        .py(px(2.))
        .rounded_full()
        .bg(color.opacity(0.14))
        .text_color(color)
        .text_xs()
        .child(text.into())
}

fn empty_state(cx: &App, title: &str, hint: &str) -> Div {
    v_flex()
        .gap_2()
        .items_center()
        .py_8()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_10()
                .rounded_full()
                .bg(cx.theme().muted)
                .child(
                    Icon::new(IconName::Info)
                        .size_5()
                        .text_color(cx.theme().muted_foreground),
                ),
        )
        .child(div().font_medium().child(title.to_string()))
        .child(
            div()
                .max_w(px(420.))
                .text_center()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(hint.to_string()),
        )
}

fn log_row(entry: &LogEntry, cx: &App) -> Div {
    let color = match entry.level {
        Level::Info => cx.theme().muted_foreground,
        Level::Success => cx.theme().success,
        Level::Warn => cx.theme().warning,
        Level::Error => cx.theme().danger,
    };
    h_flex()
        .items_start()
        .gap_3()
        .py_1()
        .child(
            div()
                .w(px(160.))
                .flex_shrink_0()
                .font_family(cx.theme().mono_font_family.clone())
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(entry.at.clone()),
        )
        .child(
            div()
                .w(px(48.))
                .flex_shrink_0()
                .child(pill(entry.level.label(), color)),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.))
                .truncate()
                .text_xs()
                .text_color(cx.theme().foreground)
                .child(entry.message.clone()),
        )
}

fn provider_color(provider: ProviderKind, cx: &App) -> Hsla {
    let _ = cx;
    match provider {
        ProviderKind::Aliyun => hsla(24.0 / 360.0, 0.92, 0.52, 1.0),
        ProviderKind::Tencent => hsla(217.0 / 360.0, 0.88, 0.60, 1.0),
    }
}

fn now_text() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = v_flex()
            .flex_1()
            .h_full()
            .min_w(px(0.))
            .child(self.render_header(cx))
            .child(
                v_flex()
                    .id("page-scroll")
                    .flex_1()
                    .min_h(px(0.))
                    .p_6()
                    .gap_4()
                    .overflow_y_scrollbar()
                    .when_some(self.render_notice(cx), |this, notice| this.child(notice))
                    .child(self.page_body(cx)),
            );

        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_sidebar(cx))
            .child(content)
    }
}
