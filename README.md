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
| Aliyun ECS | RPC, `ecs.aliyuncs.com`, `2014-05-26` | `DescribeSecurityGroups`, `DescribeSecurityGroupAttribute`, `AuthorizeSecurityGroup`, `RevokeSecurityGroup` |
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
  `ecs:AuthorizeSecurityGroup`, `ecs:RevokeSecurityGroup`.

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

## Using the app

1. Open **Cloud accounts** and add an account with **Aliyun account** or
   **Tencent account**. The AccessKey secret is written to the Windows Credential
   Manager under the service `AutoUpdateCloudFirewall`; it is never written to
   `config.json`.
2. Add a **Security group** target to the account: a name, the security-group id,
   an optional region override, and the ports to allow
   (`tcp:22,80,443; udp:53`, ranges as `8000-8010`).
3. Leave the app in the tray. It checks the public IP every
   `check_interval_secs` (300 by default, minimum 30) and syncs when the address
   changes. Use **Sync now** to force a run, **Detect IP** to refresh the address
   immediately.
4. Tray menu: **Show window**, **Hide window**, **Sync now**,
   **Open config folder**, **Quit**. Double-clicking the icon shows the window.
5. On **Settings → Language**, pick English, 简体中文, 日本語 or 한국어. The choice
   is saved in `settings.language` (`en`, `zh-CN`, `ja`, `ko`).

## How a sync works

For each enabled target:

1. Read the current ingress rules from the cloud API.
2. Build the desired rule set from the target's ports and the detected public IP.
   Each desired rule carries the description `AutoFirewall <ip> <protocol> <port>`.
3. Diff the two sets using the managed-rule prefix. The diff deletes stale or
   changed managed rules, one at a time (Tencent Cloud rule indices stay valid),
   then creates the missing ones.
4. Re-read the group and verify that no managed rules remain in the plan.

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
