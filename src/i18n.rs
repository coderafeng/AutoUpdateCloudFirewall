//! Interface translations: English, Simplified Chinese, Japanese and Korean.
//!
//! Static strings live in one table (`Text`). Strings with runtime values use
//! `{placeholder}` markers and are completed with [`tr`].

use crate::model::Language;

macro_rules! texts {
    ($($variant:ident => {
        en: $en:literal,
        zh: $zh:literal,
        ja: $ja:literal,
        ko: $ko:literal
    }),* $(,)?) => {
        /// A translatable interface string.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[allow(dead_code)]
        pub enum Text {
            $($variant),*
        }

        impl Text {
            pub fn get(self, language: Language) -> &'static str {
                match (self, language) {
                    $(
                        (Text::$variant, Language::English) => $en,
                        (Text::$variant, Language::SimplifiedChinese) => $zh,
                        (Text::$variant, Language::Japanese) => $ja,
                        (Text::$variant, Language::Korean) => $ko,
                    )*
                }
            }
        }
    };
}

texts! {
    // ---------------------------------------------------------------- shell
    AppTagline => {
        en: "Auto IP allow-list",
        zh: "自动 IP 白名单",
        ja: "IP 自動許可リスト",
        ko: "자동 IP 허용 목록"
    },
    NavWorkspace => {
        en: "Workspace",
        zh: "工作区",
        ja: "ワークスペース",
        ko: "작업 공간"
    },
    NavSystem => {
        en: "System",
        zh: "系统",
        ja: "システム",
        ko: "시스템"
    },
    WorkerIdle => {
        en: "Worker idle",
        zh: "后台任务空闲",
        ja: "ワーカー待機中",
        ko: "작업자 대기 중"
    },
    WorkerSyncing => {
        en: "Syncing…",
        zh: "正在同步…",
        ja: "同期中…",
        ko: "동기화 중…"
    },

    // --------------------------------------------------------------- pages
    PageOverview => {
        en: "Overview",
        zh: "概览",
        ja: "概要",
        ko: "개요"
    },
    PageOverviewSub => {
        en: "Public IP status and the latest synchronization results.",
        zh: "公网 IP 状态和最近的同步结果。",
        ja: "パブリック IP の状態と最新の同期結果。",
        ko: "공인 IP 상태와 최근 동기화 결과입니다."
    },
    PageAccounts => {
        en: "Cloud accounts",
        zh: "云账号",
        ja: "クラウドアカウント",
        ko: "클라우드 계정"
    },
    PageAccountsSub => {
        en: "Access keys are kept in the operating system credential store.",
        zh: "访问密钥保存在操作系统凭据管理器中。",
        ja: "アクセスキーは OS の資格情報マネージャーに保存されます。",
        ko: "액세스 키는 운영 체제 자격 증명 관리자에 저장됩니다."
    },
    PageTargets => {
        en: "Security groups",
        zh: "安全组",
        ja: "セキュリティグループ",
        ko: "보안 그룹"
    },
    PageTargetsSub => {
        en: "Ingress rules managed by this app are tagged with a description prefix.",
        zh: "由本应用管理的入站规则会带有统一描述前缀。",
        ja: "このアプリが管理するインバウンドルールには説明のプレフィックスが付きます。",
        ko: "이 앱이 관리하는 인바운드 규칙에는 설명 접두사가 붙습니다."
    },
    PageLogs => {
        en: "Activity log",
        zh: "活动日志",
        ja: "アクティビティログ",
        ko: "활동 로그"
    },
    PageLogsSub => {
        en: "Everything the background worker has done, newest last.",
        zh: "后台任务的所有操作记录，最新的在最后。",
        ja: "バックグラウンドワーカーのすべての操作。最新が末尾です。",
        ko: "백그라운드 작업자의 모든 작업 기록이며 최신 항목이 마지막에 표시됩니다."
    },
    PageSettings => {
        en: "Settings",
        zh: "设置",
        ja: "設定",
        ko: "설정"
    },
    PageSettingsSub => {
        en: "Sync interval, IP lookup services and startup behaviour.",
        zh: "同步间隔、IP 查询服务和启动行为。",
        ja: "同期間隔、IP 検索サービス、起動時の動作。",
        ko: "동기화 간격, IP 조회 서비스 및 시작 동작입니다."
    },

    // --------------------------------------------------------------- common
    DetectIp => {
        en: "Detect IP",
        zh: "检测 IP",
        ja: "IP を検出",
        ko: "IP 감지"
    },
    SyncNow => {
        en: "Sync now",
        zh: "立即同步",
        ja: "今すぐ同期",
        ko: "지금 동기화"
    },
    Cancel => {
        en: "Cancel",
        zh: "取消",
        ja: "キャンセル",
        ko: "취소"
    },
    Save => {
        en: "Save",
        zh: "保存",
        ja: "保存",
        ko: "저장"
    },
    Delete => {
        en: "Delete",
        zh: "删除",
        ja: "削除",
        ko: "삭제"
    },
    Edit => {
        en: "Edit",
        zh: "编辑",
        ja: "編集",
        ko: "편집"
    },
    Test => {
        en: "Test",
        zh: "测试",
        ja: "テスト",
        ko: "테스트"
    },
    AddTarget => {
        en: "Add target",
        zh: "添加目标",
        ja: "ターゲットを追加",
        ko: "대상 추가"
    },
    AddAccount => {
        en: "Add account",
        zh: "添加账号",
        ja: "アカウントを追加",
        ko: "계정 추가"
    },
    ViewAll => {
        en: "View all",
        zh: "查看全部",
        ja: "すべて表示",
        ko: "전체 보기"
    },
    Revert => {
        en: "Revert",
        zh: "还原",
        ja: "元に戻す",
        ko: "되돌리기"
    },
    SaveSettings => {
        en: "Save settings",
        zh: "保存设置",
        ja: "設定を保存",
        ko: "설정 저장"
    },
    OpenFolder => {
        en: "Open folder",
        zh: "打开文件夹",
        ja: "フォルダーを開く",
        ko: "폴더 열기"
    },
    ClearView => {
        en: "Clear view",
        zh: "清空视图",
        ja: "表示をクリア",
        ko: "보기 지우기"
    },
    Active => {
        en: "active",
        zh: "个活动",
        ja: "件アクティブ",
        ko: "개 활성"
    },
    AccountFallback => {
        en: "account",
        zh: "账号",
        ja: "アカウント",
        ko: "계정"
    },
    NotSet => {
        en: "not set",
        zh: "未设置",
        ja: "未設定",
        ko: "설정되지 않음"
    },
    NoPorts => {
        en: "No ports",
        zh: "未设置端口",
        ja: "ポートなし",
        ko: "포트 없음"
    },
    PlaceholderAccountName => {
        en: "e.g. production-ops",
        zh: "例如 production-ops",
        ja: "例: production-ops",
        ko: "예: production-ops"
    },
    PlaceholderTargetName => {
        en: "Web servers",
        zh: "Web 服务器",
        ja: "Web サーバー",
        ko: "웹 서버"
    },
    PlaceholderRegionInherit => {
        en: "inherit from account",
        zh: "继承账号地域",
        ja: "アカウントから継承",
        ko: "계정에서 상속"
    },

    // -------------------------------------------------------------- notices
    ConfigNotice => {
        en: "Configuration: {path}. Right-click the tray icon for quick actions.",
        zh: "配置：{path}。右键单击托盘图标可快速操作。",
        ja: "設定: {path}。トレイアイコンを右クリックするとクイック操作ができます。",
        ko: "구성: {path}. 트레이 아이콘을 마우스 오른쪽 버튼으로 클릭하면 빠른 작업을 사용할 수 있습니다."
    },
    WorkerNotRunning => {
        en: "Background worker is not running.",
        zh: "后台任务未运行。",
        ja: "バックグラウンドワーカーが実行されていません。",
        ko: "백그라운드 작업자가 실행 중이 아닙니다."
    },
    SaveConfigFailed => {
        en: "Could not save configuration: {err}",
        zh: "无法保存配置：{err}",
        ja: "設定を保存できませんでした: {err}",
        ko: "구성을 저장할 수 없습니다: {err}"
    },
    IpDetectionFailed => {
        en: "IP detection failed: {message}",
        zh: "IP 检测失败：{message}",
        ja: "IP の検出に失敗しました: {message}",
        ko: "IP 감지 실패: {message}"
    },
    ManualSyncStarted => {
        en: "Manual sync started…",
        zh: "已开始手动同步…",
        ja: "手動同期を開始しました…",
        ko: "수동 동기화를 시작했습니다…"
    },
    LanguageChanged => {
        en: "Language changed.",
        zh: "界面语言已更改。",
        ja: "表示言語を変更しました。",
        ko: "인터페이스 언어가 변경되었습니다."
    },
    SettingsSaved => {
        en: "Settings saved.",
        zh: "设置已保存。",
        ja: "設定を保存しました。",
        ko: "설정을 저장했습니다."
    },
    SettingsSavedToast => {
        en: "Settings saved",
        zh: "设置已保存",
        ja: "設定を保存しました",
        ko: "설정 저장됨"
    },
    IntervalInvalid => {
        en: "Interval must be a number of at least 60 seconds.",
        zh: "间隔必须是至少 60 秒的数字。",
        ja: "間隔は 60 秒以上の数値で指定してください。",
        ko: "간격은 60초 이상의 숫자여야 합니다."
    },
    AddService => {
        en: "Add at least one IP lookup service.",
        zh: "请至少添加一个 IP 查询服务。",
        ja: "IP 検索サービスを 1 つ以上追加してください。",
        ko: "IP 조회 서비스를 하나 이상 추가하세요."
    },
    PrefixEmpty => {
        en: "The rule prefix cannot be empty.",
        zh: "规则前缀不能为空。",
        ja: "ルールのプレフィックスは空にできません。",
        ko: "규칙 접두사는 비워 둘 수 없습니다."
    },
    AutostartEnabled => {
        en: "Auto-start enabled.",
        zh: "已启用开机自启。",
        ja: "自動起動を有効にしました。",
        ko: "자동 시작을 사용하도록 설정했습니다."
    },
    AutostartDisabled => {
        en: "Auto-start disabled.",
        zh: "已关闭开机自启。",
        ja: "自動起動を無効にしました。",
        ko: "자동 시작을 사용하지 않도록 설정했습니다."
    },
    AutostartFailed => {
        en: "Could not change auto-start: {err}",
        zh: "无法更改开机自启设置：{err}",
        ja: "自動起動を変更できませんでした: {err}",
        ko: "자동 시작을 변경할 수 없습니다: {err}"
    },

    // -------------------------------------------------------------- overview
    PublicIp => {
        en: "Public IP",
        zh: "公网 IP",
        ja: "パブリック IP",
        ko: "공인 IP"
    },
    CheckedAt => {
        en: "checked at {time}",
        zh: "检查于 {time}",
        ja: "{time} に確認",
        ko: "{time}에 확인"
    },
    NotCheckedYet => {
        en: "not checked yet",
        zh: "尚未检查",
        ja: "まだ確認していません",
        ko: "아직 확인하지 않음"
    },
    LastSync => {
        en: "Last sync",
        zh: "上次同步",
        ja: "最終同期",
        ko: "마지막 동기화"
    },
    Never => {
        en: "Never",
        zh: "从未",
        ja: "なし",
        ko: "없음"
    },
    DryRunOnHint => {
        en: "dry run mode is on",
        zh: "试运行模式已开启",
        ja: "ドライランモードが有効です",
        ko: "드라이런 모드가 켜져 있습니다"
    },
    AutoUpdatedHint => {
        en: "security groups updated automatically",
        zh: "安全组会自动更新",
        ja: "セキュリティグループは自動更新されます",
        ko: "보안 그룹이 자동으로 업데이트됩니다"
    },
    Accounts => {
        en: "Accounts",
        zh: "账号",
        ja: "アカウント",
        ko: "계정"
    },
    Targets => {
        en: "Targets",
        zh: "目标",
        ja: "ターゲット",
        ko: "대상"
    },
    EnabledTotal => {
        en: "enabled / total",
        zh: "已启用 / 总数",
        ja: "有効 / 合計",
        ko: "사용 / 전체"
    },
    NoTargetsHint => {
        en: "No targets yet. Add a cloud account first, then a security group.",
        zh: "还没有目标。请先添加云账号，再添加安全组。",
        ja: "ターゲットはまだありません。まずクラウドアカウントを追加し、次にセキュリティグループを追加してください。",
        ko: "아직 대상이 없습니다. 먼저 클라우드 계정을 추가한 다음 보안 그룹을 추가하세요."
    },
    WaitingNextCheck => {
        en: "waiting for the next check",
        zh: "等待下次检查",
        ja: "次の確認を待機中",
        ko: "다음 확인 대기 중"
    },
    StatusIdle => {
        en: "Idle",
        zh: "空闲",
        ja: "待機中",
        ko: "대기"
    },
    StatusSyncing => {
        en: "Syncing",
        zh: "同步中",
        ja: "同期中",
        ko: "동기화 중"
    },
    StatusInSync => {
        en: "In sync",
        zh: "已同步",
        ja: "同期済み",
        ko: "동기화됨"
    },
    StatusError => {
        en: "Error",
        zh: "错误",
        ja: "エラー",
        ko: "오류"
    },
    TargetStatus => {
        en: "Target status",
        zh: "目标状态",
        ja: "ターゲットの状態",
        ko: "대상 상태"
    },
    ActiveCount => {
        en: "{count} active",
        zh: "{count} 个活动",
        ja: "{count} 件アクティブ",
        ko: "{count}개 활성"
    },
    RecentActivity => {
        en: "Recent activity",
        zh: "最近活动",
        ja: "最近のアクティビティ",
        ko: "최근 활동"
    },
    NothingHappened => {
        en: "Nothing has happened yet.",
        zh: "还没有任何记录。",
        ja: "まだ何もありません。",
        ko: "아직 아무 일도 없었습니다."
    },
    SyncSecurityGroups => {
        en: "Sync security groups",
        zh: "同步安全组",
        ja: "セキュリティグループを同期",
        ko: "보안 그룹 동기화"
    },
    DryRunOn => {
        en: "Dry run: on",
        zh: "试运行：开",
        ja: "ドライラン: オン",
        ko: "드라이런: 켜짐"
    },
    DryRunOff => {
        en: "Dry run: off",
        zh: "试运行：关",
        ja: "ドライラン: オフ",
        ko: "드라이런: 꺼짐"
    },
    NextCheck => {
        en: "next automatic check in up to {secs} s",
        zh: "下次自动检查最多 {secs} 秒后",
        ja: "次回の自動チェックは最大 {secs} 秒後",
        ko: "다음 자동 확인은 최대 {secs}초 후"
    },

    // -------------------------------------------------------------- accounts
    AccountsSummary => {
        en: "{count} account(s) configured · secrets live in Windows Credential Manager",
        zh: "已配置 {count} 个账号 · 密钥保存在 Windows 凭据管理器中",
        ja: "{count} 件のアカウントを構成済み · シークレットは Windows 資格情報マネージャーに保存",
        ko: "{count}개 계정 구성됨 · 보안 비밀은 Windows 자격 증명 관리자에 저장됩니다"
    },
    AliyunAccount => {
        en: "Aliyun account",
        zh: "阿里云账号",
        ja: "Aliyun アカウント",
        ko: "Aliyun 계정"
    },
    TencentAccount => {
        en: "Tencent account",
        zh: "腾讯云账号",
        ja: "Tencent Cloud アカウント",
        ko: "Tencent Cloud 계정"
    },
    NoAccounts => {
        en: "No cloud accounts yet",
        zh: "还没有云账号",
        ja: "クラウドアカウントがまだありません",
        ko: "아직 클라우드 계정이 없습니다"
    },
    NoAccountsHint => {
        en: "Add an Aliyun or Tencent Cloud account. Only an AccessKey with security-group permissions is required.",
        zh: "添加阿里云或腾讯云账号。只需要具有安全组权限的 AccessKey。",
        ja: "Aliyun または Tencent Cloud アカウントを追加します。セキュリティグループ権限を持つ AccessKey のみが必要です。",
        ko: "Aliyun 또는 Tencent Cloud 계정을 추가하세요. 보안 그룹 권한이 있는 AccessKey만 있으면 됩니다."
    },
    Disabled => {
        en: "disabled",
        zh: "已停用",
        ja: "無効",
        ko: "비활성화됨"
    },
    RegionLabel => {
        en: "Region {region}",
        zh: "地域 {region}",
        ja: "リージョン {region}",
        ko: "리전 {region}"
    },
    KeyLabel => {
        en: "Key {key}",
        zh: "密钥 {key}",
        ja: "キー {key}",
        ko: "키 {key}"
    },
    TargetCount => {
        en: "{count} target(s)",
        zh: "{count} 个目标",
        ja: "{count} 件のターゲット",
        ko: "대상 {count}개"
    },
    EnableAccountTooltip => {
        en: "Enable or pause this account",
        zh: "启用或暂停此账号",
        ja: "このアカウントを有効化または一時停止",
        ko: "이 계정을 사용하거나 일시 중지"
    },
    TestingCredential => {
        en: "Testing credential…",
        zh: "正在测试凭据…",
        ja: "資格情報をテストしています…",
        ko: "자격 증명 테스트 중…"
    },

    // --------------------------------------------------------------- targets
    AddAccountFirst => {
        en: "Add an account first",
        zh: "请先添加账号",
        ja: "先にアカウントを追加してください",
        ko: "먼저 계정을 추가하세요"
    },
    AddAccountFirstHint => {
        en: "Security groups belong to a cloud account. Create an account on the Accounts page, then add targets here.",
        zh: "安全组属于云账号。请先在“云账号”页面创建账号，然后在此添加目标。",
        ja: "セキュリティグループはクラウドアカウントに属します。先に [クラウドアカウント] ページでアカウントを作成し、ここでターゲットを追加してください。",
        ko: "보안 그룹은 클라우드 계정에 속합니다. 먼저 [클라우드 계정] 페이지에서 계정을 만든 다음 여기에서 대상을 추가하세요."
    },
    TargetsIntro => {
        en: "Each target keeps only rules whose description starts with the configured prefix. All other rules are untouched.",
        zh: "每个目标只保留描述以配置前缀开头的规则，其他规则不受影响。",
        ja: "各ターゲットは、説明が設定したプレフィックスで始まるルールのみを保持します。それ以外のルールは変更されません。",
        ko: "각 대상은 설명이 구성된 접두사로 시작하는 규칙만 유지합니다. 다른 규칙은 변경되지 않습니다."
    },
    NoTargetsForAccount => {
        en: "No security groups for this account yet.",
        zh: "此账号还没有安全组。",
        ja: "このアカウントにはまだセキュリティグループがありません。",
        ko: "이 계정에는 아직 보안 그룹이 없습니다."
    },
    RegionLower => {
        en: "region {region}",
        zh: "地域 {region}",
        ja: "リージョン {region}",
        ko: "리전 {region}"
    },
    Paused => {
        en: "paused",
        zh: "已暂停",
        ja: "一時停止中",
        ko: "일시 중지됨"
    },

    // ------------------------------------------------------------------ logs
    LogEntries => {
        en: "{count} entries · {path}",
        zh: "{count} 条记录 · {path}",
        ja: "{count} 件 · {path}",
        ko: "{count}개 항목 · {path}"
    },
    LogEmpty => {
        en: "The log is empty.",
        zh: "日志为空。",
        ja: "ログは空です。",
        ko: "로그가 비어 있습니다."
    },

    // -------------------------------------------------------------- settings
    SectionSync => {
        en: "Synchronization",
        zh: "同步",
        ja: "同期",
        ko: "동기화"
    },
    SectionSyncHint => {
        en: "How often the public IP is checked and which services are used.",
        zh: "公网 IP 的检查频率以及使用的查询服务。",
        ja: "パブリック IP を確認する頻度と使用するサービス。",
        ko: "공인 IP 확인 빈도와 사용할 조회 서비스입니다."
    },
    CheckInterval => {
        en: "Check interval (seconds)",
        zh: "检查间隔（秒）",
        ja: "確認間隔（秒）",
        ko: "확인 간격(초)"
    },
    CheckIntervalHint => {
        en: "The worker re-checks the public IP on this interval. Minimum 60 seconds, 300 by default.",
        zh: "后台任务按此间隔重新检查公网 IP，最小 60 秒，默认 300 秒。",
        ja: "ワーカーはこの間隔でパブリック IP を再確認します。最小 60 秒、既定値は 300 秒。",
        ko: "작업자는 이 간격으로 공인 IP를 다시 확인합니다. 최소 60초, 기본값 300초."
    },
    ManagedPrefix => {
        en: "Managed-rule description prefix",
        zh: "受管规则描述前缀",
        ja: "管理対象ルールの説明プレフィックス",
        ko: "관리 규칙 설명 접두사"
    },
    ManagedPrefixHint => {
        en: "Only ingress rules whose description starts with this prefix are modified or removed.",
        zh: "只有描述以该前缀开头的入站规则会被修改或删除。",
        ja: "説明がこのプレフィックスで始まるインバウンドルールのみが変更または削除されます。",
        ko: "설명이 이 접두사로 시작하는 인바운드 규칙만 수정되거나 삭제됩니다."
    },
    IpServices => {
        en: "IP lookup services",
        zh: "IP 查询服务",
        ja: "IP 検索サービス",
        ko: "IP 조회 서비스"
    },
    IpServicesHint => {
        en: "Comma-separated URLs tried in order until a public IPv4 address is returned.",
        zh: "以逗号分隔的 URL，按顺序尝试直到返回公网 IPv4 地址。",
        ja: "カンマ区切りの URL。パブリック IPv4 アドレスが返るまで順に試します。",
        ko: "쉼표로 구분된 URL이며, 공인 IPv4 주소가 반환될 때까지 순서대로 시도합니다."
    },
    SectionBehaviour => {
        en: "Behaviour",
        zh: "行为",
        ja: "動作",
        ko: "동작"
    },
    SectionBehaviourHint => {
        en: "Background and startup behaviour.",
        zh: "后台运行与启动行为。",
        ja: "バックグラウンドと起動時の動作。",
        ko: "백그라운드 및 시작 동작입니다."
    },
    DryRun => {
        en: "Dry run",
        zh: "试运行",
        ja: "ドライラン",
        ko: "드라이런"
    },
    DryRunHint => {
        en: "Plan and log rule changes without calling the cloud write APIs.",
        zh: "只规划和记录规则变更，不调用云端写入 API。",
        ja: "クラウドの書き込み API を呼ばずに、ルール変更を計画して記録します。",
        ko: "클라우드 쓰기 API를 호출하지 않고 규칙 변경을 계획하고 기록합니다."
    },
    CloseToTray => {
        en: "Close to tray",
        zh: "关闭到托盘",
        ja: "トレイに閉じる",
        ko: "트레이로 닫기"
    },
    CloseToTrayHint => {
        en: "Closing the window keeps the background worker running.",
        zh: "关闭窗口后后台任务继续运行。",
        ja: "ウィンドウを閉じてもバックグラウンドワーカーは動作を続けます。",
        ko: "창을 닫아도 백그라운드 작업자는 계속 실행됩니다."
    },
    StartWithWindows => {
        en: "Start with Windows",
        zh: "开机自启",
        ja: "Windows 起動時に開始",
        ko: "Windows 시작 시 실행"
    },
    StartWithWindowsHint => {
        en: "Register this executable under the current user's Run key.",
        zh: "将本程序注册到当前用户的 Run 注册表项。",
        ja: "この実行ファイルを現在のユーザーの Run キーに登録します。",
        ko: "이 실행 파일을 현재 사용자의 Run 레지스트리 키에 등록합니다."
    },
    StartMinimized => {
        en: "Start minimized",
        zh: "最小化启动",
        ja: "最小化して起動",
        ko: "최소화 상태로 시작"
    },
    StartMinimizedHint => {
        en: "Open straight into the notification area (takes effect on next launch).",
        zh: "直接启动到通知区域（下次启动生效）。",
        ja: "通知領域に直接起動します（次回起動時に有効）。",
        ko: "알림 영역으로 바로 시작합니다(다음 실행부터 적용)."
    },
    SectionFiles => {
        en: "Files",
        zh: "文件",
        ja: "ファイル",
        ko: "파일"
    },
    SectionFilesHint => {
        en: "Configuration and log locations.",
        zh: "配置和日志位置。",
        ja: "設定とログの場所。",
        ko: "구성 및 로그 위치입니다."
    },
    SectionAbout => {
        en: "About",
        zh: "关于",
        ja: "このアプリについて",
        ko: "정보"
    },
    SectionAboutHint => {
        en: "Version {version} · source code and license.",
        zh: "版本 {version} · 源代码与许可证。",
        ja: "バージョン {version} · ソースコードとライセンス。",
        ko: "버전 {version} · 소스 코드 및 라이선스."
    },
    SourceCode => {
        en: "Source code",
        zh: "源代码",
        ja: "ソースコード",
        ko: "소스 코드"
    },
    SourceCodeHint => {
        en: "The full project lives on GitHub; issues and pull requests are welcome.",
        zh: "完整项目托管在 GitHub，欢迎提交 issue 和 pull request。",
        ja: "プロジェクト全体は GitHub にあります。issue と pull request を歓迎します。",
        ko: "전체 프로젝트는 GitHub에 있습니다. 이슈와 풀 리퀘스트를 환영합니다."
    },
    LicenseMit => {
        en: "MIT License",
        zh: "MIT 许可证",
        ja: "MIT ライセンス",
        ko: "MIT 라이선스"
    },
    LicenseMitHint => {
        en: "Use, modify and redistribute freely, as long as the copyright notice is kept.",
        zh: "保留版权声明即可自由使用、修改和再分发。",
        ja: "著作権表示を保持すれば、自由に使用・変更・再配布できます。",
        ko: "저작권 표시를 유지하는 한 자유롭게 사용, 수정, 재배포할 수 있습니다."
    },
    ReadLicense => {
        en: "Read license",
        zh: "查看许可证",
        ja: "ライセンスを読む",
        ko: "라이선스 보기"
    },
    ProjectHomepage => {
        en: "Project homepage",
        zh: "项目主页",
        ja: "プロジェクトサイト",
        ko: "프로젝트 홈페이지"
    },
    OpenLink => {
        en: "Open",
        zh: "打开",
        ja: "開く",
        ko: "열기"
    },
    OpenConfigFolder => {
        en: "Open config folder",
        zh: "打开配置文件夹",
        ja: "設定フォルダーを開く",
        ko: "구성 폴더 열기"
    },
    OpenLogFolder => {
        en: "Open log folder",
        zh: "打开日志文件夹",
        ja: "ログフォルダーを開く",
        ko: "로그 폴더 열기"
    },
    PreviewPlan => {
        en: "Preview managed rule plan",
        zh: "预览受管规则计划",
        ja: "管理ルールの計画をプレビュー",
        ko: "관리 규칙 계획 미리보기"
    },
    ConfigFile => {
        en: "Config file: {path}",
        zh: "配置文件：{path}",
        ja: "設定ファイル: {path}",
        ko: "구성 파일: {path}"
    },
    SectionLanguage => {
        en: "Language",
        zh: "语言",
        ja: "言語",
        ko: "언어"
    },
    SectionLanguageHint => {
        en: "Choose the interface language. The tray menu changes immediately.",
        zh: "选择界面语言，托盘菜单会立即更新。",
        ja: "表示言語を選択します。トレイメニューはすぐに更新されます。",
        ko: "인터페이스 언어를 선택합니다. 트레이 메뉴가 즉시 업데이트됩니다."
    },

    // ---------------------------------------------------------------- dialogs
    AddCloudAccount => {
        en: "Add cloud account",
        zh: "添加云账号",
        ja: "クラウドアカウントを追加",
        ko: "클라우드 계정 추가"
    },
    EditCloudAccount => {
        en: "Edit cloud account",
        zh: "编辑云账号",
        ja: "クラウドアカウントを編集",
        ko: "클라우드 계정 편집"
    },
    AccessKeyRequiredHint => {
        en: "AccessKey id and secret are required.",
        zh: "需要 AccessKey ID 和 Secret。",
        ja: "AccessKey ID とシークレットが必要です。",
        ko: "AccessKey ID와 보안 비밀이 필요합니다."
    },
    AccountName => {
        en: "Account name",
        zh: "账号名称",
        ja: "アカウント名",
        ko: "계정 이름"
    },
    AccessKeyId => {
        en: "AccessKey id",
        zh: "AccessKey ID",
        ja: "AccessKey ID",
        ko: "AccessKey ID"
    },
    AccessKeySecret => {
        en: "AccessKey secret",
        zh: "AccessKey Secret",
        ja: "AccessKey シークレット",
        ko: "AccessKey 보안 비밀"
    },
    SecretKeepHint => {
        en: "Leave blank to keep the stored secret.",
        zh: "留空以保留已保存的 Secret。",
        ja: "空欄のままにすると保存済みのシークレットを保持します。",
        ko: "비워 두면 저장된 보안 비밀을 유지합니다."
    },
    SecretStoreHint => {
        en: "Stored in the Windows Credential Manager, never in config.json.",
        zh: "保存在 Windows 凭据管理器中，不会写入 config.json。",
        ja: "Windows 資格情報マネージャーに保存され、config.json には書き込まれません。",
        ko: "Windows 자격 증명 관리자에 저장되며 config.json에는 기록되지 않습니다."
    },
    DefaultRegion => {
        en: "Default region",
        zh: "默认地域",
        ja: "既定のリージョン",
        ko: "기본 리전"
    },
    DefaultRegionHint => {
        en: "Used when a target does not override the region.",
        zh: "当目标未覆盖地域时使用。",
        ja: "ターゲットがリージョンを上書きしない場合に使用します。",
        ko: "대상이 리전을 재정의하지 않을 때 사용됩니다."
    },
    ErrAccountNameRequired => {
        en: "Account name is required.",
        zh: "账号名称不能为空。",
        ja: "アカウント名は必須です。",
        ko: "계정 이름은 필수입니다."
    },
    ErrAccessKeyRequired => {
        en: "AccessKey id is required.",
        zh: "AccessKey ID 不能为空。",
        ja: "AccessKey ID は必須です。",
        ko: "AccessKey ID는 필수입니다."
    },
    ErrRegionRequired => {
        en: "Region is required.",
        zh: "地域不能为空。",
        ja: "リージョンは必須です。",
        ko: "리전은 필수입니다."
    },
    ErrSecretRequired => {
        en: "AccessKey secret is required for a new account.",
        zh: "新账号必须填写 AccessKey Secret。",
        ja: "新しいアカウントには AccessKey シークレットが必要です。",
        ko: "새 계정에는 AccessKey 보안 비밀이 필요합니다."
    },
    ErrAccountGone => {
        en: "Account no longer exists.",
        zh: "账号已不存在。",
        ja: "アカウントは既に存在しません。",
        ko: "계정이 더 이상 존재하지 않습니다."
    },
    AccountUpdated => {
        en: "Account updated.",
        zh: "账号已更新。",
        ja: "アカウントを更新しました。",
        ko: "계정이 업데이트되었습니다."
    },
    AccountAdded => {
        en: "Account added. Add a security group target next.",
        zh: "账号已添加。接下来请添加安全组目标。",
        ja: "アカウントを追加しました。次にセキュリティグループターゲットを追加してください。",
        ko: "계정이 추가되었습니다. 다음으로 보안 그룹 대상을 추가하세요."
    },
    ToastAccountUpdated => {
        en: "Account updated",
        zh: "账号已更新",
        ja: "アカウントを更新しました",
        ko: "계정 업데이트됨"
    },
    ToastAccountAdded => {
        en: "Account added",
        zh: "账号已添加",
        ja: "アカウントを追加しました",
        ko: "계정 추가됨"
    },
    AddSecurityGroupTarget => {
        en: "Add security group target",
        zh: "添加安全组目标",
        ja: "セキュリティグループターゲットを追加",
        ko: "보안 그룹 대상 추가"
    },
    EditSecurityGroupTarget => {
        en: "Edit security group target",
        zh: "编辑安全组目标",
        ja: "セキュリティグループターゲットを編集",
        ko: "보안 그룹 대상 편집"
    },
    TargetName => {
        en: "Target name",
        zh: "目标名称",
        ja: "ターゲット名",
        ko: "대상 이름"
    },
    SecurityGroupId => {
        en: "Security group id",
        zh: "安全组 ID",
        ja: "セキュリティグループ ID",
        ko: "보안 그룹 ID"
    },
    SecurityGroupIdHint => {
        en: "For example sg-bp1abcdefg (Aliyun) or sg-xxxxxxxx (Tencent).",
        zh: "例如 sg-bp1abcdefg（阿里云）或 sg-xxxxxxxx（腾讯云）。",
        ja: "例: sg-bp1abcdefg（Aliyun）または sg-xxxxxxxx（Tencent）。",
        ko: "예: sg-bp1abcdefg(Aliyun) 또는 sg-xxxxxxxx(Tencent)."
    },
    RegionOverride => {
        en: "Region override (optional)",
        zh: "地域覆盖（可选）",
        ja: "リージョンの上書き（任意）",
        ko: "리전 재정의(선택 사항)"
    },
    RegionOverrideHint => {
        en: "Leave empty to use the account region.",
        zh: "留空则使用账号地域。",
        ja: "空欄の場合はアカウントのリージョンを使用します。",
        ko: "비워 두면 계정 리전을 사용합니다."
    },
    Rules => {
        en: "Rules",
        zh: "规则",
        ja: "ルール",
        ko: "규칙"
    },
    PortsHint => {
        en: "One port or range per row. An optional keyword finds an existing rule by its description and updates its IP in place.",
        zh: "每行一个端口或范围。备注关键字用于定位已有规则，同步时就地更新它的 IP。",
        ja: "1 行につき 1 つのポートまたは範囲。キーワードを入れると、説明文で既存ルールを特定し、IP を直接更新します。",
        ko: "행마다 포트 또는 범위 하나. 설명 키워드를 입력하면 기존 규칙을 찾아 IP를 직접 갱신합니다."
    },
    RulePort => {
        en: "Port",
        zh: "端口",
        ja: "ポート",
        ko: "포트"
    },
    RuleProtocol => {
        en: "Protocol",
        zh: "协议",
        ja: "プロトコル",
        ko: "프로토콜"
    },
    RuleKeyword => {
        en: "Description keyword",
        zh: "备注关键字",
        ja: "説明のキーワード",
        ko: "설명 키워드"
    },
    RuleKeywordPlaceholder => {
        en: "e.g. home",
        zh: "例如 home",
        ja: "例: home",
        ko: "예: home"
    },
    RuleKeywordHint => {
        en: "Optional. Only rules whose description contains this keyword are updated.",
        zh: "可选。只更新备注中包含该关键字的规则。",
        ja: "任意。説明にこのキーワードを含むルールだけを更新します。",
        ko: "선택 사항. 설명에 이 키워드가 포함된 규칙만 갱신합니다."
    },
    AddRule => {
        en: "Add rule",
        zh: "添加规则",
        ja: "ルールを追加",
        ko: "규칙 추가"
    },
    RemoveRule => {
        en: "Remove rule",
        zh: "删除规则",
        ja: "ルールを削除",
        ko: "규칙 삭제"
    },
    ErrDuplicateRule => {
        en: "`{port}` is configured twice with the same keyword.",
        zh: "端口 `{port}` 出现了重复配置（协议、端口、关键字相同）。",
        ja: "ポート `{port}` が同じキーワードで重複しています。",
        ko: "포트 `{port}`가 같은 키워드로 중복 설정되었습니다."
    },
    ErrTargetNameRequired => {
        en: "Target name is required.",
        zh: "目标名称不能为空。",
        ja: "ターゲット名は必須です。",
        ko: "대상 이름은 필수입니다."
    },
    ErrPortListEmpty => {
        en: "Add at least one rule.",
        zh: "请至少添加一条规则。",
        ja: "ルールを 1 つ以上追加してください。",
        ko: "규칙을 하나 이상 추가하세요."
    },
    ErrPortRequired => {
        en: "Enter a port for every rule.",
        zh: "每条规则都要填写端口。",
        ja: "すべてのルールにポートを入力してください。",
        ko: "모든 규칙에 포트를 입력하세요."
    },
    ErrPortInvalid => {
        en: "`{port}` is not a valid port.",
        zh: "`{port}` 不是有效的端口。",
        ja: "`{port}` は有効なポートではありません。",
        ko: "`{port}`은(는) 올바른 포트가 아닙니다."
    },
    ErrPortRange => {
        en: "Port `{port}` must be between 1 and 65535.",
        zh: "端口 `{port}` 必须在 1 到 65535 之间。",
        ja: "ポート `{port}` は 1〜65535 の範囲で指定してください。",
        ko: "포트 `{port}`는 1~65535 사이여야 합니다."
    },
    ErrPortReversed => {
        en: "Port range `{port}` is reversed.",
        zh: "端口范围 `{port}` 上下限颠倒。",
        ja: "ポート範囲 `{port}` が逆になっています。",
        ko: "포트 범위 `{port}`의 시작과 끝이 바뀌었습니다."
    },
    ErrGroupRequired => {
        en: "Security group id is required.",
        zh: "安全组 ID 不能为空。",
        ja: "セキュリティグループ ID は必須です。",
        ko: "보안 그룹 ID는 필수입니다."
    },
    ErrNoAccount => {
        en: "No cloud account selected.",
        zh: "未选择云账号。",
        ja: "クラウドアカウントが選択されていません。",
        ko: "선택된 클라우드 계정이 없습니다."
    },
    ErrLinkedAccountGone => {
        en: "The linked account no longer exists.",
        zh: "关联的账号已不存在。",
        ja: "リンクされたアカウントは既に存在しません。",
        ko: "연결된 계정이 더 이상 존재하지 않습니다."
    },
    ErrTargetGone => {
        en: "Target no longer exists.",
        zh: "目标已不存在。",
        ja: "ターゲットは既に存在しません。",
        ko: "대상이 더 이상 존재하지 않습니다."
    },
    TargetUpdatedNotice => {
        en: "Target updated. Changes apply on the next sync.",
        zh: "目标已更新，下次同步时生效。",
        ja: "ターゲットを更新しました。次回の同期時に反映されます。",
        ko: "대상이 업데이트되었습니다. 다음 동기화 시 적용됩니다."
    },
    TargetAddedNotice => {
        en: "Target added. It will be synced on the next check.",
        zh: "目标已添加，将在下次检查时同步。",
        ja: "ターゲットを追加しました。次回の確認時に同期されます。",
        ko: "대상이 추가되었습니다. 다음 확인 시 동기화됩니다."
    },
    ToastTargetUpdated => {
        en: "Target updated",
        zh: "目标已更新",
        ja: "ターゲットを更新しました",
        ko: "대상 업데이트됨"
    },
    ToastTargetAdded => {
        en: "Target added",
        zh: "目标已添加",
        ja: "ターゲットを追加しました",
        ko: "대상 추가됨"
    },
    DeleteAccountTitle => {
        en: "Delete cloud account?",
        zh: "删除云账号？",
        ja: "クラウドアカウントを削除しますか？",
        ko: "클라우드 계정을 삭제할까요?"
    },
    DeleteAccountBody => {
        en: "`{name}` and all of its security-group targets will be removed.",
        zh: "`{name}` 及其所有安全组目标都将被移除。",
        ja: "`{name}` とそのすべてのセキュリティグループターゲットが削除されます。",
        ko: "`{name}` 및 해당 보안 그룹 대상이 모두 제거됩니다."
    },
    DeleteAccountSecret => {
        en: "The access-key secret is also deleted from the OS credential store.",
        zh: "访问密钥也会从操作系统凭据管理器中删除。",
        ja: "アクセスキーのシークレットも OS の資格情報マネージャーから削除されます。",
        ko: "액세스 키 보안 비밀도 운영 체제 자격 증명 관리자에서 삭제됩니다."
    },
    AccountDeleted => {
        en: "Account deleted.",
        zh: "账号已删除。",
        ja: "アカウントを削除しました。",
        ko: "계정이 삭제되었습니다."
    },
    ToastAccountDeleted => {
        en: "Account deleted",
        zh: "账号已删除",
        ja: "アカウントを削除しました",
        ko: "계정 삭제됨"
    },
    DeleteTargetTitle => {
        en: "Delete target?",
        zh: "删除目标？",
        ja: "ターゲットを削除しますか？",
        ko: "대상을 삭제할까요?"
    },
    DeleteTargetBody => {
        en: "`{name}` will be removed from this app. Existing cloud rules are kept until the next sync removes the managed ones.",
        zh: "`{name}` 将从此应用中移除。现有云端规则会保留，直到下次同步移除受管规则。",
        ja: "`{name}` はこのアプリから削除されます。既存のクラウドルールは、次回の同期で管理対象ルールが削除されるまで保持されます。",
        ko: "`{name}`이(가) 이 앱에서 제거됩니다. 기존 클라우드 규칙은 다음 동기화에서 관리 규칙이 제거될 때까지 유지됩니다."
    },
    TargetDeleted => {
        en: "Target deleted.",
        zh: "目标已删除。",
        ja: "ターゲットを削除しました。",
        ko: "대상이 삭제되었습니다."
    },
    ToastTargetDeleted => {
        en: "Target deleted",
        zh: "目标已删除",
        ja: "ターゲットを削除しました",
        ko: "대상 삭제됨"
    },

    // ------------------------------------------------------------------- tray
    TrayShow => {
        en: "Show window",
        zh: "显示窗口",
        ja: "ウィンドウを表示",
        ko: "창 표시"
    },
    TrayHide => {
        en: "Hide window",
        zh: "隐藏窗口",
        ja: "ウィンドウを隠す",
        ko: "창 숨기기"
    },
    TraySync => {
        en: "Sync now",
        zh: "立即同步",
        ja: "今すぐ同期",
        ko: "지금 동기화"
    },
    TrayConfig => {
        en: "Open config folder",
        zh: "打开配置文件夹",
        ja: "設定フォルダーを開く",
        ko: "구성 폴더 열기"
    },
    TrayQuit => {
        en: "Quit",
        zh: "退出",
        ja: "終了",
        ko: "종료"
    },

    // ----------------------------------------------------------------- worker
    WStarted => {
        en: "background worker started (interval {secs}s)",
        zh: "后台任务已启动（间隔 {secs} 秒）",
        ja: "バックグラウンドワーカーを開始しました（間隔 {secs} 秒）",
        ko: "백그라운드 작업자가 시작되었습니다(간격 {secs}초)"
    },
    WStopped => {
        en: "background worker stopped",
        zh: "后台任务已停止",
        ja: "バックグラウンドワーカーを停止しました",
        ko: "백그라운드 작업자가 중지되었습니다"
    },
    WConfigUpdated => {
        en: "configuration updated by the UI",
        zh: "界面已更新配置",
        ja: "UI によって設定が更新されました",
        ko: "UI에서 구성이 업데이트되었습니다"
    },
    WCurrentIp => {
        en: "current public IP is {ip}",
        zh: "当前公网 IP 是 {ip}",
        ja: "現在のパブリック IP は {ip} です",
        ko: "현재 공인 IP는 {ip}입니다"
    },
    WNothingToSync => {
        en: "nothing to sync: add a cloud account and a security-group target first",
        zh: "没有可同步的内容：请先添加云账号和安全组目标",
        ja: "同期する項目がありません。まずクラウドアカウントとセキュリティグループターゲットを追加してください",
        ko: "동기화할 항목이 없습니다. 먼저 클라우드 계정과 보안 그룹 대상을 추가하세요"
    },
    WNothingConfigured => {
        en: "Nothing configured yet. Add an account and a target.",
        zh: "尚未配置。请添加账号和目标。",
        ja: "まだ設定されていません。アカウントとターゲットを追加してください。",
        ko: "아직 구성되지 않았습니다. 계정과 대상을 추가하세요."
    },
    WIpFailedReuse => {
        en: "IP detection failed ({err}); reusing last known IP {cached}",
        zh: "IP 检测失败（{err}）；继续使用上次的 IP {cached}",
        ja: "IP の検出に失敗しました（{err}）。最後に確認した IP {cached} を引き続き使用します",
        ko: "IP 감지 실패({err}). 마지막으로 확인한 IP {cached}을(를) 계속 사용합니다"
    },
    WIpDetectFailedNotice => {
        en: "Could not detect the public IP address.",
        zh: "无法检测公网 IP 地址。",
        ja: "パブリック IP アドレスを検出できませんでした。",
        ko: "공인 IP 주소를 감지할 수 없습니다."
    },
    WIpChanged => {
        en: "public IP is {ip}, syncing security groups",
        zh: "公网 IP 为 {ip}，正在同步安全组",
        ja: "パブリック IP は {ip} です。セキュリティグループを同期しています",
        ko: "공인 IP는 {ip}이며 보안 그룹을 동기화하는 중입니다"
    },
    WIpSame => {
        en: "public IP is still {ip}, verifying security groups",
        zh: "公网 IP 仍为 {ip}，正在校验安全组",
        ja: "パブリック IP は引き続き {ip} です。セキュリティグループを確認しています",
        ko: "공인 IP가 여전히 {ip}이며 보안 그룹을 확인하는 중입니다"
    },
    WMissingAccount => {
        en: "target `{name}` is linked to a missing account",
        zh: "目标 `{name}` 关联的账号不存在",
        ja: "ターゲット `{name}` は存在しないアカウントにリンクされています",
        ko: "대상 `{name}`이(가) 존재하지 않는 계정에 연결되어 있습니다"
    },
    WSyncing => {
        en: "syncing…",
        zh: "正在同步…",
        ja: "同期中…",
        ko: "동기화 중…"
    },
    WTargetFailed => {
        en: "{target}: {message}",
        zh: "{target}：{message}",
        ja: "{target}: {message}",
        ko: "{target}: {message}"
    },
    WNothingSynced => {
        en: "Nothing synced ({skipped} target(s) skipped)",
        zh: "未同步任何目标（跳过 {skipped} 个目标）",
        ja: "同期されたターゲットはありません（{skipped} 件スキップ）",
        ko: "동기화된 대상 없음({skipped}개 건너뜀)"
    },
    WSummary => {
        en: "{ok} synced, {failed} failed",
        zh: "成功 {ok} 个，失败 {failed} 个",
        ja: "{ok} 件成功、{failed} 件失敗",
        ko: "{ok}개 성공, {failed}개 실패"
    },
    WSummarySkipped => {
        en: "{ok} synced, {failed} failed, {skipped} skipped",
        zh: "成功 {ok} 个，失败 {failed} 个，跳过 {skipped} 个",
        ja: "{ok} 件成功、{failed} 件失敗、{skipped} 件スキップ",
        ko: "{ok}개 성공, {failed}개 실패, {skipped}개 건너뜀"
    },
    DryRunSuffix => {
        en: " (dry run)",
        zh: "（试运行）",
        ja: "（ドライラン）",
        ko: " (드라이런)"
    },
    WSyncFinished => {
        en: "sync finished: {summary}",
        zh: "同步完成：{summary}",
        ja: "同期完了: {summary}",
        ko: "동기화 완료: {summary}"
    },
    WDryRunNoChange => {
        en: "dry run enabled: no cloud rule was changed",
        zh: "已启用试运行：未更改任何云端规则",
        ja: "ドライランが有効です。クラウドルールは変更されていません",
        ko: "드라이런이 활성화되어 클라우드 규칙이 변경되지 않았습니다"
    },
    WSecretMissing => {
        en: "access-key secret for account `{name}` is missing from the OS credential store",
        zh: "账号 `{name}` 的访问密钥 Secret 不在操作系统凭据管理器中",
        ja: "アカウント `{name}` のアクセスキーシークレットが OS の資格情報マネージャーにありません",
        ko: "계정 `{name}`의 액세스 키 보안 비밀이 운영 체제 자격 증명 관리자에 없습니다"
    },
    WNoAccessKey => {
        en: "account `{name}` has no access-key id",
        zh: "账号 `{name}` 没有 AccessKey ID",
        ja: "アカウント `{name}` に AccessKey ID がありません",
        ko: "계정 `{name}`에 AccessKey ID가 없습니다"
    },
    WReadGroup => {
        en: "read security group {id}",
        zh: "读取安全组 {id}",
        ja: "セキュリティグループ {id} を読み取り",
        ko: "보안 그룹 {id} 읽기"
    },
    WAlreadyUpToDate => {
        en: "already up to date",
        zh: "已是最新",
        ja: "既に最新です",
        ko: "이미 최신 상태입니다"
    },
    WWouldRemove => {
        en: "{target}: would remove {protocol} {port}/{cidr} from {description}",
        zh: "{target}：将移除 {protocol} {port}/{cidr}（{description}）",
        ja: "{target}: {protocol} {port}/{cidr} を削除します（{description}）",
        ko: "{target}: {protocol} {port}/{cidr} 제거 예정({description})"
    },
    WWouldAllow => {
        en: "{target}: would allow {protocol} {port} from {cidr}",
        zh: "{target}：将允许 {protocol} {port} 来自 {cidr}",
        ja: "{target}: {protocol} {port} を {cidr} から許可します",
        ko: "{target}: {protocol} {port}을(를) {cidr}에서 허용 예정"
    },
    WDryRunSummary => {
        en: "dry run: {summary}",
        zh: "试运行：{summary}",
        ja: "ドライラン: {summary}",
        ko: "드라이런: {summary}"
    },
    WRemoveFailed => {
        en: "{target}: remove {protocol} {port}/{cidr} failed (attempt {attempt}/{attempts}): {err}",
        zh: "{target}：移除 {protocol} {port}/{cidr} 失败（第 {attempt}/{attempts} 次尝试）：{err}",
        ja: "{target}: {protocol} {port}/{cidr} の削除に失敗（試行 {attempt}/{attempts}）: {err}",
        ko: "{target}: {protocol} {port}/{cidr} 제거 실패(시도 {attempt}/{attempts}): {err}"
    },
    WRemoveContext => {
        en: "remove stale rule {protocol} {port}/{cidr}",
        zh: "移除过期规则 {protocol} {port}/{cidr}",
        ja: "古いルール {protocol} {port}/{cidr} を削除",
        ko: "오래된 규칙 {protocol} {port}/{cidr} 제거"
    },
    WRemoved => {
        en: "{target}: removed {protocol} {port} from {cidr}",
        zh: "{target}：已移除 {protocol} {port}（{cidr}）",
        ja: "{target}: {protocol} {port} を {cidr} から削除しました",
        ko: "{target}: {protocol} {port}을(를) {cidr}에서 제거했습니다"
    },
    WRefreshVersion => {
        en: "refresh Tencent policy version",
        zh: "刷新腾讯云策略版本",
        ja: "Tencent Cloud ポリシーバージョンを更新",
        ko: "Tencent Cloud 정책 버전 새로 고침"
    },
    WAllowFailed => {
        en: "{target}: allow {protocol} {port} failed (attempt {attempt}/{attempts}): {err}",
        zh: "{target}：允许 {protocol} {port} 失败（第 {attempt}/{attempts} 次尝试）：{err}",
        ja: "{target}: {protocol} {port} の許可に失敗（試行 {attempt}/{attempts}）: {err}",
        ko: "{target}: {protocol} {port} 허용 실패(시도 {attempt}/{attempts}): {err}"
    },
    WAllowContext => {
        en: "allow {protocol} {port}",
        zh: "允许 {protocol} {port}",
        ja: "{protocol} {port} を許可",
        ko: "{protocol} {port} 허용"
    },
    WAllowed => {
        en: "{target}: allowed {protocol} {port} from {cidr}",
        zh: "{target}：已允许 {protocol} {port} 来自 {cidr}",
        ja: "{target}: {protocol} {port} を {cidr} から許可しました",
        ko: "{target}: {protocol} {port}을(를) {cidr}에서 허용했습니다"
    },
    WUpdated => {
        en: "{target}: updated {protocol} {port} from {from} to {to}",
        zh: "{target}：已更新 {protocol} {port} 的源 IP，{from} → {to}",
        ja: "{target}: {protocol} {port} の送信元 IP を {from} → {to} に更新しました",
        ko: "{target}: {protocol} {port}의 원본 IP를 {from} → {to}로 갱신했습니다"
    },
    WWouldUpdate => {
        en: "{target}: would update {protocol} {port} ({description}) from {from} to {to}",
        zh: "{target}：将把 {protocol} {port}（{description}）的源 IP 从 {from} 改为 {to}",
        ja: "{target}: {protocol} {port}（{description}）の送信元 IP を {from} から {to} に変更します",
        ko: "{target}: {protocol} {port}({description})의 원본 IP를 {from}에서 {to}로 변경 예정"
    },
    WUpdateFailed => {
        en: "{target}: update {protocol} {port} failed (attempt {attempt}/{attempts}): {err}",
        zh: "{target}：更新 {protocol} {port} 失败（第 {attempt}/{attempts} 次尝试）：{err}",
        ja: "{target}: {protocol} {port} の更新に失敗（試行 {attempt}/{attempts}）: {err}",
        ko: "{target}: {protocol} {port} 갱신 실패(시도 {attempt}/{attempts}): {err}"
    },
    WUpdateContext => {
        en: "update the source address of {protocol} {port}",
        zh: "更新 {protocol} {port} 的源 IP",
        ja: "{protocol} {port} の送信元 IP を更新",
        ko: "{protocol} {port}의 원본 IP 갱신"
    },
    WUpdateDuplicate => {
        en: "update {protocol} {port} — another rule already allows {cidr}, delete that duplicate rule first",
        zh: "更新 {protocol} {port}（安全组里已有另一条同协议、同端口、源为 {cidr} 的规则，请先删除重复规则）",
        ja: "{protocol} {port} を更新できません（{cidr} を許可する同プロトコル・同ポートのルールが既に存在します。重複ルールを先に削除してください）",
        ko: "{protocol} {port}을(를) 갱신할 수 없습니다({cidr}을(를) 허용하는 동일 프로토콜·포트 규칙이 이미 있습니다. 중복 규칙을 먼저 삭제하세요)"
    },
    WAmbiguousRule => {
        en: "{target} skipped: port {port} keyword \"{keyword}\" matches {count} rules — make the keyword more specific",
        zh: "{target} 已跳过：端口 {port} 的关键字「{keyword}」命中了 {count} 条规则，请把关键字写得更精确",
        ja: "{target} をスキップ: ポート {port} のキーワード「{keyword}」が {count} 件のルールに一致しました。キーワードをより具体的にしてください",
        ko: "{target} 건너뜀: 포트 {port} 키워드 \"{keyword}\"가 규칙 {count}개와 일치합니다. 키워드를 더 구체적으로 지정하세요"
    },
    WSharedRule => {
        en: "{target} skipped: port {port} keyword \"{keyword}\" points at a rule another configured row already uses",
        zh: "{target} 已跳过：端口 {port} 的关键字「{keyword}」命中的规则已被另一条配置占用",
        ja: "{target} をスキップ: ポート {port} のキーワード「{keyword}」が指すルールは別の設定が既に使用しています",
        ko: "{target} 건너뜀: 포트 {port} 키워드 \"{keyword}\"가 가리키는 규칙을 다른 설정이 이미 사용 중입니다"
    },
    WInSync => {
        en: "in sync — {active} managed rule(s) active, {added} added, {updated} updated",
        zh: "已同步 — {active} 条受管规则生效，新增 {added} 条，更新 {updated} 条",
        ja: "同期済み — 管理ルール {active} 件が有効、{added} 件追加、{updated} 件更新",
        ko: "동기화됨 — 관리 규칙 {active}개 활성, {added}개 추가, {updated}개 갱신"
    },
    WOutOfSync => {
        en: "rules still out of sync after applying: {summary}",
        zh: "应用后规则仍未同步：{summary}",
        ja: "適用後もルールが同期されていません: {summary}",
        ko: "적용 후에도 규칙이 동기화되지 않았습니다: {summary}"
    },
    WAccountGone => {
        en: "account no longer exists",
        zh: "账号已不存在",
        ja: "アカウントは既に存在しません",
        ko: "계정이 더 이상 존재하지 않습니다"
    },
    WSecretMissingContext => {
        en: "secret is missing from the OS credential store",
        zh: "操作系统凭据管理器中缺少 Secret",
        ja: "OS の資格情報マネージャーにシークレットがありません",
        ko: "운영 체제 자격 증명 관리자에 보안 비밀이 없습니다"
    },
    WCredentialValid => {
        en: "credential is valid and the security-group API is reachable",
        zh: "凭据有效，且可以访问安全组 API",
        ja: "資格情報は有効で、セキュリティグループ API にアクセスできます",
        ko: "자격 증명이 유효하며 보안 그룹 API에 액세스할 수 있습니다"
    },
    WAccountTestOk => {
        en: "account `{name}`: {message}",
        zh: "账号 `{name}`：{message}",
        ja: "アカウント `{name}`: {message}",
        ko: "계정 `{name}`: {message}"
    },
    WAccountTestFailed => {
        en: "account `{name}` test failed: {message}",
        zh: "账号 `{name}` 测试失败：{message}",
        ja: "アカウント `{name}` のテストに失敗しました: {message}",
        ko: "계정 `{name}` 테스트 실패: {message}"
    },
    PlanSummary => {
        en: "{add} to add, {update} to update, {remove} to remove, {keep} unchanged",
        zh: "新增 {add}，更新 {update}，移除 {remove}，保持不变 {keep}",
        ja: "追加 {add}、更新 {update}、削除 {remove}、変更なし {keep}",
        ko: "추가 {add}, 갱신 {update}, 제거 {remove}, 유지 {keep}"
    },
}

/// Look up a static string.
pub fn t(language: Language, text: Text) -> &'static str {
    text.get(language)
}

/// Replace `{name}` placeholders with the supplied values.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in args {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}

/// Look up a template and fill in its placeholders.
pub fn tr(language: Language, text: Text, args: &[(&str, &str)]) -> String {
    fill(text.get(language), args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_all_texts() {
        // `Text::get` is exhaustive, so this mainly guards against empty strings.
        for language in Language::all() {
            for text in [Text::PageOverview, Text::SettingsSaved, Text::TrayQuit] {
                assert!(!text.get(language).is_empty());
            }
        }
    }

    #[test]
    fn fills_placeholders() {
        let message = tr(
            Language::SimplifiedChinese,
            Text::ActiveCount,
            &[("count", "3")],
        );
        assert_eq!(message, "3 个活动");
        assert!(!message.contains("{count}"));
    }

    #[test]
    fn port_errors_are_translated() {
        use crate::model::PortError;
        for language in Language::all() {
            let message = PortError::Invalid("abc".to_string()).message(language);
            assert!(message.contains("abc"));
            assert!(!message.contains("{port}"));
            assert!(!PortError::Empty.message(language).is_empty());
        }
    }
}
