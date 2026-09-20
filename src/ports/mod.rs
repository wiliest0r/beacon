use crate::models::{AccountProfile, UniversalCrmEvent};

/// Port for retrieving and managing account profiles and tenant configuration
pub trait AccountRepository: Send + Sync {
    fn get_account(&self, account_id: &str) -> Result<AccountProfile, String>;
    fn is_account_active(&self, account_id: &str) -> bool;
}

/// Port for publishing ingested events to the streaming bus / buffer
pub trait EventSink: Send + Sync {
    fn sink_event(&self, event_json: &str) -> Result<(), String>;
}

/// Port for dispatching conversion events downstream to CRM platforms
pub trait CrmGateway: Send + Sync {
    fn dispatch_conversion(&self, event: &UniversalCrmEvent) -> Result<(), String>;
}
