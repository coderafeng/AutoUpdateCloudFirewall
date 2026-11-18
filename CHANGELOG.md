# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

English · [简体中文](CHANGELOG.zh-CN.md) · [日本語](CHANGELOG.ja.md) · [한국어](CHANGELOG.ko.md)

## [Unreleased]

## [1.1.0] - 2026-11-18

Scheduling moved from a single global countdown to per-target due times, and the
dead Edit button on pages with several targets is fixed.

### Added

- Cron expressions: besides the fixed interval, the settings page accepts a
  five-field expression (minute hour day month weekday) — `0 */5 * * *` for
  every five minutes, `0 3 * * *` for daily at 03:00 — as well as `@hourly`,
  `@daily`, `@weekly`, `@monthly` and `@yearly`. Leaving it empty keeps the
  interval.
- Per-target schedule overrides: a target may carry its own interval in seconds
  or its own cron expression. An empty field inherits the global setting, and
  the expression wins when both are filled. The dialog shows an example and the
  global default that is currently inherited.
- Target cards show the schedule actually in effect (`every 300s`,
  `cron 0 3 * * *`), marked as global when it is inherited.

### Changed

- The worker computes a due time per target, so targets can run on different
  cadences. A scheduled run only touches the targets that are due; "Sync now"
  still covers every target.

### Fixed

- With more than one security-group target, the Edit button did nothing for
  every target after the first. Every row reused the same element IDs
  (`edit-target`, `delete-target`, `target-enabled`, and likewise for accounts),
  so gpui attached the handlers to the first row only. The IDs now include the
  target or account id.

### Known limitations

- Cron is interpreted in the local time zone with minute granularity. A bad
  expression is rejected when saving; editing `config.json` by hand falls back
  to the global interval.
- The other 1.0.0 limitations still apply: IPv4 ingress only; Tencent Cloud
  rewrites a source address by deleting and recreating the rule; widening a rule
  back to `0.0.0.0/0` still needs the console.

## [1.0.0] - 2026-11-16

First stable release.

### Added

- Cloud accounts for Aliyun ECS and Tencent Cloud CVM, each with its own
  AccessKey, region and enabled flag; one account can drive several
  security-group targets.
- Per-target ingress rule rows: protocol, a single port or a range
  (`5432`, `8000-8010`, `8000/8010`) and an optional description keyword.
- In-place rule adoption by description keyword. A row with a keyword takes over
  the rule whose protocol and port match and whose description contains it, and
  only the source address is rewritten (Aliyun `ModifySecurityGroupRule`); the
  description and priority survive and no duplicate rule is created.
- Ambiguity guard: a keyword matching more than one rule skips the whole target
  with an error instead of guessing.
- Prefix-based ownership for rows without a keyword (default `AutoFirewall`);
  rules you created by hand are never modified or deleted.
- Dry-run mode that logs the plan without calling any write API.
- Tray integration: closing the window keeps the worker running, with optional
  start-with-system and start-minimized (Windows).
- Activity log: the newest 500 entries in the UI, the full stream appended to
  `%LOCALAPPDATA%\AutoUpdateCloudFirewall\logs\app.log`.
- Four UI languages — English, Simplified Chinese, Japanese, Korean — switching
  instantly across the window, dialogs, tray menu and new log lines.
- Secrets in the OS credential store; `config.json` only holds the AccessKey ID.
- Release binaries are published per platform as
  `auto-update-cloud-firewall-x86_64-windows.exe`,
  `auto-update-cloud-firewall-x86_64-darwin` and
  `auto-update-cloud-firewall-x86_64-linux` — only Windows carries `.exe`.
- Linux is additionally published as
  `auto-update-cloud-firewall-x86_64-linux.tar.gz`, which keeps the executable
  bit and bundles `LICENSE` and `README.md`.

### Changed

- Sync writes are applied as **delete → update → create**. Aliyun rejects a
  modify that would make a rule identical to another one
  (`InvalidOperation.RuleDuplicate`), so stale rules are removed before an
  address is written onto an adopted rule.
- Provider errors report the denied RAM action and the request id instead of a
  truncated JSON body.
- The sync interval validates to a minimum of 60 seconds, 300 by default.
- The sidebar footer and the settings page show the repository link and the MIT
  license.

### Fixed

- Long log lines no longer overflow their card; they are truncated with an
  ellipsis.
- Version metadata is 1.0.0 in the window footer and in the executable
  resources.

### Known limitations

- Windows 10/11 is the supported platform for 1.0.0. The macOS and Linux binaries
  are published anyway: they are built in CI from the same source but have never
  been launched on a real machine, so the tray, the OS credential store and
  window rendering may all fail there.
- IPv4 and ingress rules only.
- Tencent Cloud has no per-rule update call, so changing an address revokes and
  re-creates the rule: the rule ID changes, description and priority are kept.
- Only the current public address can be written; widening a rule back to
  `0.0.0.0/0` must be done in the cloud console.

[Unreleased]: https://github.com/coderafeng/AutoUpdateCloudFirewall/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/coderafeng/AutoUpdateCloudFirewall/releases/tag/v1.1.0
[1.0.0]: https://github.com/coderafeng/AutoUpdateCloudFirewall/releases/tag/v1.0.0
