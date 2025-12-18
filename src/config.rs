use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// 対応している言語
/// 対応している言語を表す列挙型
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Default)]
pub enum Language {
    /// 英語 (デフォルト)
    #[default]
    En,
    /// 日本語
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

/// アプリケーション全体の共通設定を保持する構造体
#[derive(Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct AppConfig {
    /// 言語設定（`Option` で管理し、未設定時は起動時に尋ねる）
    pub language: Option<Language>,

    /// デフォルトで使用する GitHub アカウントのニックネーム
    pub default_account: Option<String>,
    
    /// 登録されているアカウントのニックネームからアカウント設定へのマップ
    #[serde(default)]
    pub accounts: HashMap<String, AccountConfig>,

    /// ディレクトリの絶対パスからアカウントのニックネームへのマップ
    /// 
    /// 特定のディレクトリ配下で Git コマンドを実行する際に、どのアカウントを使用するかを定義します。
    #[serde(default)]
    pub path_rules: HashMap<String, String>,
}

/// 個別のアカウント情報を保持する構造体
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AccountConfig {
    /// GitHub のユーザー名
    pub username: String,
}

impl AppConfig {
    /// 設定ファイルの保存先パスを取得します。
    /// 
    /// OS 標準の設定ディレクトリ（Windows の場合は AppData/Roaming など）内の
    /// `gas/config.toml` を返します。
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("gas");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        Ok(config_dir.join("config.toml"))
    }

    /// デフォルトのパスから設定を読み込みます。
    /// ファイルが存在しない場合は `AppConfig::default()` を返します。
    ///
    /// # Errors
    /// ファイルの読み込みまたはパースに失敗した場合にエラーを返します。
    pub fn load() -> Result<Self> {
        Self::load_from_path(&Self::get_config_path()?)
    }

    /// 指定されたパスから設定を読み込みます。
    ///
    /// # Arguments
    /// * `path` - 読み込み対象のファイルパス
    ///
    /// # Errors
    /// ファイルの読み込みまたはパースに失敗した場合にエラーを返します。
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

    /// デフォルトのパスへ現在の設定を保存します。
    ///
    /// # Errors
    /// シリアライズまたはファイルへの書き込みに失敗した場合にエラーを返します。
    pub fn save(&self) -> Result<()> {
        Self::save_to_path(self, &Self::get_config_path()?)
    }

    /// 指定されたパスへ設定を保存します。
    /// 親ディレクトリが存在しない場合は作成を試みます。
    ///
    /// # Arguments
    /// * `path` - 保存先のファイルパス
    ///
    /// # Errors
    /// シリアライズまたはファイルへの書き込みに失敗した場合にエラーを返します。
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

    #[test]
    fn test_app_config_default_is_empty() {
        let config = AppConfig::default();
        assert!(config.language.is_none());
        assert!(config.default_account.is_none());
        assert!(config.accounts.is_empty());
        assert!(config.path_rules.is_empty());
    }

    #[test]
    fn test_load_non_existent_path_returns_default() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("non_existent.toml");
        let loaded = AppConfig::load_from_path(&file_path).expect("Should not error");
        assert_eq!(loaded, AppConfig::default());
    }

    #[test]
    fn test_language_display() {
        assert_eq!(Language::En.to_string(), "English");
        assert_eq!(Language::Ja.to_string(), "日本語");
    }
}
