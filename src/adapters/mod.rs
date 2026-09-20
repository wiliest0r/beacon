use crate::account_store::AccountConfigProvider;
use crate::models::{AccountProfile, UniversalCrmEvent};
use crate::ports::{AccountRepository, CrmGateway, EventSink};

/// Adapter bridging legacy AccountConfigProvider into the AccountRepository port
pub struct AccountStoreAdapter<T: AccountConfigProvider>(pub T);

impl<T: AccountConfigProvider> AccountRepository for AccountStoreAdapter<T> {
    fn get_account(&self, account_id: &str) -> Result<AccountProfile, String> {
        self.0
            .get_account(account_id)
            .map_err(|e| format!("{:?}", e))
    }

    fn is_account_active(&self, account_id: &str) -> bool {
        self.0.is_account_active(account_id)
    }
}

/// Standard Out / Shared Volume NDJSON Event Sink Adapter for Vector Sidecar
pub struct StdoutEventSink;

impl EventSink for StdoutEventSink {
    fn sink_event(&self, event_json: &str) -> Result<(), String> {
        println!("{}", event_json);
        Ok(())
    }
}

/// In-memory / Logging CRM Gateway Adapter
pub struct LoggingCrmGateway;

impl CrmGateway for LoggingCrmGateway {
    fn dispatch_conversion(&self, event: &UniversalCrmEvent) -> Result<(), String> {
        eprintln!(
            "[CRM_SYNC] Dispatched conversion for account {}: {}",
            event.account_id, event.event_id
        );
        Ok(())
    }
}
