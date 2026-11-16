//! Modal dialogs for account and target management.

use gpui_kit::component::{
    ActiveTheme, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::{DialogAction, DialogClose, DialogFooter},
    h_flex,
    input::Input,
    v_flex,
};
use gpui_kit::*;

use super::{AppView, NoticeLevel, field, provider_color};
use crate::i18n::{self, Text};
use crate::model::{Account, Protocol, ProviderKind, RuleSpec, Target};
use crate::{storage, worker::Command};

impl AppView {
    pub fn open_account_dialog(
        &mut self,
        provider: ProviderKind,
        account_id: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let language = self.language;
        let editing = account_id
            .as_deref()
            .and_then(|id| self.config.account(id))
            .cloned();

        self.editing_account = account_id.clone();
        self.account_provider = editing
            .as_ref()
            .map(|account| account.provider)
            .unwrap_or(provider);

        let (name, key, region) = match &editing {
            Some(account) => (
                account.name.clone(),
                account.access_key_id.clone(),
                account.region.clone(),
            ),
            None => (
                String::new(),
                String::new(),
                self.account_provider.default_region().to_string(),
            ),
        };

        self.set_input(&self.account_name.clone(), name, window, cx);
        self.set_input(&self.account_key.clone(), key, window, cx);
        self.set_input(&self.account_secret.clone(), String::new(), window, cx);
        self.set_input(&self.account_region.clone(), region, window, cx);

        let entity = cx.entity();
        let name_input = self.account_name.clone();
        let key_input = self.account_key.clone();
        let secret_input = self.account_secret.clone();
        let region_input = self.account_region.clone();
        let is_edit = account_id.is_some();
        let provider = self.account_provider;
        let color = provider_color(provider, cx);

        let title = i18n::t(
            language,
            if is_edit {
                Text::EditCloudAccount
            } else {
                Text::AddCloudAccount
            },
        );
        let ok_text = i18n::t(
            language,
            if is_edit {
                Text::Save
            } else {
                Text::AddAccount
            },
        );
        let secret_hint = i18n::t(
            language,
            if is_edit {
                Text::SecretKeepHint
            } else {
                Text::SecretStoreHint
            },
        );

        window.open_dialog(cx, move |dialog, _, cx| {
            let entity = entity.clone();
            dialog
                .w(px(480.))
                .title(title.to_string())
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(super::pill(provider.label(), color))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(i18n::t(language, Text::AccessKeyRequiredHint)),
                                ),
                        )
                        .child(field(
                            i18n::t(language, Text::AccountName),
                            Input::new(&name_input),
                            None,
                            cx,
                        ))
                        .child(field(
                            i18n::t(language, Text::AccessKeyId),
                            Input::new(&key_input),
                            None,
                            cx,
                        ))
                        .child(field(
                            i18n::t(language, Text::AccessKeySecret),
                            Input::new(&secret_input),
                            Some(secret_hint),
                            cx,
                        ))
                        .child(field(
                            i18n::t(language, Text::DefaultRegion),
                            Input::new(&region_input),
                            Some(i18n::t(language, Text::DefaultRegionHint)),
                            cx,
                        )),
                )
                .footer(
                    DialogFooter::new()
                        .gap_2()
                        .child(DialogClose::new().trigger(|button| {
                            button.outline().label(i18n::t(language, Text::Cancel))
                        }))
                        .child(
                            DialogAction::new().child(
                                Button::new("account-save")
                                    .primary()
                                    .label(ok_text.to_string()),
                            ),
                        ),
                )
                .on_ok(move |_, window, cx| {
                    match entity.update(cx, |this, cx| this.submit_account(cx)) {
                        Ok(message) => {
                            window.push_notification(message, cx);
                            true
                        }
                        Err(message) => {
                            window.push_notification(message, cx);
                            false
                        }
                    }
                })
        });
    }

    fn submit_account(&mut self, cx: &mut Context<Self>) -> Result<String, String> {
        let name = self.account_name.read(cx).value().trim().to_string();
        let access_key_id = self.account_key.read(cx).value().trim().to_string();
        let secret = self.account_secret.read(cx).value().trim().to_string();
        let region = self.account_region.read(cx).value().trim().to_string();

        if name.is_empty() {
            return Err(self.t(Text::ErrAccountNameRequired).to_string());
        }
        if access_key_id.is_empty() {
            return Err(self.t(Text::ErrAccessKeyRequired).to_string());
        }
        if region.is_empty() {
            return Err(self.t(Text::ErrRegionRequired).to_string());
        }

        let is_edit = self.editing_account.is_some();
        if !is_edit && secret.is_empty() {
            return Err(self.t(Text::ErrSecretRequired).to_string());
        }

        if let Some(id) = self.editing_account.clone() {
            let Some(account) = self
                .config
                .accounts
                .iter_mut()
                .find(|account| account.id == id)
            else {
                return Err(self.t(Text::ErrAccountGone).to_string());
            };
            account.name = name;
            account.access_key_id = access_key_id;
            account.region = region;
            if !secret.is_empty() {
                storage::save_secret(&id, &secret).map_err(|err| format!("{err:#}"))?;
            }
        } else {
            let mut account = Account::new(name, self.account_provider);
            account.access_key_id = access_key_id;
            account.region = region;
            storage::save_secret(&account.id, &secret).map_err(|err| format!("{err:#}"))?;
            self.config.accounts.push(account);
        }

        self.editing_account = None;
        let notice = if is_edit {
            Text::AccountUpdated
        } else {
            Text::AccountAdded
        };
        let notice_text = self.t(notice);
        self.persist();
        self.set_notice(NoticeLevel::Success, notice_text);

        Ok(self
            .t(if is_edit {
                Text::ToastAccountUpdated
            } else {
                Text::ToastAccountAdded
            })
            .to_string())
    }

    pub fn open_target_dialog(
        &mut self,
        account_id: String,
        target_id: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let language = self.language;
        let Some(account) = self.config.account(&account_id).cloned() else {
            self.set_notice(NoticeLevel::Error, self.t(Text::ErrAccountGone));
            return;
        };
        let editing = target_id
            .as_deref()
            .and_then(|id| self.config.targets.iter().find(|target| target.id == id))
            .cloned();

        self.editing_target = target_id.clone();
        self.target_account = Some(account_id.clone());

        let (name, group, region, rules) = match &editing {
            Some(target) => (
                target.name.clone(),
                target.security_group_id.clone(),
                target.region.clone(),
                target.rules.clone(),
            ),
            None => (
                String::new(),
                String::new(),
                String::new(),
                vec![
                    RuleSpec::new(Protocol::Tcp, 22, 22, ""),
                    RuleSpec::new(Protocol::Tcp, 80, 80, ""),
                    RuleSpec::new(Protocol::Tcp, 443, 443, ""),
                ],
            ),
        };

        self.set_input(&self.target_name.clone(), name, window, cx);
        self.set_input(&self.target_group.clone(), group, window, cx);
        self.set_input(&self.target_region.clone(), region, window, cx);
        self.rules_editor
            .update(cx, |editor, cx| editor.load(&rules, window, cx));

        let entity = cx.entity();
        let name_input = self.target_name.clone();
        let group_input = self.target_group.clone();
        let region_input = self.target_region.clone();
        let rules_editor = self.rules_editor.clone();
        let is_edit = target_id.is_some();
        let account_label = format!(
            "{} · {} · {}",
            account.name,
            account.provider.label(),
            account.region_or_default()
        );
        let color = provider_color(account.provider, cx);

        let title = i18n::t(
            language,
            if is_edit {
                Text::EditSecurityGroupTarget
            } else {
                Text::AddSecurityGroupTarget
            },
        );

        window.open_dialog(cx, move |dialog, _, cx| {
            let entity = entity.clone();
            dialog
                .w(px(520.))
                .title(title.to_string())
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(super::pill(account.provider.short_label(), color))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(account_label.clone()),
                                ),
                        )
                        .child(field(
                            i18n::t(language, Text::TargetName),
                            Input::new(&name_input),
                            None,
                            cx,
                        ))
                        .child(field(
                            i18n::t(language, Text::SecurityGroupId),
                            Input::new(&group_input),
                            Some(i18n::t(language, Text::SecurityGroupIdHint)),
                            cx,
                        ))
                        .child(field(
                            i18n::t(language, Text::RegionOverride),
                            Input::new(&region_input),
                            Some(i18n::t(language, Text::RegionOverrideHint)),
                            cx,
                        ))
                        .child(field(
                            i18n::t(language, Text::Rules),
                            rules_editor.clone(),
                            Some(i18n::t(language, Text::PortsHint)),
                            cx,
                        )),
                )
                .footer(
                    DialogFooter::new()
                        .gap_2()
                        .child(DialogClose::new().trigger(|button| {
                            button.outline().label(i18n::t(language, Text::Cancel))
                        }))
                        .child(DialogAction::new().child(
                            Button::new("target-save").primary().label(i18n::t(
                                language,
                                if is_edit { Text::Save } else { Text::AddTarget },
                            )),
                        )),
                )
                .on_ok(move |_, window, cx| {
                    match entity.update(cx, |this, cx| this.submit_target(cx)) {
                        Ok(message) => {
                            window.push_notification(message, cx);
                            true
                        }
                        Err(message) => {
                            window.push_notification(message, cx);
                            false
                        }
                    }
                })
        });
    }

    fn submit_target(&mut self, cx: &mut Context<Self>) -> Result<String, String> {
        let name = self.target_name.read(cx).value().trim().to_string();
        let group = self.target_group.read(cx).value().trim().to_string();
        let region = self.target_region.read(cx).value().trim().to_string();

        if name.is_empty() {
            return Err(self.t(Text::ErrTargetNameRequired).to_string());
        }
        if group.is_empty() {
            return Err(self.t(Text::ErrGroupRequired).to_string());
        }
        let rules: Vec<RuleSpec> = match self.rules_editor.read(cx).collect(cx) {
            Ok(rules) => rules,
            Err(message) => {
                self.rules_editor
                    .update(cx, |editor, cx| editor.set_error(Some(message.clone()), cx));
                return Err(message);
            }
        };
        self.rules_editor
            .update(cx, |editor, cx| editor.set_error(None, cx));

        let account_id = self
            .target_account
            .clone()
            .ok_or_else(|| self.t(Text::ErrNoAccount).to_string())?;
        if self.config.account(&account_id).is_none() {
            return Err(self.t(Text::ErrLinkedAccountGone).to_string());
        }

        let is_edit = self.editing_target.is_some();
        if let Some(id) = self.editing_target.clone() {
            let Some(target) = self
                .config
                .targets
                .iter_mut()
                .find(|target| target.id == id)
            else {
                return Err(self.t(Text::ErrTargetGone).to_string());
            };
            target.name = name;
            target.security_group_id = group;
            target.region = region;
            target.rules = rules;
        } else {
            let mut target = Target::new(account_id, name);
            target.security_group_id = group;
            target.region = region;
            target.rules = rules;
            self.config.targets.push(target);
        }

        self.editing_target = None;
        let notice = if is_edit {
            Text::TargetUpdatedNotice
        } else {
            Text::TargetAddedNotice
        };
        let notice_text = self.t(notice);
        self.persist();
        self.set_notice(NoticeLevel::Success, notice_text);

        // Preview the next sync quickly.
        let _ = self.commands.send(Command::DetectIp);
        Ok(self
            .t(if is_edit {
                Text::ToastTargetUpdated
            } else {
                Text::ToastTargetAdded
            })
            .to_string())
    }

    pub fn confirm_delete_account(
        &mut self,
        account_id: String,
        account_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let language = self.language;
        let entity = cx.entity();
        window.open_dialog(cx, move |dialog, _, cx| {
            let id = account_id.clone();
            let name = account_name.clone();
            let body = i18n::tr(language, Text::DeleteAccountBody, &[("name", &name)]);
            let deleted = i18n::t(language, Text::AccountDeleted);
            let toast = i18n::t(language, Text::ToastAccountDeleted);
            dialog
                .w(px(440.))
                .title(i18n::t(language, Text::DeleteAccountTitle))
                .child(
                    v_flex().gap_2().child(body).child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(i18n::t(language, Text::DeleteAccountSecret)),
                    ),
                )
                .footer(
                    DialogFooter::new()
                        .gap_2()
                        .child(DialogClose::new().trigger(|button| {
                            button.outline().label(i18n::t(language, Text::Cancel))
                        }))
                        .child(
                            DialogAction::new().child(
                                Button::new("delete-account-ok")
                                    .danger()
                                    .label(i18n::t(language, Text::Delete)),
                            ),
                        ),
                )
                .on_ok({
                    let entity = entity.clone();
                    move |_, window, cx| {
                        entity.update(cx, |this, _| {
                            this.config.accounts.retain(|account| account.id != id);
                            this.config.targets.retain(|target| target.account_id != id);
                            storage::delete_secret(&id);
                            this.persist();
                            this.set_notice(NoticeLevel::Success, deleted);
                        });
                        window.push_notification(toast, cx);
                        true
                    }
                })
        });
    }

    pub fn confirm_delete_target(
        &mut self,
        target_id: String,
        target_name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let language = self.language;
        let entity = cx.entity();
        window.open_dialog(cx, move |dialog, _, _| {
            let id = target_id.clone();
            let name = target_name.clone();
            let body = i18n::tr(language, Text::DeleteTargetBody, &[("name", &name)]);
            let deleted = i18n::t(language, Text::TargetDeleted);
            let toast = i18n::t(language, Text::ToastTargetDeleted);
            dialog
                .w(px(440.))
                .title(i18n::t(language, Text::DeleteTargetTitle))
                .child(body)
                .footer(
                    DialogFooter::new()
                        .gap_2()
                        .child(DialogClose::new().trigger(|button| {
                            button.outline().label(i18n::t(language, Text::Cancel))
                        }))
                        .child(
                            DialogAction::new().child(
                                Button::new("delete-target-ok")
                                    .danger()
                                    .label(i18n::t(language, Text::Delete)),
                            ),
                        ),
                )
                .on_ok({
                    let entity = entity.clone();
                    move |_, window, cx| {
                        entity.update(cx, |this, _| {
                            this.target_status.remove(&id);
                            this.config.targets.retain(|target| target.id != id);
                            this.persist();
                            this.set_notice(NoticeLevel::Success, deleted);
                        });
                        window.push_notification(toast, cx);
                        true
                    }
                })
        });
    }
}
