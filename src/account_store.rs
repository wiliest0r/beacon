use std::collections::HashMap;
use std::fmt::Display;
use std::sync::RwLock;

use crate::models::{AccountProfile, AccountSecurityConfig, AccountStatus};

/// Errors that can occur when querying the account provider
#[derive(Debug, PartialEq, Eq)]
pub enum AccountStoreError {
    NotFound(String),
    Inactive(String),
    BackendFailure(String),
}

impl Display for AccountStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountStoreError::NotFound(id) => write!(f, "Account '{id}' was not found"),
            AccountStoreError::Inactive(id) => write!(f, "Account '{id}' is suspended or inactive"),
            AccountStoreError::BackendFailure(err) => write!(f, "Account store failure: {err}"),
        }
    }
}

impl std::error::Error for AccountStoreError {}

/// Contract for resolving tenant/client business accounts and their configuration
pub trait AccountConfigProvider: Send + Sync {
    /// Retrieve full account profile by unique account_id
    fn get_account(&self, account_id: &str) -> Result<AccountProfile, AccountStoreError>;

    /// Quick check if account exists and has Active status
    fn is_account_active(&self, account_id: &str) -> bool {
        match self.get_account(account_id) {
            Ok(profile) => profile.status == AccountStatus::Active,
            Err(_) => false,
        }
    }
}

/// In-memory implementation of AccountConfigProvider for local testing, dev, and fallback
#[derive(Debug)]
pub struct InMemoryAccountStore {
    accounts: RwLock<HashMap<String, AccountProfile>>,
}

impl InMemoryAccountStore {
    pub fn new() -> Self {
        let mut map = HashMap::new();

        // Default developer account
        let dev_account = AccountProfile {
            account_id: "acc_playtests_dev".to_string(),
            name: "PlayTests Dev Sandbox".to_string(),
            status: AccountStatus::Active,
            security: AccountSecurityConfig {
                allowed_domains: vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                    "*.example.com".to_string(),
                    "playtests.io".to_string(),
                ],
                hmac_secret: "secret-key-poc-32-bytes-long!".to_string(),
                enforce_domain_check: false,
            },
            crm: crate::models::AccountCrmConfig::default(),
            enable_ecommerce: true,
            enable_spa: true,
        };

        // Fallback default account for legacy APP-XYZ-123 payloads
        let legacy_account = AccountProfile {
            account_id: "APP-XYZ-123".to_string(),
            name: "Legacy Default Account".to_string(),
            status: AccountStatus::Active,
            security: AccountSecurityConfig {
                allowed_domains: vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                    "*.example.com".to_string(),
                ],
                hmac_secret: "secret-key-poc-32-bytes-long!".to_string(),
                enforce_domain_check: false,
            },
            crm: crate::models::AccountCrmConfig::default(),
            enable_ecommerce: true,
            enable_spa: true,
        };

        map.insert(dev_account.account_id.clone(), dev_account);
        map.insert(legacy_account.account_id.clone(), legacy_account);

        Self {
            accounts: RwLock::new(map),
        }
    }

    pub fn register_account(&self, profile: AccountProfile) {
        if let Ok(mut lock) = self.accounts.write() {
            lock.insert(profile.account_id.clone(), profile);
        }
    }
}

impl Default for InMemoryAccountStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountConfigProvider for InMemoryAccountStore {
    fn get_account(&self, account_id: &str) -> Result<AccountProfile, AccountStoreError> {
        let lock = self
            .accounts
            .read()
            .map_err(|e| AccountStoreError::BackendFailure(e.to_string()))?;

        match lock.get(account_id) {
            Some(profile) => {
                if profile.status != AccountStatus::Active {
                    Err(AccountStoreError::Inactive(account_id.to_string()))
                } else {
                    Ok(profile.clone())
                }
            }
            None => Err(AccountStoreError::NotFound(account_id.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccountCrmConfig, CrmProviderType};

    #[test]
    fn test_in_memory_account_store_lookup() {
        let store = InMemoryAccountStore::new();

        let acc = store.get_account("acc_playtests_dev").unwrap();
        assert_eq!(acc.account_id, "acc_playtests_dev");
        assert_eq!(acc.status, AccountStatus::Active);
        assert!(store.is_account_active("acc_playtests_dev"));

        assert_eq!(
            store.get_account("non_existent"),
            Err(AccountStoreError::NotFound("non_existent".to_string()))
        );
    }

    #[test]
    fn test_suspended_account() {
        let store = InMemoryAccountStore::new();
        let suspended = AccountProfile {
            account_id: "acc_suspended".to_string(),
            name: "Suspended Account".to_string(),
            status: AccountStatus::Suspended,
            security: AccountSecurityConfig {
                allowed_domains: vec![],
                hmac_secret: "secret".to_string(),
                enforce_domain_check: true,
            },
            crm: AccountCrmConfig {
                provider: CrmProviderType::Salesforce,
                enabled: false,
                secret_ref: None,
                webhook_url: None,
                field_mappings: None,
            },
            enable_ecommerce: false,
            enable_spa: false,
        };

        store.register_account(suspended);
        assert!(!store.is_account_active("acc_suspended"));
        assert_eq!(
            store.get_account("acc_suspended"),
            Err(AccountStoreError::Inactive("acc_suspended".to_string()))
        );
    }
}
