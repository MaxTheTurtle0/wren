//! Refresh tokens in the operating system's credential store: the Keychain
//! on macOS, the Credential Manager on Windows, the Secret Service (GNOME
//! Keyring, KWallet) on Linux.
//!
//! Only refresh tokens are stored. Access tokens live in memory for their
//! hour. Errors from the native stores are reduced to fixed categories,
//! because provider messages can quote platform data.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const SERVICE: &str = "app.wren.Wren";

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum SecretError {
    #[error("the credential store is locked")]
    Locked,
    #[error("the credential store is unavailable")]
    Unavailable,
}

/// A place for secrets. Calls may block (D-Bus, Keychain prompts): run them
/// off the interface thread.
pub trait SecretStore: Send + Sync {
    fn read(&self, key: &str) -> Result<Option<String>, SecretError>;
    fn write(&self, key: &str, secret: &str) -> Result<(), SecretError>;
    fn delete(&self, key: &str) -> Result<(), SecretError>;
}

/// The refresh token's key for one account.
pub fn refresh_key(account: &str) -> String {
    format!("refresh-token:{account}")
}

fn native_error(error: keyring_core::Error) -> SecretError {
    match error {
        keyring_core::Error::NoStorageAccess(_) => SecretError::Locked,
        _ => SecretError::Unavailable,
    }
}

/// The platform's own store.
#[derive(Default)]
pub struct NativeStore {
    store: Mutex<Option<Arc<keyring_core::api::CredentialStore>>>,
}

impl NativeStore {
    fn entry(&self, key: &str) -> Result<keyring_core::Entry, SecretError> {
        let mut store = self.store.lock().unwrap_or_else(|p| p.into_inner());
        if store.is_none() {
            #[cfg(target_os = "linux")]
            let made = zbus_secret_service_keyring_store::Store::new();
            #[cfg(target_os = "macos")]
            let made = apple_native_keyring_store::keychain::Store::new();
            #[cfg(windows)]
            let made = windows_native_keyring_store::Store::new();
            #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
            let made: Result<
                Arc<keyring_core::api::CredentialStore>,
                keyring_core::Error,
            > = Err(keyring_core::Error::NoDefaultStore);
            *store = Some(made.map_err(native_error)?);
        }
        store
            .as_ref()
            .ok_or(SecretError::Unavailable)?
            .build(SERVICE, key, None)
            .map_err(native_error)
    }
}

impl SecretStore for NativeStore {
    fn read(&self, key: &str) -> Result<Option<String>, SecretError> {
        match self.entry(key)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(error) => Err(native_error(error)),
        }
    }

    fn write(&self, key: &str, secret: &str) -> Result<(), SecretError> {
        self.entry(key)?.set_password(secret).map_err(native_error)
    }

    fn delete(&self, key: &str) -> Result<(), SecretError> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(error) => Err(native_error(error)),
        }
    }
}

/// Secrets for this process only: demos and tests.
#[derive(Default)]
pub struct MemoryStore {
    entries: Mutex<HashMap<String, String>>,
}

impl SecretStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, SecretError> {
        Ok(self
            .entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(key)
            .cloned())
    }

    fn write(&self, key: &str, secret: &str) -> Result<(), SecretError> {
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(key.to_owned(), secret.to_owned());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), SecretError> {
        self.entries
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_store_round_trips() {
        let store = MemoryStore::default();
        let key = refresh_key("a.b");
        assert_eq!(store.read(&key).unwrap(), None);
        store.write(&key, "secret").unwrap();
        assert_eq!(store.read(&key).unwrap().as_deref(), Some("secret"));
        store.delete(&key).unwrap();
        assert_eq!(store.read(&key).unwrap(), None);
    }
}
