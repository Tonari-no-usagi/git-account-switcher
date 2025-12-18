use anyhow::{Context, Result};
use keyring::Entry;

/// Keyringで使用するサービス名
pub const SERVICE_NAME: &str = "gas";

/// 資格情報の読み書きを行うためのインターフェース
pub trait CredentialStore {
    fn set(&self, service: &str, username: &str, password: &str) -> Result<()>;
    fn get(&self, service: &str, username: &str) -> Result<String>;
    fn delete(&self, service: &str, username: &str) -> Result<()>;
}

/// 【本番用】OSの資格情報マネージャー（Keyring）を使う実装
pub struct KeyringStore;

impl KeyringStore {
    fn get_entry(service: &str, username: &str) -> Result<Entry> {
        Entry::new(service, username).context("Failed to create keyring entry")
    }
}

impl CredentialStore for KeyringStore {
    fn set(&self, service: &str, username: &str, password: &str) -> Result<()> {
        let entry = Self::get_entry(service, username)?;
        entry.set_password(password).context("Failed to save password to keyring")
    }

    fn get(&self, service: &str, username: &str) -> Result<String> {
        let entry = Self::get_entry(service, username)?;
        entry.get_password().context("Failed to retrieve password from keyring")
    }

    fn delete(&self, service: &str, username: &str) -> Result<()> {
        let entry = Self::get_entry(service, username)?;
        let _ = entry.delete_password();
        Ok(())
    }
}

// --- テスト用モック（外部のテストからも使えるように公開）---
#[cfg(test)]
pub use self::mock::MockStore;

#[cfg(test)]
mod mock {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    pub struct MockStore {
        storage: RefCell<HashMap<String, String>>,
    }

    impl MockStore {
        pub fn new() -> Self {
            Self {
                storage: RefCell::new(HashMap::new()),
            }
        }
        fn make_key(service: &str, username: &str) -> String {
            format!("{}:{}", service, username)
        }
    }

    impl CredentialStore for MockStore {
        fn set(&self, service: &str, username: &str, password: &str) -> Result<()> {
            let key = Self::make_key(service, username);
            self.storage.borrow_mut().insert(key, password.to_string());
            Ok(())
        }

        fn get(&self, service: &str, username: &str) -> Result<String> {
            let key = Self::make_key(service, username);
            self.storage
                .borrow()
                .get(&key)
                .cloned()
                .context("Password not found")
        }

        fn delete(&self, service: &str, username: &str) -> Result<()> {
            let key = Self::make_key(service, username);
            self.storage.borrow_mut().remove(&key);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::mock::MockStore;

    #[test]
    fn test_set_and_get_password() {
        let store = MockStore::new();
        store.set("test", "user", "pass").unwrap();
        assert_eq!(store.get("test", "user").unwrap(), "pass");
    }
}