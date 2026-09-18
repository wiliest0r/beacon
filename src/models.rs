use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client configuration loaded based on domain, app_id or tenant_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub app_id: String,
    pub allowed_domains: Vec<String>,
    pub hmac_secret: String,
    pub enable_ecommerce: bool,
    pub enable_spa: bool,
}

/// Marketing and AdTech click tracking attributes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawMarketingContext {
    pub gclid: Option<String>,
    pub fbclid: Option<String>,
    pub gbraid: Option<String>,
    pub wbraid: Option<String>,
    pub msclkid: Option<String>,
    pub ttclid: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_term: Option<String>,
    pub utm_content: Option<String>,
}

/// Privacy-safe hashed user identities for AdTech (Enhanced Conversions / CAPI) and CRM
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RawUserIdentity {
    pub hashed_email: Option<String>,
    pub hashed_phone: Option<String>,
    pub crm_lead_id: Option<String>,
}

/// Incoming raw payload from client web tag
#[derive(Debug, Deserialize)]
pub struct RawClientPayload {
    /// Multi-tenant identifier (B2B client/account)
    #[serde(default)]
    pub tenant_id: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,

    /// End-user identifiers (B2C site visitor)
    #[serde(default)]
    pub visitor_id: Option<String>,
    #[serde(default)]
    pub anonymous_id: Option<String>,
    pub session_id: String,
    pub user_id: Option<String>,

    pub event_id: String,
    pub event_name: String,
    pub client_timestamp: String,
    pub token: Option<String>,
    pub context: Option<RawContext>,
    pub marketing: Option<RawMarketingContext>,
    pub user_identity: Option<RawUserIdentity>,
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct RawContext {
    pub page: Option<RawPageContext>,
    pub screen: Option<RawScreenContext>,
    pub locale: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawPageContext {
    pub url: Option<String>,
    pub path: Option<String>,
    pub referrer: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawScreenContext {
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[allow(dead_code)]
    pub density: Option<f32>,
}

/// Fully-enriched, flattened event model ready for Parquet/BigQuery/Lakehouse ingestion
#[derive(Debug, Serialize, Deserialize)]
pub struct IngestedParquetEvent {
    // Multi-tenant & Primary Identity Keys
    pub tenant_id: String,
    pub app_id: String,
    pub event_id: String,
    pub event_name: String,
    pub client_timestamp: String,
    pub server_timestamp: DateTime<Utc>,

    // Traffic Integrity & Anti-Spam
    pub is_quarantined: bool,
    pub quarantine_reason: Option<String>,

    // End-User Identity Dimensions
    pub visitor_id: String,
    pub anonymous_id: String,
    pub session_id: String,
    pub user_id: Option<String>,
    pub hashed_email: Option<String>,
    pub hashed_phone: Option<String>,
    pub crm_lead_id: Option<String>,

    // AdTech & Attribution Dimensions
    pub has_ad_attribution: bool,
    pub is_conversion: bool,
    pub gclid: Option<String>,
    pub fbclid: Option<String>,
    pub gbraid: Option<String>,
    pub wbraid: Option<String>,
    pub msclkid: Option<String>,
    pub ttclid: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_term: Option<String>,
    pub utm_content: Option<String>,

    // Client Context Dimensions (Flattened Columns)
    pub page_url: Option<String>,
    pub page_path: Option<String>,
    pub page_title: Option<String>,
    pub page_referrer: Option<String>,
    pub screen_width: Option<u32>,
    pub screen_height: Option<u32>,
    pub locale: Option<String>,
    pub timezone: Option<String>,

    // Server-Enriched Network Dimensions
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,

    // Semi-structured dynamic payload
    pub custom_properties_json: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_client_payload_deserialization_multitenant() {
        let json_data = r#"{
            "tenant_id": "ten_live_test_123",
            "visitor_id": "vid_abc_456",
            "session_id": "ses_789",
            "event_id": "evt_001",
            "event_name": "purchase",
            "client_timestamp": "2026-09-19T00:00:00Z",
            "marketing": {
                "gclid": "test_gclid_12345",
                "utm_source": "google",
                "utm_campaign": "autumn_promo"
            },
            "user_identity": {
                "hashed_email": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            }
        }"#;

        let payload: RawClientPayload =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        assert_eq!(payload.tenant_id.as_deref(), Some("ten_live_test_123"));
        assert_eq!(payload.visitor_id.as_deref(), Some("vid_abc_456"));
        assert_eq!(payload.event_name, "purchase");

        let marketing = payload
            .marketing
            .expect("Marketing context should be present");
        assert_eq!(marketing.gclid.as_deref(), Some("test_gclid_12345"));
        assert_eq!(marketing.utm_source.as_deref(), Some("google"));

        let identity = payload
            .user_identity
            .expect("User identity should be present");
        assert_eq!(
            identity.hashed_email.as_deref(),
            Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        );
    }

    #[test]
    fn test_raw_client_payload_backwards_compatible() {
        let json_data = r#"{
            "app_id": "APP-XYZ-123",
            "anonymous_id": "anon-999",
            "session_id": "ses-999",
            "event_id": "evt-999",
            "event_name": "page_view",
            "client_timestamp": "2026-09-19T00:00:00Z"
        }"#;

        let payload: RawClientPayload =
            serde_json::from_str(json_data).expect("Failed to deserialize");
        assert_eq!(payload.app_id.as_deref(), Some("APP-XYZ-123"));
        assert_eq!(payload.anonymous_id.as_deref(), Some("anon-999"));
        assert!(payload.tenant_id.is_none());
        assert!(payload.marketing.is_none());
    }
}
