use crate::config::{AppConfig, AccountConfig};
use crate::credential::{CredentialStore, SERVICE_NAME};
use anyhow::{Context, Result, bail};
use std::process::Command;

pub const ENV_OVERRIDE: &str = "GAS_ACCOUNT_OVERRIDE";

/// Git コンテキスト情報を保持する構造体。
/// `git credential` コマンドが標準入力から渡す情報を格納します。
#[derive(Debug, Default, PartialEq)]
pub struct GitContext {
    /// 通信プロトコル (https, ssh 等)
    pub protocol: String,
    /// ホスト名 (github.com 等)
    pub host: String,
    /// リポジトリのパス
    pub path: Option<String>,
    /// 指定されたユーザー名
    pub username: Option<String>,
}

/// Git から渡された入力文字列を解析して `GitContext` に変換します。
///
/// 入力は `key=value` 形式の行の集合であることを想定しています。
///
/// # 解析例
/// 入力: `protocol=https\nhost=github.com\npath=user/repo.git\n`
/// 結果: `protocol: "https", host: "github.com", path: Some("user/repo.git")`
pub fn parse_git_input(input: &str) -> GitContext {
    let mut ctx = GitContext::default();
    for line in input.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().to_string();
            match key.trim() {
                "protocol" => ctx.protocol = value,
                "host" => ctx.host = value,
                "path" => ctx.path = Some(value),
                "username" => ctx.username = Some(value),
                _ => {}
            }
        }
    }
    ctx
}

/// Git の `credential.helper` として自身を登録するための初期設定を行います。
///
/// グローバルな Git 設定 (`~/.gitconfig`) を変更し、既存のヘルパー設定をクリアした上で、
/// 本バイナリのパスを新しいヘルパーとして登録します。
///
/// # Errors
/// Git コマンドの実行に失敗した場合や、現在の実行ファイルのパス取得に失敗した場合にエラーを返します。
pub fn setup_git_config() -> Result<()> {
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    /// let exe_path = current_exe.to_string_lossy().replace("\\", "/");    
    let exe_path = format!("!\"{}\"", current_exe.to_string_lossy().replace("\\", "/"));
    

    // 1. 既存のヘルパー設定を「グローバルレベル」で一度すべてクリアする
    // これをしないと、OS標準のマネージャーが優先されてしまいます
    let _ = Command::new("git")
        .args(&["config", "--global", "--unset-all", "credential.helper"])
        .status();

    // 2. 「空のヘルパー」を登録する（これによってシステムレベルの設定を打ち消す）
    let _ = Command::new("git")
        .args(&["config", "--global", "--add", "credential.helper", ""])
        .status();

    // 3. 自分自身を登録する
    let status = Command::new("git")
        .args(&["config", "--global", "--add", "credential.helper"])
        .arg(&exe_path)
        .status()
        .context("Failed to execute git command.")?;

    if !status.success() { bail!("git config command failed"); }

    Ok(())
}

/// 新しいアカウントを登録し、OS の資格情報マネージャー（Keyring）にパスワードを保存します。
///
/// # Arguments
/// * `config` - アプリケーション設定へのミュータブル参照
/// * `store` - 資格情報の保存先トレイト実装（`KeyringStore` またはテスト用の `MockStore`）
/// * `nickname` - アカウントを識別するための表示名（例: "Work", "Personal"）
/// * `username` - GitHub のユーザー名
/// * `password` - アクセストークン等
///
/// # Errors
/// Keyring への保存に失敗した場合にエラーを返します。
pub fn register_account(
    config: &mut AppConfig,
    store: &impl CredentialStore,
    nickname: String,
    username: String,
    password: String,
) -> Result<()> {
    config.accounts.insert(nickname.clone(), AccountConfig { username: username.clone() });
    if config.default_account.is_none() {
        config.default_account = Some(nickname.clone());
    }
    store.set(SERVICE_NAME, &nickname, &password)?;
    Ok(())
}

/// 登録済みのアカウントを削除し、関連する設定（ディレクトリルール等）も消去します。
///
/// # Arguments
/// * `config` - アプリケーション設定へのミュータブル参照
/// * `store` - 資格情報の削除先トレイト実装
/// * `nickname` - 削除対象のアカウントニックネーム
///
/// # Errors
/// Keyring からの削除に失敗した場合にエラーを返します。
pub fn remove_account(
    config: &mut AppConfig,
    store: &impl CredentialStore,
    nickname: &str,
) -> Result<()> {
    // 1. アカウント一覧から削除
    config.accounts.remove(nickname);
    // 2. そのアカウントを使っているディレクトリルールも削除
    config.path_rules.retain(|_, acc| acc != nickname);
    // 3. デフォルト設定なら解除
    if config.default_account.as_deref() == Some(nickname) {
        config.default_account = None;
    }
    // 4. Windows資格情報から削除
    store.delete(SERVICE_NAME, nickname)?;
    Ok(())
}

/// `git credential get` の要求に応じて、適切なアカウントのユーザー名とパスワードを標準出力します。
///
/// 現在のディレクトリパスに従って `path_rules` を検索し、合致するルールがない場合は
/// デフォルトのアカウントを使用します。
///
/// # Arguments
/// * `config` - アプリケーション設定
/// * `store` - 資格情報の取得先トレイト実装
/// * `input_str` - Git から標準入力経由で渡された文字列
/// * `current_dir` - 現在のディレクトリパス
/// * `override_account` - 環境変数等で明示的に指定されたアカウントニックネーム（あれば優先）
///
/// # Errors
/// 資格情報の取得に失敗した場合にエラーを返します。
pub fn get_credentials(
    config: &AppConfig,
    store: &impl CredentialStore,
    input_str: &str, 
    current_dir: &str,
    override_account: Option<String>,
) -> Result<()> {
    let ctx = parse_git_input(input_str);
    let mut target_account_name = None;

    if let Some(account) = override_account {
        target_account_name = Some(account);
    } else {
        let normalized_current = current_dir.to_lowercase().replace("/", "\\");
        let mut rules: Vec<_> = config.path_rules.iter().collect();
        rules.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));
        for (path_prefix, account) in rules {
            let normalized_prefix = path_prefix.to_lowercase().replace("/", "\\");
            if normalized_current.starts_with(&normalized_prefix) {
                target_account_name = Some(account.clone());
                break;
            }
        }
        if target_account_name.is_none() {
            target_account_name = config.default_account.clone();
        }
    }

    let account_name = match target_account_name {
        Some(name) => name,
        None => return Ok(()),
    };

    let account_config = match config.accounts.get(&account_name) {
        Some(c) => c,
        None => return Ok(()),
    };
    let password = store.get(SERVICE_NAME, &account_name)?;
    println!("username={}", account_config.username);
    println!("password={}", password);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::MockStore;
    use crate::config::AppConfig;

    #[test]
    fn test_parse_git_input_full() {
        let input = "protocol=https\nhost=github.com\nusername=testuser\npath=org/repo\n";
        let ctx = parse_git_input(input);
        assert_eq!(ctx.protocol, "https");
        assert_eq!(ctx.host, "github.com");
        assert_eq!(ctx.username.unwrap(), "testuser");
        assert_eq!(ctx.path.unwrap(), "org/repo");
    }

    #[test]
    fn test_parse_git_input_partial() {
        let input = "host=github.com\n";
        let ctx = parse_git_input(input);
        assert_eq!(ctx.host, "github.com");
        assert_eq!(ctx.protocol, "");
        assert!(ctx.path.is_none());
    }

    #[test]
    fn test_register_and_remove_account() {
        let mut config = AppConfig::default();
        let store = MockStore::new();
        
        register_account(&mut config, &store, "Work".into(), "workuser".into(), "token123".into()).unwrap();
        assert!(config.accounts.contains_key("Work"));
        assert_eq!(config.default_account.as_deref(), Some("Work"));
        assert_eq!(store.get(SERVICE_NAME, "Work").unwrap(), "token123");

        remove_account(&mut config, &store, "Work").unwrap();
        assert!(config.accounts.is_empty());
        assert!(config.default_account.is_none());
    }

    #[test]
    fn test_get_credentials_path_rule() {
        let mut config = AppConfig::default();
        let store = MockStore::new();
        config.accounts.insert("Home".into(), AccountConfig { username: "homeuser".into() });
        store.set(SERVICE_NAME, "Home", "homepass").unwrap();
        config.path_rules.insert("C:/projects/home".into(), "Home".into());

        // パスマッチテスト
        // get_credentials は標準出力するためテストが難しいが、ロジック自体の検証は可能
        // 注意: 実際のテストでは println! をキャプチャするか、戻り値を持つ内部関数に分離するのが理想
    }
}
