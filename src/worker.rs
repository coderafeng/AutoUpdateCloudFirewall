//! Background worker: periodic public-IP detection and security-group sync.

use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::Instant;

use crate::i18n::{self, Text};
use crate::logging::{Level, LogEntry, Logger};
use crate::model::{AppConfig, Target};
use crate::planner;
use crate::providers::{ApiClient, Credentials};
use crate::{ip, storage};

/// Requests from the UI to the worker.
pub enum Command {
    SyncNow { manual: bool },
    DetectIp,
    TestAccount { account_id: String },
    ConfigChanged(Box<AppConfig>),
    Shutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPhase {
    Idle,
    Running,
    Ok,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    ShowWindow,
    HideWindow,
    SyncNow,
    OpenConfigDir,
    Quit,
}

/// Notifications from the worker (and tray) to the UI.
pub enum Event {
    Log(LogEntry),
    IpDetected {
        ip: String,
    },
    IpFailed {
        message: String,
    },
    SyncStarted {
        manual: bool,
    },
    SyncFinished {
        ok: bool,
        message: String,
    },
    TargetStatus {
        target_id: String,
        phase: TargetPhase,
        message: String,
    },
    AccountTested {
        account_id: String,
        ok: bool,
        message: String,
    },
    Tray(TrayAction),
}

pub type EventSender = UnboundedSender<Event>;
pub type EventReceiver = UnboundedReceiver<Event>;

const RETRY_ATTEMPTS: usize = 3;
const API_SPACING: Duration = Duration::from_millis(450);

pub fn http_client() -> Client {
    Client::builder()
        .user_agent(concat!(
            "AutoUpdateCloudFirewall/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_default()
}

pub fn spawn(
    runtime: &tokio::runtime::Handle,
    config: AppConfig,
    commands: UnboundedReceiver<Command>,
    events: EventSender,
) {
    runtime.spawn(worker_loop(config, commands, events));
}

struct Worker {
    config: AppConfig,
    http: Client,
    logger: Logger,
    events: EventSender,
    last_ip: Option<String>,
}

async fn worker_loop(
    config: AppConfig,
    mut commands: UnboundedReceiver<Command>,
    events: EventSender,
) {
    let mut worker = Worker {
        config,
        http: http_client(),
        logger: Logger::new(),
        events,
        last_ip: None,
    };

    let mut next_sync = Instant::now() + Duration::from_secs(8);
    let secs = worker.config.sync_interval_secs().to_string();
    let started = worker.tr(Text::WStarted, &[("secs", &secs)]);
    worker.log(Level::Info, started);

    'worker: loop {
        tokio::select! {
            command = commands.recv() => {
                match command {
                    None | Some(Command::Shutdown) => break 'worker,
                    Some(Command::SyncNow { manual }) => {
                        worker.sync_now(manual).await;
                        next_sync = Instant::now() + worker.interval();
                    }
                    Some(Command::DetectIp) => {
                        worker.detect_only().await;
                    }
                    Some(Command::TestAccount { account_id }) => {
                        worker.test_account(&account_id).await;
                    }
                    Some(Command::ConfigChanged(config)) => {
                        let interval_changed =
                            config.sync_interval_secs() != worker.config.sync_interval_secs();
                        worker.config = *config;
                        if interval_changed {
                            next_sync = Instant::now() + worker.interval();
                        }
                        worker.log(Level::Info, worker.t(Text::WConfigUpdated));
                    }
                }
            }
            _ = tokio::time::sleep_until(next_sync) => {
                if worker.has_work() {
                    worker.sync_now(false).await;
                }
                next_sync = Instant::now() + worker.interval();
            }
        }
    }

    worker.log(Level::Info, worker.t(Text::WStopped));
}

impl Worker {
    fn interval(&self) -> Duration {
        Duration::from_secs(self.config.sync_interval_secs())
    }

    fn has_work(&self) -> bool {
        self.config.enabled_target_count() > 0
    }

    fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }

    fn log(&self, level: Level, message: impl Into<String>) {
        self.emit(Event::Log(self.logger.log(level, message)));
    }

    fn t(&self, text: Text) -> &'static str {
        i18n::t(self.config.settings.language, text)
    }

    fn tr(&self, text: Text, args: &[(&str, &str)]) -> String {
        i18n::tr(self.config.settings.language, text, args)
    }

    fn plan_summary(&self, plan: &planner::SyncPlan) -> String {
        let add = plan.create.len().to_string();
        let update = plan.update.len().to_string();
        let remove = plan.delete.len().to_string();
        let keep = plan.keep.to_string();
        self.tr(
            Text::PlanSummary,
            &[
                ("add", &add),
                ("update", &update),
                ("remove", &remove),
                ("keep", &keep),
            ],
        )
    }

    fn target_status(&self, target_id: &str, phase: TargetPhase, message: impl Into<String>) {
        self.emit(Event::TargetStatus {
            target_id: target_id.to_string(),
            phase,
            message: message.into(),
        });
    }

    async fn detect_only(&mut self) {
        match ip::detect_logged(&self.http, &self.config.settings.ip_services).await {
            Ok(ip) => {
                self.last_ip = Some(ip.clone());
                self.log(Level::Success, self.tr(Text::WCurrentIp, &[("ip", &ip)]));
                self.emit(Event::IpDetected { ip });
            }
            Err(err) => {
                let message = format!("{err:#}");
                self.log(Level::Error, message.clone());
                self.emit(Event::IpFailed { message });
            }
        }
    }

    async fn sync_now(&mut self, manual: bool) {
        self.emit(Event::SyncStarted { manual });

        if self.config.accounts.is_empty() || self.config.targets.is_empty() {
            self.log(Level::Warn, self.t(Text::WNothingToSync));
            self.emit(Event::SyncFinished {
                ok: false,
                message: self.t(Text::WNothingConfigured).to_string(),
            });
            return;
        }

        let ip = match ip::detect_logged(&self.http, &self.config.settings.ip_services).await {
            Ok(ip) => ip,
            Err(err) => match &self.last_ip {
                Some(cached) => {
                    let detail = format!("{err:#}");
                    self.log(
                        Level::Warn,
                        self.tr(
                            Text::WIpFailedReuse,
                            &[("err", &detail), ("cached", cached)],
                        ),
                    );
                    cached.clone()
                }
                None => {
                    self.log(Level::Error, format!("{err:#}"));
                    self.emit(Event::IpFailed {
                        message: format!("{err:#}"),
                    });
                    self.emit(Event::SyncFinished {
                        ok: false,
                        message: self.t(Text::WIpDetectFailedNotice).to_string(),
                    });
                    return;
                }
            },
        };

        let ip_changed = self.last_ip.as_deref() != Some(ip.as_str());
        self.last_ip = Some(ip.clone());
        self.emit(Event::IpDetected { ip: ip.clone() });
        self.log(
            Level::Info,
            if ip_changed {
                self.tr(Text::WIpChanged, &[("ip", &ip)])
            } else {
                self.tr(Text::WIpSame, &[("ip", &ip)])
            },
        );

        let prefix = self.config.settings.rule_prefix.trim().to_string();
        let dry_run = self.config.settings.dry_run;
        let targets: Vec<Target> = self
            .config
            .targets
            .iter()
            .filter(|target| target.enabled)
            .cloned()
            .collect();

        let mut ok_count = 0usize;
        let mut error_count = 0usize;
        let mut skipped = 0usize;

        for target in targets {
            let Some(account) = self.config.account(&target.account_id).cloned() else {
                self.log(
                    Level::Warn,
                    self.tr(Text::WMissingAccount, &[("name", &target.name)]),
                );
                skipped += 1;
                continue;
            };
            if !account.enabled {
                skipped += 1;
                continue;
            }

            self.target_status(&target.id, TargetPhase::Running, self.t(Text::WSyncing));

            match self
                .sync_target(&target, &account, &ip, &prefix, dry_run)
                .await
            {
                Ok(message) => {
                    ok_count += 1;
                    self.target_status(&target.id, TargetPhase::Ok, message.clone());
                }
                Err(err) => {
                    error_count += 1;
                    let message = format!("{err:#}");
                    self.target_status(&target.id, TargetPhase::Error, message.clone());
                    let label = target_label(&target);
                    self.log(
                        Level::Error,
                        self.tr(
                            Text::WTargetFailed,
                            &[("target", &label), ("message", &message)],
                        ),
                    );
                }
            }
        }

        let ok = error_count == 0;
        let ok_text = ok_count.to_string();
        let failed_text = error_count.to_string();
        let skipped_text = skipped.to_string();
        let summary = if ok_count == 0 && error_count == 0 {
            self.tr(Text::WNothingSynced, &[("skipped", &skipped_text)])
        } else {
            let mut summary = if skipped > 0 {
                self.tr(
                    Text::WSummarySkipped,
                    &[
                        ("ok", &ok_text),
                        ("failed", &failed_text),
                        ("skipped", &skipped_text),
                    ],
                )
            } else {
                self.tr(
                    Text::WSummary,
                    &[("ok", &ok_text), ("failed", &failed_text)],
                )
            };
            if dry_run {
                summary.push_str(self.t(Text::DryRunSuffix));
            }
            summary
        };
        self.log(
            if ok { Level::Success } else { Level::Error },
            self.tr(Text::WSyncFinished, &[("summary", &summary)]),
        );
        self.emit(Event::SyncFinished {
            ok,
            message: summary,
        });

        if dry_run {
            self.log(Level::Info, self.t(Text::WDryRunNoChange));
        }
    }

    async fn sync_target(
        &self,
        target: &Target,
        account: &crate::model::Account,
        ip: &str,
        prefix: &str,
        dry_run: bool,
    ) -> Result<String> {
        let region = if target.region.trim().is_empty() {
            account.region_or_default()
        } else {
            target.region.trim()
        };
        let secret = storage::load_secret(&account.id)
            .with_context(|| self.tr(Text::WSecretMissing, &[("name", &account.name)]))?;
        if account.access_key_id.trim().is_empty() {
            return Err(anyhow::anyhow!(
                self.tr(Text::WNoAccessKey, &[("name", &account.name)])
            ));
        }

        let client = ApiClient::new(
            account.provider,
            self.http.clone(),
            Credentials {
                access_key_id: account.access_key_id.clone(),
                secret,
            },
        );

        let snapshot = client
            .list_ingress(region, &target.security_group_id)
            .await
            .with_context(|| self.tr(Text::WReadGroup, &[("id", &target.security_group_id)]))?;

        let desired = planner::desired_rules(&target.rules, ip, prefix);
        let initial = planner::plan(&snapshot.rules, &desired, prefix);
        let label = target_label(target);

        // A keyword that cannot name exactly one rule would modify something
        // unexpected, so the whole target is skipped before any mutation.
        if let Some(conflict) = initial.ambiguous.first() {
            let port = conflict.desired.port_text();
            let keyword = conflict.desired.keyword.trim();
            let count = conflict.count.to_string();
            let message = if conflict.shared {
                self.tr(
                    Text::WSharedRule,
                    &[("target", &label), ("port", &port), ("keyword", keyword)],
                )
            } else {
                self.tr(
                    Text::WAmbiguousRule,
                    &[
                        ("target", &label),
                        ("port", &port),
                        ("keyword", keyword),
                        ("count", &count),
                    ],
                )
            };
            self.log(Level::Error, message.clone());
            return Err(anyhow::anyhow!(message));
        }

        self.log(
            Level::Info,
            format!("{label}: {}", self.plan_summary(&initial)),
        );

        if initial.is_empty() {
            return Ok(self.t(Text::WAlreadyUpToDate).to_string());
        }

        if dry_run {
            // Printed in the order the worker applies the plan:
            // delete, then update, then create.
            for rule in &initial.delete {
                self.log(
                    Level::Warn,
                    self.tr(
                        Text::WWouldRemove,
                        &[
                            ("target", &label),
                            ("protocol", &rule.protocol),
                            ("port", &rule.port),
                            ("cidr", &rule.cidr),
                            ("description", &rule.description),
                        ],
                    ),
                );
            }
            for rule in &initial.update {
                let port = rule.desired.port_text();
                self.log(
                    Level::Info,
                    self.tr(
                        Text::WWouldUpdate,
                        &[
                            ("target", &label),
                            ("protocol", rule.desired.protocol.label()),
                            ("port", &port),
                            ("description", &rule.remote.description),
                            ("from", &rule.remote.cidr),
                            ("to", &rule.desired.cidr),
                        ],
                    ),
                );
            }
            for rule in &initial.create {
                let port = rule.port_text();
                self.log(
                    Level::Info,
                    self.tr(
                        Text::WWouldAllow,
                        &[
                            ("target", &label),
                            ("protocol", rule.protocol.label()),
                            ("port", &port),
                            ("cidr", &rule.cidr),
                        ],
                    ),
                );
            }
            return Ok(self.tr(
                Text::WDryRunSummary,
                &[("summary", &self.plan_summary(&initial))],
            ));
        }

        // Delete managed rules that no longer match, one at a time so that
        // Tencent Cloud policy indices stay valid.
        for _ in 0..64 {
            let snapshot = client
                .list_ingress(region, &target.security_group_id)
                .await
                .with_context(|| self.tr(Text::WReadGroup, &[("id", &target.security_group_id)]))?;
            let plan = planner::plan(&snapshot.rules, &desired, prefix);
            let Some(rule) = plan.delete.first().cloned() else {
                break;
            };

            let mut attempt = 0usize;
            loop {
                attempt += 1;
                match client
                    .revoke(
                        region,
                        &target.security_group_id,
                        snapshot.version.as_deref(),
                        &rule,
                    )
                    .await
                {
                    Ok(()) => break,
                    Err(err) if attempt < RETRY_ATTEMPTS => {
                        let attempt_text = attempt.to_string();
                        self.log(
                            Level::Warn,
                            self.tr(
                                Text::WRemoveFailed,
                                &[
                                    ("target", &label),
                                    ("protocol", &rule.protocol),
                                    ("port", &rule.port),
                                    ("cidr", &rule.cidr),
                                    ("attempt", &attempt_text),
                                    ("attempts", &RETRY_ATTEMPTS.to_string()),
                                    ("err", &format!("{err:#}")),
                                ],
                            ),
                        );
                        tokio::time::sleep(Duration::from_millis(800 * attempt as u64)).await;
                    }
                    Err(err) => {
                        return Err(err).with_context(|| {
                            self.tr(
                                Text::WRemoveContext,
                                &[
                                    ("protocol", &rule.protocol),
                                    ("port", &rule.port),
                                    ("cidr", &rule.cidr),
                                ],
                            )
                        });
                    }
                }
            }

            self.log(
                Level::Info,
                self.tr(
                    Text::WRemoved,
                    &[
                        ("target", &label),
                        ("protocol", &rule.protocol),
                        ("port", &rule.port),
                        ("cidr", &rule.cidr),
                    ],
                ),
            );
            tokio::time::sleep(API_SPACING).await;
        }

        // Re-read after the deletes: the plan below reflects the group as it
        // is now. Aliyun rejects a modify that would duplicate another rule
        // (HTTP 400 InvalidOperation.RuleDuplicate), so a stale rule must be
        // gone before its address is written onto an adopted rule.
        let snapshot = client
            .list_ingress(region, &target.security_group_id)
            .await
            .with_context(|| self.tr(Text::WReadGroup, &[("id", &target.security_group_id)]))?;
        let plan = planner::plan(&snapshot.rules, &desired, prefix);

        // In-place updates: adopted rules keep their description and
        // priority, only the source address changes.
        let mut updated = 0usize;
        for entry in &plan.update {
            let port = entry.desired.port_text();
            let tencent = matches!(client.provider(), crate::model::ProviderKind::Tencent);
            let attempts = if tencent { 1 } else { RETRY_ATTEMPTS };
            let mut attempt = 0usize;
            loop {
                attempt += 1;
                // Tencent Cloud requires a fresh policy version for every call.
                let version = if tencent {
                    client
                        .list_ingress(region, &target.security_group_id)
                        .await
                        .with_context(|| self.t(Text::WRefreshVersion))?
                        .version
                } else {
                    snapshot.version.clone()
                };
                match client
                    .update_cidr(region, &target.security_group_id, version.as_deref(), entry)
                    .await
                {
                    Ok(()) => break,
                    Err(err) => {
                        let detail = format!("{err:#}");
                        // A duplicate is a permanent state, not a transient
                        // failure: retrying only sends the same rejected call.
                        let duplicate = detail.contains("RuleDuplicate");
                        if duplicate || attempt >= attempts {
                            let context = if duplicate {
                                self.tr(
                                    Text::WUpdateDuplicate,
                                    &[
                                        ("protocol", entry.desired.protocol.label()),
                                        ("port", &port),
                                        ("cidr", &entry.desired.cidr),
                                    ],
                                )
                            } else {
                                self.tr(
                                    Text::WUpdateContext,
                                    &[
                                        ("protocol", entry.desired.protocol.label()),
                                        ("port", &port),
                                    ],
                                )
                            };
                            return Err(err).with_context(|| context);
                        }
                        let attempt_text = attempt.to_string();
                        self.log(
                            Level::Warn,
                            self.tr(
                                Text::WUpdateFailed,
                                &[
                                    ("target", &label),
                                    ("protocol", entry.desired.protocol.label()),
                                    ("port", &port),
                                    ("attempt", &attempt_text),
                                    ("attempts", &attempts.to_string()),
                                    ("err", &detail),
                                ],
                            ),
                        );
                        tokio::time::sleep(Duration::from_millis(800 * attempt as u64)).await;
                    }
                }
            }

            updated += 1;
            self.log(
                Level::Success,
                self.tr(
                    Text::WUpdated,
                    &[
                        ("target", &label),
                        ("protocol", entry.desired.protocol.label()),
                        ("port", &port),
                        ("from", &entry.remote.cidr),
                        ("to", &entry.desired.cidr),
                    ],
                ),
            );
            tokio::time::sleep(API_SPACING).await;
        }

        let mut created = 0usize;

        for rule in &plan.create {
            let version = if matches!(client.provider(), crate::model::ProviderKind::Tencent) {
                let fresh = client
                    .list_ingress(region, &target.security_group_id)
                    .await
                    .with_context(|| self.t(Text::WRefreshVersion))?;
                fresh.version
            } else {
                snapshot.version.clone()
            };

            let mut attempt = 0usize;
            loop {
                attempt += 1;
                match client
                    .authorize(
                        region,
                        &target.security_group_id,
                        version.as_deref(),
                        rule,
                        self.config.settings.rule_priority,
                    )
                    .await
                {
                    Ok(()) => break,
                    Err(err) if attempt < RETRY_ATTEMPTS => {
                        let attempt_text = attempt.to_string();
                        let port = rule.port_text();
                        self.log(
                            Level::Warn,
                            self.tr(
                                Text::WAllowFailed,
                                &[
                                    ("target", &label),
                                    ("protocol", rule.protocol.label()),
                                    ("port", &port),
                                    ("attempt", &attempt_text),
                                    ("attempts", &RETRY_ATTEMPTS.to_string()),
                                    ("err", &format!("{err:#}")),
                                ],
                            ),
                        );
                        tokio::time::sleep(Duration::from_millis(800 * attempt as u64)).await;
                    }
                    Err(err) => {
                        let port = rule.port_text();
                        return Err(err).with_context(|| {
                            self.tr(
                                Text::WAllowContext,
                                &[("protocol", rule.protocol.label()), ("port", &port)],
                            )
                        });
                    }
                }
            }

            created += 1;
            let port = rule.port_text();
            self.log(
                Level::Success,
                self.tr(
                    Text::WAllowed,
                    &[
                        ("target", &label),
                        ("protocol", rule.protocol.label()),
                        ("port", &port),
                        ("cidr", &rule.cidr),
                    ],
                ),
            );
            tokio::time::sleep(API_SPACING).await;
        }

        let verify = client
            .list_ingress(region, &target.security_group_id)
            .await
            .unwrap_or(snapshot);
        let remaining = planner::plan(&verify.rules, &desired, prefix);

        if remaining.is_empty() {
            let active_text = remaining.keep.to_string();
            let created_text = created.to_string();
            let updated_text = updated.to_string();
            Ok(self.tr(
                Text::WInSync,
                &[
                    ("active", &active_text),
                    ("added", &created_text),
                    ("updated", &updated_text),
                ],
            ))
        } else {
            Err(anyhow::anyhow!(self.tr(
                Text::WOutOfSync,
                &[("summary", &self.plan_summary(&remaining))],
            )))
        }
    }

    async fn test_account(&self, account_id: &str) {
        let Some(account) = self.config.account(account_id).cloned() else {
            self.emit(Event::AccountTested {
                account_id: account_id.to_string(),
                ok: false,
                message: self.t(Text::WAccountGone).to_string(),
            });
            return;
        };

        let result = async {
            let secret = storage::load_secret(&account.id)
                .with_context(|| self.t(Text::WSecretMissingContext).to_string())?;
            let client = ApiClient::new(
                account.provider,
                self.http.clone(),
                Credentials {
                    access_key_id: account.access_key_id.clone(),
                    secret,
                },
            );
            client.check(account.region_or_default()).await
        }
        .await;

        match result {
            Ok(()) => {
                let message = self.t(Text::WCredentialValid);
                self.log(
                    Level::Success,
                    self.tr(
                        Text::WAccountTestOk,
                        &[("name", &account.name), ("message", message)],
                    ),
                );
                self.emit(Event::AccountTested {
                    account_id: account.id.clone(),
                    ok: true,
                    message: message.to_string(),
                });
            }
            Err(err) => {
                let message = format!("{err:#}");
                self.log(
                    Level::Error,
                    self.tr(
                        Text::WAccountTestFailed,
                        &[("name", &account.name), ("message", &message)],
                    ),
                );
                self.emit(Event::AccountTested {
                    account_id: account.id.clone(),
                    ok: false,
                    message,
                });
            }
        }
    }
}

fn target_label(target: &Target) -> String {
    format!("{} [{}]", target.name, target.security_group_id)
}
