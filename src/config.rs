use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// 対応している言語
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum Language {
    #[default]
    En,
    Ja,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::En => write!(f, "English"),
            Language::Ja => write!(f, "日本語"),
        }
    }
}

/// アプリケーション全体の設定
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    /// 言語設定
    pub language: Option<Language>,

    /// デフォルトのアカウント名
    pub default_account: Option<String>,
    
    /// 登録されているアカウント一覧
    #[serde(default)]
    pub accounts: HashMap<String, AccountConfig>,

    /// ディレクトリごとのルール
    #[serde(default)]
    pub path_rules: HashMap<String, String>,
}

/// 個別のアカウント情報
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AccountConfig {
    pub username: String,
}

impl AppConfig {
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("gas");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::get_config_path()?)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        let config: AppConfig = toml::from_str(&content)
            .context("Failed to parse config file")?;
            
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        Self::save_to_path(self, &Self::get_config_path()?)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
            
        fs::write(path, content)
            .context("Failed to write config file")?;
            
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_with_language() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_config.toml");

        let mut config = AppConfig::default();
        config.language = Some(Language::Ja);
        config.default_account = Some("Work".to_string());

        config.save_to_path(&file_path).expect("Failed to save");
        let loaded = AppConfig::load_from_path(&file_path).expect("Failed to load");

        assert_eq!(config, loaded);
        assert_eq!(loaded.language, Some(Language::Ja));
    }
}