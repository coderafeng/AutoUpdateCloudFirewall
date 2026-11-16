# 更新日志

本文件记录项目的所有重要变更。

格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)，版本号遵循
[语义化版本](https://semver.org/lang/zh-CN/)。

[English](CHANGELOG.md) · 简体中文 · [日本語](CHANGELOG.ja.md) · [한국어](CHANGELOG.ko.md)

## [未发布]

## [1.0.0] - 2026-11-16

首个稳定版本。

### 新增

- 阿里云 ECS 与腾讯云 CVM 云账号，每个账号持有独立的 AccessKey、地域和启用开关，
  一个账号可驱动多个安全组目标。
- 每个目标的入站规则行：协议、单端口或端口范围
  （`5432`、`8000-8010`、`8000/8010`）以及可选的备注关键字。
- 按备注关键字接管已有规则：填了关键字的规则行会接管协议、端口都相同且备注包含该
  关键字的规则，仅改写源地址（阿里云 `ModifySecurityGroupRule`），备注与优先级原样
  保留，不会产生重复规则。
- 歧义防护：关键字命中多条规则时，整个目标报错跳过，不做任何猜测。
- 没填关键字的行按描述前缀归属（默认 `AutoFirewall`），手动创建的规则永远不会被修改
  或删除。
- 演练模式：只把计划写进日志，不调用任何写接口。
- 托盘集成：关闭窗口后后台任务继续运行，支持随系统启动与最小化启动（Windows）。
- 活动日志：界面显示最近 500 条，完整记录追加写入
  `%LOCALAPPDATA%\AutoUpdateCloudFirewall\logs\app.log`。
- 四种界面语言——英语、简体中文、日语、韩语，切换后立即作用于窗口、对话框、托盘菜单
  与新增日志。
- 密钥保存在系统凭据存储中，`config.json` 只记录 AccessKey ID。
- 发布产物按平台命名：`auto-update-cloud-firewall-x86_64-{windows,darwin,linux}.exe`。

### 变更

- 同步按 **删除 → 更新 → 新建** 的顺序执行。阿里云会拒绝把规则改成与另一条完全重复
  （`InvalidOperation.RuleDuplicate`），因此先移除过期规则，再把地址写到被接管的规则上。
- 云厂商错误改为报出被拒绝的 RAM 动作与 RequestId，而不是被截断的 JSON。
- 同步间隔校验为最小 60 秒，默认 300 秒。
- 侧边栏底部与设置页展示仓库链接与 MIT 协议。

### 修复

- 过长的日志行不再溢出卡片边框，超出部分用省略号截断。
- 窗口左下角与可执行文件版本资源统一为 1.0.0。

### 已知限制

- 1.0.0 的受支持平台为 Windows 10/11；macOS 与 Linux 使用同一份源码构建，但未做同等
  验证。
- 仅处理 IPv4 与入站规则。
- 腾讯云没有单条规则修改接口，改源 IP 等于「按原备注删除 + 重建」：规则 ID 会变，
  备注与优先级保留。
- 只能写入当前公网地址，把规则放宽回 `0.0.0.0/0` 需要到控制台手工修改。

[未发布]: https://github.com/coderafeng/AutoUpdateCloudFirewall/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/coderafeng/AutoUpdateCloudFirewall/releases/tag/v1.0.0
