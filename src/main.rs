//! `gas` is a tool for easily switching between multiple Git accounts (e.g., work, personal).
//! 
//! ## Key Features
//! - Automatically switches the Git account used per directory.
//! - Securely obtains access tokens via GitHub Device Flow (browser authentication).
//! - Safely stores passwords using the OS credential manager (Keyring).
//! - Supports Japanese and English.
//! -------------------------------------------------------------------------------------------- 
//! `gas` は、Git の複数のアカウント（仕事用、個人用など）を簡単に切り替えるためのツールです。
//! 
//! ## 主な機能
//! - ディレクトリごとに使用する Git アカウントを自動切り替え。
//! - GitHub Device Flow（ブラウザ認証）による安全なアクセストークンの取得。
//! - OS の資格情報マネージャー（Keyring）による安全なパスワード保存。
//! - 日本語と英語に対応。

mod config;
mod credential;
mod ops;
mod i18n;
mod github_auth;

use config::{AppConfig, Language};
use credential::KeyringStore;
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use std::io::{self, Read, BufRead};
use std::process::{Command, Stdio, exit};
use i18n::{t, Key};

/// Main command-line argument structure / メインのコマンドライン引数構造体
#[derive(Parser)]
#[command(name = "gas")]
#[command(version = "0.3.0")]
#[command(about = "Git Account Switcher - アカウントを賢く切り替えます")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// List of available subcommands / 実行可能なサブコマンドの一覧
#[derive(Subcommand)]
enum Commands {
    /// Initial setup for git config / 最初のセットアップを行います
    Setup,
    /// Register a new account / アカウントを新しく登録します
    Add { 
        /// アカウントのニックネーム (例: 'Work')
        name: Option<String> 
    },
    /// Remove an account / アカウントの削除
    Remove { 
        /// 削除するアカウント名
        name: Option<String> 
    },
    /// Assign an account to current directory / 現在のディレクトリで使用するアカウントを指定します
    Use { 
        /// 使用するアカウント名
        name: Option<String> 
    },
    /// List all registered accounts / 登録されているアカウントを一覧表示します
    List,
    /// Execute a command with specific account / 指定したアカウントでコマンドを実行します
    With {
        /// 使用するアカウント名
        account: String,
        /// 実行するコマンド
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    /// Change language setting / 言語設定を変更します
    Lang,
    /// [Internal] Git credential helper 'get' command / [内部] Git 認証情報ヘルパーの 'get' コマンド
    Get,
    /// [Internal] Git credential helper 'store' command / [内部] Git 認証情報ヘルパーの 'store' コマンド
    Store,
    /// [Internal] Git credential helper 'erase' command / [内部] Git 認証情報ヘルパーの 'erase' コマンド
    Erase,
}

/// Ensure that the application language is set. / アプリケーションの言語が設定されていることを保証します。
/// If not set, it will ask the user. / 未設定の場合はユーザーに問い合せます。
fn ensure_language(config: &mut AppConfig) -> Result<Language> {
    if let Some(lang) = config.language { return Ok(lang); }
    let items = vec![Language::En, Language::Ja];
    let selection = dialoguer::Select::new().with_prompt(t(&Language::En, Key::AskLanguage)).items(&items).default(0).interact()?;
    let selected_lang = items[selection];
    config.language = Some(selected_lang);
    config.save()?;
    Ok(selected_lang)
}

fn main() -> Result<()> {
// (Existing main processing) / (既存の main 処理)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Setup => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;
            ops::setup_git_config()?;
            eprintln!(">>> {}", t(&lang, Key::SetupComplete));
        }
        Commands::Lang => {
            let mut config = AppConfig::load()?;
            config.language = None; 
            let lang = ensure_language(&mut config)?;
            eprintln!("{}", t(&lang, Key::LanguageChanged));
        }
        Commands::Add { name } => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;
            let store = KeyringStore; 
            let account_name = match name {
                Some(n) => n.clone(),
                None => dialoguer::Input::new().with_prompt(t(&lang, Key::EnterNickname)).interact_text()?,
            };
            let auth_methods = vec![t(&lang, Key::AuthMethodBrowser), t(&lang, Key::AuthMethodToken)];
            let selection = dialoguer::Select::new().with_prompt(t(&lang, Key::SelectAuthMethod)).items(&auth_methods).default(0).interact()?;

            let (username, password) = if selection == 0 {
                let (device_code, user_code, verification_uri, interval) = github_auth::start_device_flow()?;
                eprintln!("{}", t(&lang, Key::DeviceCodeInfo).replace("{}", &user_code));
                let _ = dialoguer::Input::<String>::new().allow_empty(true).interact_text()?;
                let _ = webbrowser::open(&verification_uri);
                eprintln!("{}", t(&lang, Key::WaitingForAuth));
                let token = github_auth::poll_for_token(&device_code, interval)?;
                let user = github_auth::get_username(&token)?;
                eprintln!("{}", t(&lang, Key::AuthSuccess).replace("{}", &user));
                (user, token)
            } else {
                let u = dialoguer::Input::new().with_prompt(t(&lang, Key::EnterUsername)).interact_text()?;
                let p = dialoguer::Password::new().with_prompt(t(&lang, Key::EnterToken)).interact()?;
                (u, p)
            };
            ops::register_account(&mut config, &store, account_name.clone(), username, password)?;
            config.save()?;
            eprintln!("{}", t(&lang, Key::AccountRegistered).replacen("{}", &account_name, 1));
        }
        Commands::Remove { name } => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;
            let store = KeyringStore;
            let account_name = match name {
                Some(n) => n.clone(),
                None => {
                    let accounts: Vec<String> = config.accounts.keys().cloned().collect();
                    if accounts.is_empty() { eprintln!("{}", t(&lang, Key::NoAccounts)); return Ok(()); }
                    let selection = dialoguer::Select::new().with_prompt(t(&lang, Key::SelectAccountToRemove)).items(&accounts).interact()?;
                    accounts[selection].clone()
                }
            };
            ops::remove_account(&mut config, &store, &account_name)?;
            config.save()?;
            eprintln!("{}", t(&lang, Key::AccountRemoved).replace("{}", &account_name));
        }
        Commands::Use { name } => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;
            if config.accounts.is_empty() { eprintln!("{}", t(&lang, Key::NoAccounts)); return Ok(()); }
            let current_dir = std::env::current_dir()?.to_string_lossy().to_string();
            let account_name = match name {
                Some(n) => n.clone(),
                None => {
                    let accounts: Vec<String> = config.accounts.keys().cloned().collect();
                    let selection = dialoguer::Select::new().with_prompt(t(&lang, Key::SelectAccount).replace("{}", &current_dir)).items(&accounts).interact()?;
                    accounts[selection].clone()
                }
            };
            config.path_rules.insert(current_dir.clone(), account_name.clone());
            config.save()?;
            eprintln!("{}", t(&lang, Key::RuleSaved).replacen("{}", &current_dir, 1).replacen("{}", &account_name, 1));
        }
        Commands::List => {
            let config = AppConfig::load()?;
            eprintln!("--- Accounts ---");
            for (name, details) in &config.accounts {
                let mark = if config.default_account.as_ref() == Some(name) { " *" } else { "" };
                eprintln!("{}{}: {}", name, mark, details.username);
            }
        }
        Commands::With { account, cmd } => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;
            let program = &cmd[0];
            let mut child = Command::new(program).args(&cmd[1..]).env(ops::ENV_OVERRIDE, account).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).spawn()?;
            let status = child.wait()?;
            if let Some(code) = status.code() { exit(code); }
        }
        Commands::Get => {
            let config = AppConfig::load()?;
            let mut input = String::new();
            for line in io::stdin().lock().lines() {
                let l = line?; if l.trim().is_empty() { break; }
                input.push_str(&l); input.push('\n');
            }
            if !input.is_empty() {
                let current_dir = std::env::current_dir()?.to_string_lossy().to_string();
                let override_acc = std::env::var(ops::ENV_OVERRIDE).ok();
                ops::get_credentials(&config, &KeyringStore, &input, &current_dir, override_acc)?;
            }
        }
        _ => {}
    }
    Ok(())
}