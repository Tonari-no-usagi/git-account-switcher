use crate::config::{AppConfig, AccountConfig};
use crate::credential::{CredentialStore, SERVICE_NAME};
use anyhow::{Context, Result};
use std::process::Command;

pub const ENV_OVERRIDE: &str = "GAS_ACCOUNT_OVERRIDE";

#[derive(Debug, Default)]
pub struct GitContext {
    pub protocol: String,
    pub host: String,
    pub path: Option<String>,
    pub username: Option<String>,
}

fn parse_git_input(input: &str) -> GitContext {
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

pub fn setup_git_config() -> Result<()> {
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    let exe_path = current_exe.to_string_lossy().replace("\\", "/");
    eprintln!("[GAS] Setting up credential helper to: {}", exe_path);

    let status = Command::new("git")
        .args(&["config", "--global", "credential.helper"])
        .arg(&exe_path)
        .status()
        .context("Failed to execute git command. Please ensure Git is installed and in PATH.")?;

    if !status.success() {
        anyhow::bail!("git config command failed with exit code: {:?}", status.code());
    }

    Ok(())
}

pub fn register_account(
    config: &mut AppConfig,
    store: &impl CredentialStore,
    nickname: String,
    username: String,
    password: String,
) -> Result<()> {
    config.accounts.insert(nickname.clone(), AccountConfig {
        username: username.clone(),
    });

    if config.default_account.is_none() {
        config.default_account = Some(nickname.clone());
    }

    store.set(SERVICE_NAME, &nickname, &password)?;

    Ok(())
}

pub fn get_credentials(
    config: &AppConfig,
    store: &impl CredentialStore,
    input_str: &str, 
    current_dir: &str,
    override_account: Option<String>,
) -> Result<()> {
    let ctx = parse_git_input(input_str);
    
    // 技術ログは英語のまま
    eprintln!("[GAS] Git request: host={}, path={:?}", ctx.host, ctx.path);

    let mut target_account_name = None;

    // (A) Override
    if let Some(account) = override_account {
        eprintln!("[GAS] Override active: using account '{}'", account);
        target_account_name = Some(account);
    } else {
        // (B) Path Rule
        let mut rules: Vec<_> = config.path_rules.iter().collect();
        rules.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));

        for (path_prefix, account) in rules {
            if current_dir.starts_with(path_prefix) {
                eprintln!("[GAS] Rule Match: {} -> Account: {}", path_prefix, account);
                target_account_name = Some(account.clone());
                break;
            }
        }

        // (C) Default
        if target_account_name.is_none() {
            target_account_name = config.default_account.clone();
        }
    }

    let account_name = match target_account_name {
        Some(name) => name,
        None => {
            eprintln!("[GAS] No account rule matched and no default account.");
            return Ok(());
        }
    };

    let account_config = config.accounts.get(&account_name)
        .context(format!("Account '{}' is not registered in config", account_name))?;

    let password = store.get(SERVICE_NAME, &account_name)?;

    println!("username={}", account_config.username);
    println!("password={}", password);
    
    eprintln!("[GAS] Credentials provided for account: {}", account_name);

    Ok(())
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::MockStore;

    fn setup_env() -> (AppConfig, MockStore) {
        let mut config = AppConfig::default();
        let store = MockStore::new();

        config.accounts.insert("Work".to_string(), AccountConfig { username: "WorkUser".to_string() });
        store.set(SERVICE_NAME, "Work", "WorkPass").unwrap();
        config.default_account = Some("Work".to_string());

        (config, store)
    }

    #[test]
    fn test_register_account() {
        let (mut config, store) = setup_env();
        register_account(
            &mut config,
            &store,
            "NewAcc".to_string(),
            "NewUser".to_string(),
            "NewPass".to_string()
        ).unwrap();
        assert!(config.accounts.contains_key("NewAcc"));
    }
}