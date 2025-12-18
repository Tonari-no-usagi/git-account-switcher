use crate::config::Language;

/// メッセージのキー定義
pub enum Key {
    // Prompts
    AskLanguage,
    EnterNickname,
    EnterUsername, // Manual用
    EnterToken,    // Manual用
    SelectAccount,
    SelectAuthMethod, // 新規: 認証方法の選択
    
    // Auth Flow (新規)
    AuthMethodBrowser,
    AuthMethodToken,
    DeviceCodeInfo,
    WaitingForAuth,
    AuthSuccess,
    AuthFailed,
    
    // Status / Success
    SetupComplete,
    SetupHint,
    AccountRegistered,
    RuleSaved,
    LanguageChanged,
    
    // Errors / Warnings
    NoAccounts,
    AccountNotFound,
    CommandError,
    NoCommand,
    
    // Logs / Debug
    OverrideActive,
}

pub fn t(lang: &Language, key: Key) -> &'static str {
    match lang {
        Language::En => match key {
            Key::AskLanguage => "Select Language / 言語を選択してください",
            Key::EnterNickname => "Enter account nickname (e.g. Work)",
            Key::EnterUsername => "Enter Git username",
            Key::EnterToken => "Enter Personal Access Token (hidden)",
            Key::SelectAccount => "Select account to use in '{}'",
            Key::SelectAuthMethod => "Select authentication method",
            
            Key::AuthMethodBrowser => "Browser (Recommended)",
            Key::AuthMethodToken => "Manual Input (Personal Access Token)",
            Key::DeviceCodeInfo => "Copy this code: [{}] -> Press Enter to open GitHub...",
            Key::WaitingForAuth => "Waiting for authorization in browser...",
            Key::AuthSuccess => "Authorization successful! Username: {}",
            Key::AuthFailed => "Authorization failed or timed out.",

            Key::SetupComplete => "Successfully configured git credential helper.",
            Key::SetupHint => "You can now use 'gas' automatically with git commands.",
            Key::AccountRegistered => "Account '{}' registered successfully.",
            Key::RuleSaved => "Rule saved: Directory '{}' will use account '{}'.",
            Key::LanguageChanged => "Language setting changed to English.",
            
            Key::NoAccounts => "No accounts registered. Please use 'gas add' first.",
            Key::AccountNotFound => "Account '{}' is not registered.",
            Key::CommandError => "Failed to execute command: {}",
            Key::NoCommand => "Error: No command specified.",
            
            Key::OverrideActive => "Override active: using account '{}'",
        },
        Language::Ja => match key {
            Key::AskLanguage => "Select Language / 言語を選択してください",
            Key::EnterNickname => "アカウントの登録名を入力してください (例: Work)",
            Key::EnterUsername => "GitHubのユーザー名を入力してください",
            Key::EnterToken => "パーソナルアクセストークンを入力してください (入力文字は隠れます)",
            Key::SelectAccount => "'{}' で使用するアカウントを選択してください",
            Key::SelectAuthMethod => "認証方法を選択してください",

            Key::AuthMethodBrowser => "ブラウザ認証 (推奨)",
            Key::AuthMethodToken => "手動入力 (パーソナルアクセストークン)",
            Key::DeviceCodeInfo => "このコードをコピーしてください: [{}] -> Enterを押すとGitHubを開きます...",
            Key::WaitingForAuth => "ブラウザでの承認を待機しています...",
            Key::AuthSuccess => "認証に成功しました！ ユーザー名: {}",
            Key::AuthFailed => "認証に失敗したか、タイムアウトしました。",
            
            Key::SetupComplete => "GitのCredential Helperへの登録が完了しました。",
            Key::SetupHint => "これでGitコマンド使用時に自動的にgasが動作します。",
            Key::AccountRegistered => "アカウント '{}' を登録しました。",
            Key::RuleSaved => "設定保存: ディレクトリ '{}' ではアカウント '{}' を使用します。",
            Key::LanguageChanged => "言語設定を日本語に変更しました。",
            
            Key::NoAccounts => "アカウントが登録されていません。まずは 'gas add' で登録してください。",
            Key::AccountNotFound => "アカウント '{}' は登録されていません。",
            Key::CommandError => "コマンドの実行に失敗しました: {}",
            Key::NoCommand => "エラー: コマンドが指定されていません。",
            
            Key::OverrideActive => "一時的な切り替え: アカウント '{}' を使用します",
        },
    }
}