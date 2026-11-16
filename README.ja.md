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

クラウドのセキュリティグループのインバウンドルールを、常に現在のパブリック IPv4 アドレスに向けておく小さなデスクトップアプリです。通知領域で静かに動作し、一定間隔で IP を再確認し、自分が管理するルールだけを更新します。セキュリティグループ内のそれ以外の設定には一切触れません。

Rust と [gpui-kit](https://gpui-kit.com/docs/getting-started/) で構築しています。

## 対応クラウド

| プロバイダー | API | 使用する操作 |
| --- | --- | --- |
| Aliyun ECS | RPC, `ecs.aliyuncs.com`, `2014-05-26` | `DescribeSecurityGroups`, `DescribeSecurityGroupAttribute`, `AuthorizeSecurityGroup`, `RevokeSecurityGroup` |
| Tencent Cloud CVM | TC3-HMAC-SHA256, `ecs.tencentcloudapi.com`, `2017-03-12` | `DescribeSecurityGroups`, `DescribeSecurityGroupPolicies`, `CreateSecurityGroupPolicies`, `DeleteSecurityGroupPolicies` |

クラウドベンダーの SDK は使用しておらず、両方のリクエスト署名を直接実装しています。

## 機能

- **複数アカウントとセキュリティグループ。** Aliyun または Tencent Cloud のアカウントをいくつでも追加でき、それぞれに独自の AccessKey とリージョンを持たせたうえで、1 つのアカウントに 1 つ以上のセキュリティグループ対象を紐付けられます。
- **パブリック IP の自動検出。** 複数の HTTP ルックアップを順に試します（既定では ipify、ifconfig.me、ipinfo.io、ipip.net）。プライベート、ループバック、リンクローカル、CGNAT、マルチキャスト、予約済みの IPv4 範囲は拒否されます。
- **安全なルールの所有権。** 説明が設定したプレフィックス（既定は `AutoFirewall`）で始まるインバウンドルールだけが作成・変更・削除されます。手動で作成したルールは決して変更されません。
- **ドライランモード。** 書き込み API を呼ばずに、予定されている変更だけをログに記録します。
- **バックグラウンド動作。** ウィンドウを閉じてもアプリはトレイに残り、非表示の間もワーカーが同期を続けます。Windows 起動時の自動開始も任意で設定できます。
- **アクティビティログ。** 直近 500 件を UI に表示し、完全なログはディスクに追記されます。
- **4 つの表示言語。** 英語、簡体字中国語、日本語、韓国語。初回起動時は Windows の表示言語に従い、その後は **設定 → 言語** でいつでも切り替えられます。UI、ダイアログ、トレイメニュー、新しいログメッセージに即座に反映されます。

## 動作要件

- Windows 10/11（トレイ、資格情報ストア、自動開始の連携に Win32 を使用）。
- Rust 1.85 以降（edition 2024）と MSVC ツールチェーン。
- セキュリティグループの読み取りと変更が可能な AccessKey：

  **Aliyun** — `ecs:DescribeSecurityGroups`、`ecs:DescribeSecurityGroupAttribute`、`ecs:AuthorizeSecurityGroup`、`ecs:RevokeSecurityGroup`。

  **Tencent Cloud** — `DescribeSecurityGroups`、`DescribeSecurityGroupPolicies`、`CreateSecurityGroupPolicies`、`DeleteSecurityGroupPolicies`。

ルートアカウントではなく、アプリ専用の RAM/CAM ユーザーを使用してください。

## ビルドと実行

```powershell
cargo run --release
```

デバッグビルド（`cargo run`）では診断用のコンソールウィンドウが表示されます。リリースビルドは GUI のみです。

テストの実行：

```powershell
cargo test
```

## アプリの使い方

1. **クラウドアカウント** を開き、**Aliyun アカウント** または **Tencent Cloud アカウント** でアカウントを追加します。AccessKey のシークレットはサービス名 `AutoUpdateCloudFirewall` として Windows 資格情報マネージャーに保存され、`config.json` に書き込まれることはありません。
2. アカウントに **セキュリティグループ** 対象を追加します。名前、セキュリティグループ ID、任意のリージョン上書き、そしてインバウンドルール 1 件につき 1 行（プロトコル、ポートまたは `8000-8010` のような範囲、任意の説明キーワード）を指定します。
3. アプリはトレイに置いたままで構いません。`check_interval_secs`（既定 300、最小 30）ごとにパブリック IP を確認し、アドレスが変わったときに同期します。**今すぐ同期** で強制的に実行し、**IP を検出** でアドレスを即座に更新できます。
4. トレイメニュー：**ウィンドウを表示**、**ウィンドウを隠す**、**今すぐ同期**、**設定フォルダーを開く**、**終了**。アイコンをダブルクリックするとウィンドウが表示されます。
5. **設定 → 言語** で English、简体中文、日本語、한국어 から選択できます。選択内容は `settings.language`（`en`、`zh-CN`、`ja`、`ko`）に保存されます。

## 同期の仕組み

有効な対象ごとに：

1. クラウド API から現在のインバウンドルールを読み取ります。
2. 対象のルールと検出したパブリック IP から、あるべきルール集合を構築します。どのルールにも説明が付き、キーワードを入力した行はキーワードも記録します。
3. 行に **説明キーワード** がある場合、プロトコルとポートが一致し、説明にそのキーワードを含むルールを引き受け、**送信元 IP をその場で置き換えます**（Aliyun は `ModifySecurityGroupRule`）。説明と優先度は変わりません。ルール単位の更新 API がない Tencent Cloud では、同じ説明で削除してから再作成します。キーワードが複数のルールに一致した場合は、推測せずに対象全体をエラーでスキップします。
4. キーワードのない行は従来どおりプレフィックスで管理します。説明が `AutoFirewall` で始まる古いルールを 1 件ずつ削除し（Tencent Cloud のルールインデックスが有効なまま保たれます）、その後で不足しているルールを作成します。
5. グループを再読み取りし、計画したルールが残っていないことを検証します。

自分で作成したルールは、対応する行を設定から削除しても決して削除されません。

書き込み呼び出しは短いバックオフを挟んで最大 3 回再試行されます。エラーはログに記録され、バックグラウンドループを停止させることなく UI に表示されます。

## ファイル

| パス | 用途 |
| --- | --- |
| `%APPDATA%\AutoUpdateCloudFirewall\config.json` | アカウント、対象、設定。シークレットは含みません。 |
| `%LOCALAPPDATA%\AutoUpdateCloudFirewall\logs\app.log` | 追記のみのアクティビティログ。 |
| Windows 資格情報マネージャー、サービス `AutoUpdateCloudFirewall` | AccessKey のシークレット（アカウント ID ごとに 1 件）。 |

`config.json.example` にディスク上の形式が記載されています。編集には GUI の使用をおすすめします。壊れたファイルは `config.json.broken` としてバックアップされ、次回起動時に既定値で置き換えられます。

## プロジェクト構成

| ファイル | 役割 |
| --- | --- |
| `src/main.rs` | ランタイム、チャネル、トレイとウィンドウの起動。 |
| `src/i18n.rs` | 英語 / 簡体字中国語 / 日本語 / 韓国語の文言。 |
| `src/model.rs` | アカウント・対象・設定の型、ポートの解析と整形。 |
| `src/storage.rs` | 設定の読み書きと資格情報ストアへのアクセス。 |
| `src/ip.rs` | パブリック IPv4 の検出と検証。 |
| `src/planner.rs` | あるべきルールとリモートルールの差分、管理対象ルールの選別。 |
| `src/providers/` | Aliyun RPC と Tencent TC3 のクライアントおよび署名。 |
| `src/worker.rs` | バックグラウンドループ、再試行、検証、アカウントのテスト。 |
| `src/platform.rs` | ネイティブウィンドウの表示/非表示、Windows 自動開始。 |
| `src/tray.rs` | 通知領域のアイコンとメニュー。 |
| `src/ui/` | サイドバーシェル、各ページ、アカウント/対象のダイアログ。 |
| `assets/` | アイコン原画（`logo.svg`、`logo-compact.svg`、`logo-mono.svg`、`logo-ui.svg`）と生成済みの PNG/ICO。 |
| `tools/generate-icons.py` | SVG 原画からアイコン一式を再生成します。 |
| `build.rs` | 実行ファイルのアイコンとバージョン情報を埋め込みます。 |

## 制限事項

- IPv4 のみ。
- インバウンドルールのみ。
- ルールは説明のプレフィックスで照合されます。手動で作成したルールをアプリに触らせたくない場合は、そのプレフィックスを編集しないでください。

## リンク

- 公式サイト: [rustsilk.com](https://rustsilk.com)

## ライセンス

このプロジェクトは [MIT ライセンス](LICENSE) で公開されています。

Copyright (c) 2026 Auto Update Cloud Firewall contributors.
