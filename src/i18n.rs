use crate::config::Language;

/// メッセージを特定するためのキー列挙型
pub enum Key {
    /// 言語選択のプロンプト
    AskLanguage,
    /// アカウントのニックネーム入力プロンプト
    EnterNickname,
    /// Git ユーザー名入力プロンプト
    EnterUsername,
    /// 各種アクセストークン入力プロンプト
    EnterToken,
    /// アカウント選択プロンプト
    SelectAccount,
    /// 認証方法選択プロンプト
    SelectAuthMethod,
    /// ブラウザ認証の選択肢
    AuthMethodBrowser,
    /// 手動トークン入力の選択肢
    AuthMethodToken,
    /// デバイスコード情報の表示
    DeviceCodeInfo,
    /// ブラウザ承認待機中のメッセージ
    WaitingForAuth,
    /// 認証成功時のメッセージ
    AuthSuccess,
    /// 認証失敗時のメッセージ
    AuthFailed,
    /// セットアップ完了時のメッセージ
    SetupComplete,
    /// セットアップ後のヒント
    SetupHint,
    /// アカウント登録完了時のメッセージ
    AccountRegistered,
    /// ルール保存完了時のメッセージ
    RuleSaved,
    /// 言語設定変更時のメッセージ
    LanguageChanged,
    /// 登録アカウントがない場合のエラーメッセージ
    NoAccounts,
    /// アカウントが見つからない場合のエラーメッセージ
    AccountNotFound,
    /// コマンド実行エラー
    CommandError,
    /// コマンド未指定エラー
    NoCommand,
    /// オーバーライド有効時の通知
    OverrideActive,
    /// アカウント削除完了時のメッセージ
    AccountRemoved,
    /// 削除対象カウントの選択プロンプト
    SelectAccountToRemove,
}

/// 指定された言語とキーに対応する翻訳済みテキストを返します。
///
/// 現在は英語 (En) と日本語 (Ja) に対応しており、
/// 存在しないキーや組み合わせがないことを列挙型によって保証しています。
pub fn t(lang: &Language, key: Key) -> &'static str {
    match lang {
// (既存の match 本文)
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
            Key::AccountRemoved => "Account '{}' removed successfully.",
            Key::SelectAccountToRemove => "Select account to remove",
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
            Key::AccountRemoved => "アカウント '{}' を削除しました。",
            Key::SelectAccountToRemove => "削除するアカウントを選択してください",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Language;

    #[test]
    fn test_translation_languages() {
        assert!(t(&Language::En, Key::AskLanguage).contains("Select Language"));
        assert!(t(&Language::Ja, Key::AskLanguage).contains("言語を選択してください"));
    }

    #[test]
    fn test_all_keys_en() {
        // コンパイルが通ることで、すべてのキーの網羅性がチェックされています
        let _ = t(&Language::En, Key::AccountRegistered);
    }
}
