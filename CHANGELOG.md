# Changelog

All notable changes to this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

English · [简体中文](CHANGELOG.zh-CN.md) · [日本語](CHANGELOG.ja.md) · [한국어](CHANGELOG.ko.md)

## [Unreleased]

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

[Unreleased]: https://github.com/coderafeng/AutoUpdateCloudFirewall/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/coderafeng/AutoUpdateCloudFirewall/releases/tag/v1.0.0
