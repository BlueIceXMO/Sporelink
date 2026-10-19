use crate::error::{AppError, AppResult};
use keyring::Entry;

const SERVICE: &str = "com.sporelink.app";

#[derive(Clone, Default)]
pub struct Keychain;

impl Keychain {
    pub fn new() -> Self {
        Self
    }

    fn entry(&self, profile_id: &str) -> AppResult<Entry> {
        Entry::new(SERVICE, &format!("provider:{profile_id}"))
            .map_err(|e| AppError::new("keychain_unavailable", e.to_string()))
    }

    pub fn set(&self, profile_id: &str, secret: &str) -> AppResult<()> {
        let entry = self.entry(profile_id)?;
        entry
            .set_password(secret)
            .map_err(|e| AppError::new("keychain_unavailable", e.to_string()))
    }

    pub fn get(&self, profile_id: &str) -> AppResult<Option<String>> {
        let entry = self.entry(profile_id)?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::new("keychain_unavailable", e.to_string())),
        }
    }

    pub fn status(&self, profile_id: &str) -> bool {
        self.get(profile_id).ok().flatten().is_some()
    }

    pub fn delete(&self, profile_id: &str) -> AppResult<()> {
        let entry = self.entry(profile_id)?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(AppError::new("keychain_unavailable", e.to_string())),
        }
    }

    pub fn available(&self) -> bool {
        // A probe entry is not written; we only check that the platform backend can be constructed.
        Entry::new(SERVICE, "probe").is_ok()
    }
}
