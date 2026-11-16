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

一个轻量的桌面应用，让云安全组的入站规则始终指向你当前的公网 IPv4 地址。它常驻通知区域静默运行，按设定间隔重新检测 IP，并且只更新由它自己管理的规则——安全组中的其他内容不会被改动。

使用 Rust 和 [gpui-kit](https://gpui-kit.com/docs/getting-started/) 构建。

## 支持的云厂商

| 厂商 | API | 使用的接口 |
| --- | --- | --- |
| 阿里云 ECS | RPC, `ecs.aliyuncs.com`, `2014-05-26` | `DescribeSecurityGroups`、`DescribeSecurityGroupAttribute`、`AuthorizeSecurityGroup`、`RevokeSecurityGroup`、`ModifySecurityGroupRule` |
| 腾讯云 CVM | TC3-HMAC-SHA256, `ecs.tencentcloudapi.com`, `2017-03-12` | `DescribeSecurityGroups`、`DescribeSecurityGroupPolicies`、`CreateSecurityGroupPolicies`、`DeleteSecurityGroupPolicies` |

未使用任何云厂商 SDK，两套请求签名均为直接实现。

## 功能

- **多账号、多安全组。** 可以添加任意数量的阿里云或腾讯云账号，每个账号拥有自己的 AccessKey 和地域，然后为一个账号绑定一个或多个安全组目标。
- **自动检测公网 IP。** 依次尝试多个 HTTP 查询服务（默认 ipify、ifconfig.me、ipinfo.io、ipip.net）。私有、环回、链路本地、CGNAT、组播和保留 IPv4 地址段会被拒绝。
- **安全的规则归属。** 只有描述以配置前缀开头的入站规则（默认 `AutoFirewall`）才会被创建、修改或删除。手动创建的规则永远不会被改动。
- **演练模式。** 只记录计划中的变更，不调用写入类 API。
- **后台运行。** 关闭窗口后应用仍留在托盘，窗口隐藏时后台任务继续同步。可选择随 Windows 开机自启。
- **活动日志。** 界面中显示最近 500 条记录，完整日志会追加写入磁盘。
- **四种界面语言。** 英语、简体中文、日语和韩语。首次启动跟随 Windows 显示语言；之后可随时在 **设置 → 语言** 中切换，立即作用于界面、对话框、托盘菜单和新的日志消息。

## 环境要求

- Windows 10/11（托盘、凭据存储和开机自启集成使用 Win32）。
- Rust 1.85+（edition 2024）和 MSVC 工具链。
- 一个拥有安全组读写权限的 AccessKey：

  **阿里云** — `ecs:DescribeSecurityGroups`、`ecs:DescribeSecurityGroupAttribute`、`ecs:AuthorizeSecurityGroup`、`ecs:RevokeSecurityGroup`、`ecs:ModifySecurityGroupRule`。

  **腾讯云** — `DescribeSecurityGroups`、`DescribeSecurityGroupPolicies`、`CreateSecurityGroupPolicies`、`DeleteSecurityGroupPolicies`。

请为应用使用专用的 RAM/CAM 子用户，而不是主账号。

## 构建与运行

```powershell
cargo run --release
```

调试构建（`cargo run`）会附带一个控制台窗口便于排查问题；发布构建则只有图形界面。

运行测试：

```powershell
cargo test
```

## 发布

```powershell
pwsh tools/release.ps1 -Version 1.0.0           # 构建并按平台命名
pwsh tools/release.ps1 -Version 1.0.0 -Publish  # 同时创建 GitHub Release
```

产物固定在 `dist/`，文件名带平台后缀：
`auto-update-cloud-firewall-x86_64-windows.exe`、
`auto-update-cloud-firewall-x86_64-darwin.exe`、
`auto-update-cloud-firewall-x86_64-linux.exe`。
脚本会在构建前跑完 `cargo fmt --check`、clippy 和测试，发布说明取自
`docs/releases/<版本>.md`（四语）。每个平台的二进制需要在对应系统（或 CI）上构建。

正式发布走 CI：推送 `v*` tag 会触发 `.github/workflows/release.yml`，在 Windows、macOS、
Linux 三个 runner 上分别构建，把三份产物连同 `docs/releases/<版本>.md` 一起挂到 Release。
其中 macOS 与 Linux 的产物在发布说明中标为**未验证**。

版本变更记录见 [CHANGELOG.zh-CN.md](CHANGELOG.zh-CN.md)。

## 使用方式

1. 打开 **云账号**，通过 **阿里云账号** 或 **腾讯云账号** 添加账号。AccessKey 密钥会写入 Windows 凭据管理器，服务名为 `AutoUpdateCloudFirewall`，永远不会写入 `config.json`。
2. 为该账号添加一个 **安全组** 目标：名称、安全组 ID、可选的地域覆盖，以及每条入站规则一行（协议、端口或范围如 `8000-8010`、可选的备注关键字）。
3. 让应用留在托盘即可。它会每隔 `check_interval_secs`（默认 300，最小 60）检测一次公网 IP，并在地址变化时进行同步。使用 **立即同步** 强制运行，使用 **检测 IP** 立即刷新地址。
4. 托盘菜单：**显示窗口**、**隐藏窗口**、**立即同步**、**打开配置文件夹**、**退出**。双击图标可显示窗口。
5. 在 **设置 → 语言** 中选择 English、简体中文、日本語 或 한국어。该选择保存在 `settings.language` 中（`en`、`zh-CN`、`ja`、`ko`）。

## 同步流程

对每个启用的目标：

1. 从云 API 读取当前的入站规则。
2. 根据目标的规则和检测到的公网 IP 构建期望规则集合，每条都带描述；填了关键字的规则还会记录关键字。
3. **先删除**描述以 `AutoFirewall` 开头、已不再匹配的过期规则（逐条进行，腾讯云的规则索引因此保持有效）。必须先删：阿里云不允许把一条规则改成与另一条规则完全重复（`InvalidOperation.RuleDuplicate`），而过期的旧规则往往正占着新 IP 那一条。
4. 规则行填了 **备注关键字** 时：在安全组里找到协议和端口都相同、备注包含该关键字的那条规则，**就地替换它的源 IP**（阿里云使用 `ModifySecurityGroupRule`），备注和优先级保持不变；腾讯云没有单条修改接口，改为按原备注删除后重建。若关键字命中多条规则，整个目标直接报错跳过，不做任何猜测。
5. 最后创建缺失的规则，没填关键字的行仍按 `AutoFirewall` 前缀归属。
6. 重新读取安全组，确认计划中的规则已全部落地。

你自己手工创建的规则永远不会被删除，即使对应的配置行已被移除。

写入调用最多重试三次，并带有短暂退避；错误会被记录并在界面中报告，不会中断后台循环。

## 文件位置

| 路径 | 用途 |
| --- | --- |
| `%APPDATA%\AutoUpdateCloudFirewall\config.json` | 账号、目标和设置。不包含任何密钥。 |
| `%LOCALAPPDATA%\AutoUpdateCloudFirewall\logs\app.log` | 只追加的活动日志。 |
| Windows 凭据管理器，服务 `AutoUpdateCloudFirewall` | AccessKey 密钥，每个账号 ID 一条记录。 |

`config.json.example` 记录了磁盘上的文件格式。建议通过图形界面编辑；文件损坏时会被备份为 `config.json.broken`，并在下次启动时替换为默认配置。

## 项目结构

| 文件 | 职责 |
| --- | --- |
| `src/main.rs` | 运行时、通道、托盘和窗口引导。 |
| `src/i18n.rs` | 英语 / 简体中文 / 日语 / 韩语文案。 |
| `src/model.rs` | 账号、目标、设置类型，端口解析与格式化。 |
| `src/storage.rs` | 配置读写与凭据存储访问。 |
| `src/ip.rs` | 公网 IPv4 检测与校验。 |
| `src/planner.rs` | 期望与远端规则的差异比较和受管规则筛选。 |
| `src/providers/` | 阿里云 RPC 与腾讯云 TC3 客户端及签名。 |
| `src/worker.rs` | 后台循环、重试、校验和账号连通性测试。 |
| `src/platform.rs` | 显示/隐藏原生窗口，Windows 开机自启。 |
| `src/tray.rs` | 通知区域图标与菜单。 |
| `src/ui/` | 侧边栏外壳、页面以及账号/目标对话框。 |
| `assets/` | 图标源文件（`logo.svg`、`logo-compact.svg`、`logo-mono.svg`、`logo-ui.svg`）及生成的 PNG/ICO。 |
| `tools/generate-icons.py` | 从 SVG 源重新生成整套图标。 |
| `build.rs` | 嵌入可执行文件图标和版本资源。 |

## 限制

- 仅支持 IPv4。
- 仅处理入站规则。
- 规则按描述前缀匹配；若希望应用不去改动手动创建的规则，请不要修改其前缀。

## 相关链接

- 官方网站：[rustsilk.com](https://rustsilk.com)

## 许可协议

本项目基于 [MIT 协议](LICENSE) 开源。

Copyright (c) 2026 Auto Update Cloud Firewall contributors.
