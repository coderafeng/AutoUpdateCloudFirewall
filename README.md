<p align="center">
  <img src="assets/icon-128.png" width="112" alt="Auto Update Cloud Firewall logo">
</p>

<h1 align="center">Auto Update Cloud Firewall</h1>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README.zh-CN.md">简体中文</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.ko.md">한국어</a>
</p>

A small desktop app that keeps cloud security-group ingress rules pointed at your
current public IPv4 address. It runs quietly in the notification area, re-checks
your IP on an interval, and updates only the rules it owns — everything else in
the security group is left untouched.

Built with Rust and [gpui-kit](https://gpui-kit.com/docs/getting-started/).

## Supported clouds

| Provider | API | Managed operations |
| --- | --- | --- |
| Aliyun ECS | RPC, `ecs.aliyuncs.com`, `2014-05-26` | `DescribeSecurityGroups`, `DescribeSecurityGroupAttribute`, `AuthorizeSecurityGroup`, `RevokeSecurityGroup`, `ModifySecurityGroupRule` |
| Tencent Cloud CVM | TC3-HMAC-SHA256, `ecs.tencentcloudapi.com`, `2017-03-12` | `DescribeSecurityGroups`, `DescribeSecurityGroupPolicies`, `CreateSecurityGroupPolicies`, `DeleteSecurityGroupPolicies` |

No cloud vendor SDK is used; both request signers are implemented directly.

## Features

- **Multiple accounts and security groups.** Add any number of Aliyun or
  Tencent Cloud accounts, each with its own AccessKey and region, then attach one
  or more security-group targets to an account.
- **Automatic public IP detection.** Several HTTP lookups are tried in order
  (ipify, ifconfig.me, ipinfo.io, ipip.net by default). Private, loopback,
  link-local, CGNAT, multicast and reserved IPv4 ranges are rejected.
- **Safe rule ownership.** Only ingress rules whose description starts with the
  configured prefix (`AutoFirewall` by default) are created, changed or removed.
  Manually created rules are never modified.
- **Dry-run mode.** Logs the planned changes without calling the write APIs.
- **Background operation.** Closing the window keeps the app in the tray; the
  worker keeps syncing while the window is hidden. Optional auto-start with
  Windows.
- **Activity log.** The last 500 entries are shown in the UI, and the full log is
  appended to disk.
- **Four interface languages.** English, Simplified Chinese, Japanese and Korean.
  The first launch follows the Windows display language; the language can be
  switched at any time on **Settings → Language** and applies to the UI, dialogs,
  tray menu and new log messages.

## Requirements

- Windows 10/11 (the tray, credential store and auto-start integration use Win32).
- Rust 1.85+ (edition 2024) and the MSVC toolchain.
- An AccessKey with permission to read and modify security groups:

  **Aliyun** — `ecs:DescribeSecurityGroups`, `ecs:DescribeSecurityGroupAttribute`,
  `ecs:AuthorizeSecurityGroup`, `ecs:RevokeSecurityGroup`,
  `ecs:ModifySecurityGroupRule`.

  **Tencent Cloud** — `DescribeSecurityGroups`, `DescribeSecurityGroupPolicies`,
  `CreateSecurityGroupPolicies`, `DeleteSecurityGroupPolicies`.

Use a dedicated RAM/CAM user for the app instead of a root account.

## Build and run

```powershell
cargo run --release
```

A debug build (`cargo run`) keeps a console window attached for diagnostics; a
release build is GUI-only.

Run the tests with:

```powershell
cargo test
```

## Releasing

```powershell
pwsh tools/release.ps1 -Version 1.0.0           # build and rename per platform
pwsh tools/release.ps1 -Version 1.0.0 -Publish  # also create the GitHub release
```

Artifacts land in `dist/` with a platform suffix:
`auto-update-cloud-firewall-x86_64-windows.exe`,
`auto-update-cloud-firewall-x86_64-darwin.exe`,
`auto-update-cloud-firewall-x86_64-linux.exe`.
The script runs `cargo fmt --check`, clippy and the test suite first, and takes the
release notes from `docs/releases/<version>.md` (four languages). Each platform
binary has to be built on that platform or in CI.

Releases go through CI: pushing a `v*` tag runs
`.github/workflows/release.yml`, which builds on a Windows, a macOS and a Linux
runner and attaches all three assets plus `docs/releases/<version>.md` to the
release. The macOS and Linux assets are labelled **unverified** in the notes.

See [CHANGELOG.md](CHANGELOG.md) for the version history.

## Using the app

1. Open **Cloud accounts** and add an account with **Aliyun account** or
   **Tencent account**. The AccessKey secret is written to the Windows Credential
   Manager under the service `AutoUpdateCloudFirewall`; it is never written to
   `config.json`.
2. Add a **Security group** target to the account: a name, the security-group id,
   an optional region override, and one row per ingress rule (protocol, port or
   range such as `8000-8010`, and an optional description keyword).
3. Leave the app in the tray. It checks the public IP every
   `check_interval_secs` (300 by default, minimum 60) and syncs when the address
   changes. Use **Sync now** to force a run, **Detect IP** to refresh the address
   immediately.
4. Tray menu: **Show window**, **Hide window**, **Sync now**,
   **Open config folder**, **Quit**. Double-clicking the icon shows the window.
5. On **Settings → Language**, pick English, 简体中文, 日本語 or 한국어. The choice
   is saved in `settings.language` (`en`, `zh-CN`, `ja`, `ko`).

## How a sync works

For each enabled target:

1. Read the current ingress rules from the cloud API.
2. Build the desired rule set from the target's rules and the detected public IP.
   Every desired rule carries a description; rows with a keyword also record it.
3. **Delete first**: stale managed rules (description starting with `AutoFirewall`)
   that no longer match are removed one at a time, which also keeps Tencent Cloud
   policy indices valid. This has to happen before the update step — Aliyun
   refuses to turn a rule into an exact copy of another one
   (`InvalidOperation.RuleDuplicate`), and the stale rule usually is the one
   holding the new address.
4. When a row has a **description keyword**, the rule whose protocol and port
   match and whose description contains the keyword is adopted: its source IP is
   replaced in place (Aliyun `ModifySecurityGroupRule`) with its description and
   priority untouched. On Tencent Cloud, which has no per-rule update call, the
   rule is revoked and re-created with the same description. If the keyword
   matches several rules, the whole target is skipped with an error instead of
   guessing.
5. Missing rules are created last; rows without a keyword keep the prefix-based
   ownership.
6. Re-read the group and verify that no rule remains in the plan.

Rules you created yourself are never deleted, even after the matching row is
removed from the configuration.

Write calls retry up to three times with a short backoff; errors are logged and
reported in the UI without stopping the background loop.

## Files

| Path | Purpose |
| --- | --- |
| `%APPDATA%\AutoUpdateCloudFirewall\config.json` | Accounts, targets and settings. Contains no secrets. |
| `%LOCALAPPDATA%\AutoUpdateCloudFirewall\logs\app.log` | Append-only activity log. |
| Windows Credential Manager, service `AutoUpdateCloudFirewall` | AccessKey secrets, one entry per account id. |

`config.json.example` documents the on-disk format. The GUI is the recommended
way to edit it; a broken file is backed up as `config.json.broken` and replaced
with defaults on the next start.

## Project layout

| File | Responsibility |
| --- | --- |
| `src/main.rs` | Runtime, channels, tray and window bootstrap. |
| `src/i18n.rs` | English / Simplified Chinese / Japanese / Korean strings. |
| `src/model.rs` | Account/target/settings types, port parsing and formatting. |
| `src/storage.rs` | Config read/write and credential-store access. |
| `src/ip.rs` | Public IPv4 detection and validation. |
| `src/planner.rs` | Desired-vs-remote rule diff and managed-rule selection. |
| `src/providers/` | Aliyun RPC and Tencent TC3 clients and signing. |
| `src/worker.rs` | Background loop, retries, verification, account test. |
| `src/platform.rs` | Show/hide the native window, Windows auto-start. |
| `src/tray.rs` | Notification-area icon and menu. |
| `src/ui/` | Sidebar shell, pages and account/target dialogs. |
| `assets/` | Icon sources (`logo.svg`, `logo-compact.svg`, `logo-mono.svg`, `logo-ui.svg`) and the generated PNG/ICO files. |
| `tools/generate-icons.py` | Regenerates the icon set from the SVG sources. |
| `build.rs` | Embeds the executable icon and version resources. |

## Limitations

- IPv4 only.
- Ingress rules only.
- Rules are matched by the description prefix; do not edit the prefix on rules
  created by hand if you want the app to leave them alone.

## Links

- Official site: [rustsilk.com](https://rustsilk.com)

## License

Released under the [MIT License](LICENSE).

Copyright (c) 2026 Auto Update Cloud Firewall contributors.
