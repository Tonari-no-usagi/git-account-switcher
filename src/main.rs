mod config;
mod credential;
mod ops;
mod i18n;
mod github_auth; // 追加

use config::{AppConfig, Language};
use credential::KeyringStore;
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use std::io::Read;
use std::process::{Command, Stdio, exit};
use i18n::{t, Key};

/// Git Account Switcher (GAS)
#[derive(Parser)]
#[command(name = "gas")]
#[command(version = "0.2.1")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
    Add { name: Option<String> },
    Use { name: Option<String> },
    List,
    With {
        account: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    Lang,
    Get,
    Store,
    Erase,
}

fn ensure_language(config: &mut AppConfig) -> Result<Language> {
    if let Some(lang) = config.language {
        return Ok(lang);
    }
    let items = vec![Language::En, Language::Ja];
    let selection = dialoguer::Select::new()
        .with_prompt(t(&Language::En, Key::AskLanguage))
        .items(&items)
        .default(0)
        .interact()?;
    let selected_lang = items[selection];
    config.language = Some(selected_lang);
    config.save()?;
    Ok(selected_lang)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Setup => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;
            ops::setup_git_config()?;
            eprintln!(">>> {}", t(&lang, Key::SetupComplete));
            eprintln!(">>> {}", t(&lang, Key::SetupHint));
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

            // 1. ニックネーム入力
            let account_name = match name {
                Some(n) => n.clone(),
                None => dialoguer::Input::new()
                    .with_prompt(t(&lang, Key::EnterNickname))
                    .interact_text()?,
            };

            // 2. 認証方法の選択
            let auth_methods = vec![
                t(&lang, Key::AuthMethodBrowser),
                t(&lang, Key::AuthMethodToken),
            ];
            let selection = dialoguer::Select::new()
                .with_prompt(t(&lang, Key::SelectAuthMethod))
                .items(&auth_methods)
                .default(0)
                .interact()?;

            let (username, password) = if selection == 0 {
                // --- ブラウザ認証フロー ---
                
                // Client IDが設定されていない場合のチェック
                if github_auth::CLIENT_ID == "YOUR_CLIENT_ID_HERE" {
                    eprintln!("Error: Client ID not configured in source code.");
                    eprintln!("Please use Manual Input or configure CLIENT_ID in src/github_auth.rs");
                    return Ok(());
                }

                // Device Flow開始
                let (device_code, user_code, verification_uri, interval) = github_auth::start_device_flow()?;
                
                // ユーザーにコードを表示して指示
                let msg = t(&lang, Key::DeviceCodeInfo).replace("{}", &user_code);
                eprintln!("{}", msg);
                
                // Enter待機
                let _ = dialoguer::Input::<String>::new()
                    .allow_empty(true)
                    .interact_text()?;

                // ブラウザを開く
                if webbrowser::open(&verification_uri).is_err() {
                    eprintln!("Failed to open browser. Please visit: {}", verification_uri);
                }

                eprintln!("{}", t(&lang, Key::WaitingForAuth));

                // ポーリング開始
                match github_auth::poll_for_token(&device_code, interval) {
                    Ok(token) => {
                        // ユーザー名を取得
                        let user = github_auth::get_username(&token)?;
                        eprintln!("{}", t(&lang, Key::AuthSuccess).replace("{}", &user));
                        (user, token)
                    }
                    Err(e) => {
                        eprintln!("{}: {}", t(&lang, Key::AuthFailed), e);
                        return Ok(());
                    }
                }

            } else {
                // --- 手動入力フロー (既存) ---
                let u = dialoguer::Input::new()
                    .with_prompt(t(&lang, Key::EnterUsername))
                    .interact_text()?;
                let p = dialoguer::Password::new()
                    .with_prompt(t(&lang, Key::EnterToken))
                    .interact()?;
                (u, p)
            };

            // 3. 登録処理
            ops::register_account(&mut config, &store, account_name.clone(), username, password)?;
            config.save()?;
            
            let msg = t(&lang, Key::AccountRegistered).replace("{}", &account_name);
            eprintln!("{}", msg);
        }
        
        // ... (他のコマンドは変更なし) ...
        Commands::Use { name } => {
            let mut config = AppConfig::load()?;
            let lang = ensure_language(&mut config)?;

            if config.accounts.is_empty() {
                eprintln!("{}", t(&lang, Key::NoAccounts));
                return Ok(());
            }

            let current_dir = std::env::current_dir()?.to_string_lossy().to_string();

            let account_name = match name {
                Some(n) => {
                    if !config.accounts.contains_key(n) {
                        let msg = t(&lang, Key::AccountNotFound).replace("{}", n);
                        eprintln!("{}", msg);
                        return Ok(());
                    }
                    n.clone()
                },
                None => {
                    let accounts: Vec<String> = config.accounts.keys().cloned().collect();
                    let prompt = t(&lang, Key::SelectAccount).replace("{}", &current_dir);
                    let selection = dialoguer::Select::new()
                        .with_prompt(prompt)
                        .items(&accounts)
                        .default(0)
                        .interact()?;
                    accounts[selection].clone()
                }
            };

            config.path_rules.insert(current_dir.clone(), account_name.clone());
            config.save()?;
            
            let msg = t(&lang, Key::RuleSaved)
                .replace("{}", &current_dir)
                .replacen("{}", &account_name, 1);
            eprintln!("{}", msg);
        }
        Commands::List => {
            let config = AppConfig::load()?;
            eprintln!("--- Registered Accounts ---");
            if config.accounts.is_empty() {
                let lang = config.language.unwrap_or(Language::En);
                eprintln!("({})", t(&lang, Key::NoAccounts));
            } else {
                for (name, details) in &config.accounts {
                    let default_mark = if config.default_account.as_ref() == Some(name) { " *" } else { "" };
                    eprintln!("{}{}: {}", name, default_mark, details.username);
                }
            }
            if !config.path_rules.is_empty() {
                eprintln!("\n--- Directory Rules ---");
                for (path, acc) in &config.path_rules {
                    eprintln!("{} -> {}", path, acc);
                }
            }
        }
        Commands::With { account, cmd } => {
            let config = AppConfig::load()?;
            let lang = config.language.unwrap_or(Language::En);

            if !config.accounts.contains_key(account) {
                let msg = t(&lang, Key::AccountNotFound).replace("{}", account);
                eprintln!("Error: {}", msg);
                return Ok(());
            }
            if cmd.is_empty() {
                eprintln!("{}", t(&lang, Key::NoCommand));
                return Ok(());
            }

            let msg = t(&lang, Key::OverrideActive).replace("{}", account);
            eprintln!(">>> {}...", msg);
            
            let program = &cmd[0];
            let args = &cmd[1..];

            let mut child = Command::new(program)
                .args(args)
                .env(ops::ENV_OVERRIDE, account)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .context(format!("{}", t(&lang, Key::CommandError).replace("{}", program)))?;

            let status = child.wait()?;
            if let Some(code) = status.code() {
                exit(code);
            }
        }
        Commands::Get => {
            let config = AppConfig::load()?;
            let store = KeyringStore;
            let mut input = String::new();
            if std::io::stdin().read_to_string(&mut input).is_ok() {
                let current_dir = std::env::current_dir()?.to_string_lossy().to_string();
                let override_account = std::env::var(ops::ENV_OVERRIDE).ok();
                ops::get_credentials(&config, &store, &input, &current_dir, override_account)?;
            }
        }
        Commands::Store => {}
        Commands::Erase => {}
    }

    Ok(())
}