//! The rule editor used by the security-group target dialog.
//!
//! One row per ingress rule: protocol, port (or range) and an optional
//! description keyword. The keyword names an existing rule in the security
//! group so synchronisation updates that rule in place instead of adding a
//! second one with the same port.

use gpui_kit::component::{
    ActiveTheme, IconName, Sizable as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::i18n::{self, Text};
use crate::model::{Language, Protocol, RuleSpec, parse_port_field};

struct RuleRow {
    protocol: Protocol,
    port: Entity<InputState>,
    keyword: Entity<InputState>,
}

pub struct RulesEditor {
    rows: Vec<RuleRow>,
    language: Language,
    error: Option<String>,
}

impl RulesEditor {
    /// A fresh editor with the three rules new targets start with.
    pub fn with_defaults(language: Language, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut editor = Self {
            rows: Vec::new(),
            language,
            error: None,
        };
        let defaults = [
            (Protocol::Tcp, "22"),
            (Protocol::Tcp, "80"),
            (Protocol::Tcp, "443"),
        ];
        editor.rows = defaults
            .into_iter()
            .map(|(protocol, port)| editor.make_row(protocol, port, "", window, cx))
            .collect();
        editor
    }

    /// Replace the rows with the rules of the target being edited.
    pub fn load(&mut self, rules: &[RuleSpec], window: &mut Window, cx: &mut Context<Self>) {
        self.rows = rules
            .iter()
            .map(|rule| {
                self.make_row(
                    rule.protocol,
                    &rule.port_text(),
                    rule.keyword.trim(),
                    window,
                    cx,
                )
            })
            .collect();
        self.error = None;
        cx.notify();
    }

    /// Validate the rows and turn them into configuration values.
    pub fn collect(&self, cx: &App) -> Result<Vec<RuleSpec>, String> {
        if self.rows.is_empty() {
            return Err(i18n::t(self.language, Text::ErrPortListEmpty).to_string());
        }

        let mut rules: Vec<RuleSpec> = Vec::with_capacity(self.rows.len());
        for row in &self.rows {
            let port = row.port.read(cx).value().trim().to_string();
            if port.is_empty() {
                return Err(i18n::t(self.language, Text::ErrPortRequired).to_string());
            }
            let (start, end) = parse_port_field(&port).map_err(|err| err.message(self.language))?;
            let keyword = row.keyword.read(cx).value().trim().to_string();
            let spec = RuleSpec::new(row.protocol, start, end, keyword);
            if rules.iter().any(|rule| rule.same_spec(&spec)) {
                return Err(i18n::tr(
                    self.language,
                    Text::ErrDuplicateRule,
                    &[("port", &spec.port_text())],
                ));
            }
            rules.push(spec);
        }
        Ok(rules)
    }

    pub fn set_error(&mut self, error: Option<String>, cx: &mut Context<Self>) {
        self.error = error;
        cx.notify();
    }

    fn make_row(
        &self,
        protocol: Protocol,
        port: &str,
        keyword: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> RuleRow {
        let language = self.language;
        RuleRow {
            protocol,
            port: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("22")
                    .default_value(port.to_string())
            }),
            keyword: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(i18n::t(language, Text::RuleKeywordPlaceholder))
                    .default_value(keyword.to_string())
            }),
        }
    }

    fn add_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let row = self.make_row(Protocol::Tcp, "", "", window, cx);
        self.rows.push(row);
        self.error = None;
        cx.notify();
    }

    fn remove_row(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.rows.len() {
            self.rows.remove(index);
        }
        self.error = None;
        cx.notify();
    }

    fn toggle_protocol(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.get_mut(index) {
            row.protocol = match row.protocol {
                Protocol::Tcp => Protocol::Udp,
                Protocol::Udp => Protocol::Tcp,
            };
        }
        self.error = None;
        cx.notify();
    }
}

impl Render for RulesEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let language = self.language;
        let muted = cx.theme().muted_foreground;
        let danger = cx.theme().danger;

        let mut rows: Vec<Div> = Vec::with_capacity(self.rows.len());
        for (index, row) in self.rows.iter().enumerate() {
            let protocol = row.protocol;
            let port_input = row.port.clone();
            let keyword_input = row.keyword.clone();

            rows.push(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Button::new(("rule-protocol", index))
                            .outline()
                            .small()
                            .label(protocol.label())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_protocol(index, cx);
                            })),
                    )
                    .child(Input::new(&port_input).w(px(110.)))
                    .child(Input::new(&keyword_input).flex_1())
                    .child(
                        Button::new(("rule-remove", index))
                            .ghost()
                            .small()
                            .icon(IconName::CircleX)
                            .tooltip(i18n::t(language, Text::RemoveRule))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_row(index, cx);
                            })),
                    ),
            );
        }

        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        div()
                            .w(px(64.))
                            .text_xs()
                            .text_color(muted)
                            .child(i18n::t(language, Text::RuleProtocol)),
                    )
                    .child(
                        div()
                            .w(px(110.))
                            .text_xs()
                            .text_color(muted)
                            .child(i18n::t(language, Text::RulePort)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .text_color(muted)
                            .child(i18n::t(language, Text::RuleKeyword)),
                    )
                    .child(div().w(px(28.))),
            )
            .children(rows)
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("rule-add")
                            .outline()
                            .small()
                            .icon(IconName::Plus)
                            .label(i18n::t(language, Text::AddRule))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_row(window, cx);
                            })),
                    )
                    .when_some(self.error.clone(), |this, error| {
                        this.child(div().text_xs().text_color(danger).child(error))
                    }),
            )
    }
}
