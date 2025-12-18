use anyhow::{Context, Result};
use keyring::Entry;

/// Service name used in Keyring / Keyringで使用するサービス名
pub const SERVICE_NAME: &str = "gas";

/// An interface for reading and writing credential information.
/// Used for abstraction with the OS credential manager (Keyring) and test mocks.
/// 資格情報の読み書きを行うためのインターフェース。
/// OS の資格情報マネージャー（Keyring）やテスト用の Mock との抽象化に使用されます。
pub trait CredentialStore {
    /// Save credentials. / 資格情報を保存します。
    fn set(&self, service: &str, username: &str, password: &str) -> Result<()>;

    /// Retrieve the saved credentials. / 保存されている資格情報を取得します。
    fn get(&self, service: &str, username: &str) -> Result<String>;
    
    /// Delete the specified credentials. / 指定された資格情報を削除します。
    fn delete(&self, service: &str, username: &str) -> Result<()>;
}

/// [Production Use] Implementation using the OS Credential Manager (Credential Manager in Windows).
/// 【本番用】OS の資格情報マネージャー（Windows の場合は「資格情報マネージャー」）を使用する実装。
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

// --- Test mock (publicly accessible for use in external tests) ---
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