use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Legacy client configuration kept for backwards compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub app_id: String,
    pub allowed_domains: Vec<String>,
    pub hmac_secret: String,
    pub enable_ecommerce: bool,
    pub enable_spa: bool,
}

/// Status of a B2B business account / tenant
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountStatus {
    Active,
    Suspended,
    Provisioning,
}

/// Security rules and domain validation policies for an account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSecurityConfig {
    pub allowed_domains: Vec<String>,
    pub hmac_secret: String,
    pub enforce_domain_check: bool,
}

/// Supported CRM vendors or generic integration types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CrmProviderType {
    Salesforce,
    Hubspot,
    Pipedrive,
    CustomWebhook,
    #[default]
    None,
}

/// CRM configuration contract for account-level lead and deal synchronization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccountCrmConfig {
    pub provider: CrmProviderType,
    pub enabled: bool,
    /// Reference to Secret Manager secret (e.g. `projects/.../secrets/beacon-acc_123-crm`)
    pub secret_ref: Option<String>,
    /// Custom webhook endpoint for CustomWebhook CRM mode
    pub webhook_url: Option<String>,
    /// Optional field mapping: event properties -> CRM fields
    pub field_mappings: Option<HashMap<String, String>>,
}

/// Business profile and full configuration of a platform tenant / client account
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountProfile {
    pub account_id: String,
    pub name: String,
    pub status: AccountStatus,
    pub security: AccountSecurityConfig,
    pub crm: AccountCrmConfig,
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
    /// Web Analytics Tag & Measurement IDs (Standard Public Client Contract)
    #[serde(default)]
    pub tag_id: Option<String>,
    #[serde(default)]
    pub measurement_id: Option<String>,

    /// Canonical B2B account identifier (Internal Contract)
    #[serde(default)]
    pub account_id: Option<String>,

    /// Backwards compatibility alias for account_id
    #[serde(default)]
    pub tenant_id: Option<String>,

    /// Legacy application ID
    #[serde(default)]
    pub app_id: Option<String>,

    /// Standard Web Analytics Client ID (GA4/Segment client_id)
    #[serde(default)]
    pub client_id: Option<String>,

    /// Zero-dependency browser signature / entropy hash (Anti-AdBlock / Anti-Fingerprint filter)
    #[serde(default)]
    pub sig: Option<String>,
    #[serde(default)]
    pub client_sig: Option<String>,

    /// Persistent client physical device or browser identifier (Backward compat)
    #[serde(default)]
    pub device_id: Option<String>,

    /// Hardware/canvas/screen fingerprint hash
    #[serde(default)]
    pub device_fp: Option<String>,

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
    pub account_id: String,
    pub tenant_id: String, // Kept for backwards compatibility
    pub app_id: String,
    pub event_id: String,
    pub event_name: String,
    pub client_timestamp: String,
    pub server_timestamp: DateTime<Utc>,

    // Traffic Integrity & Anti-Spam
    pub is_quarantined: bool,
    pub quarantine_reason: Option<String>,

    // Device & End-User Identity Dimensions
    pub device_id: String,
    pub device_fp: Option<String>,
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

/// Universal normalized contract for dispatching conversion events into any CRM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalCrmEvent {
    pub account_id: String,
    pub event_id: String,
    pub event_name: String,
    pub timestamp: DateTime<Utc>,

    // Device & Identity Resolution
    pub device_id: String,
    pub visitor_id: String,
    pub session_id: Option<String>,
    pub hashed_email: Option<String>,
    pub hashed_phone: Option<String>,
    pub crm_lead_id: Option<String>,

    // Attribution Linking
    pub gclid: Option<String>,
    pub fbclid: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,

    // Conversion & Deal metrics
    pub is_conversion: bool,
    pub conversion_value: Option<f64>,
    pub currency: Option<String>,

    // Custom mapped payload
    pub properties: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_client_standard_contract_deserialization() {
        let json_payload = r#"{
            "tag_id": "GTM-PLAYTESTS-01",
            "client_id": "c7a8b9d0-1234-4567-89ab-cdef01234567",
            "sig": "a9f4c3b218e76543",
            "session_id": "sess_456",
            "event_id": "evt_789",
            "event_name": "page_view",
            "client_timestamp": "2026-09-20T12:00:00Z"
        }"#;

        let parsed: RawClientPayload = serde_json::from_str(json_payload).unwrap();
        assert_eq!(parsed.tag_id.as_deref(), Some("GTM-PLAYTESTS-01"));
        assert_eq!(
            parsed.client_id.as_deref(),
            Some("c7a8b9d0-1234-4567-89ab-cdef01234567")
        );
        assert_eq!(parsed.sig.as_deref(), Some("a9f4c3b218e76543"));
    }

    #[test]
    fn test_payload_deserialization_with_account_id() {
        let json_payload = r#"{
            "account_id": "acc_nike_global_01",
            "visitor_id": "vis_anon_987",
            "session_id": "sess_456",
            "event_id": "evt_789",
            "event_name": "lead_submission",
            "client_timestamp": "2026-09-20T12:00:00Z",
            "marketing": {
                "gclid": "test_gclid_12345",
                "utm_source": "google",
                "utm_campaign": "spring_sale"
            },
            "user_identity": {
                "hashed_email": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                "crm_lead_id": "SF-LEAD-99120"
            }
        }"#;

        let parsed: RawClientPayload = serde_json::from_str(json_payload).unwrap();
        assert_eq!(parsed.account_id.as_deref(), Some("acc_nike_global_01"));
        assert_eq!(parsed.visitor_id.as_deref(), Some("vis_anon_987"));

        let marketing = parsed.marketing.unwrap();
        assert_eq!(marketing.gclid.as_deref(), Some("test_gclid_12345"));
        assert_eq!(marketing.utm_source.as_deref(), Some("google"));

        let identity = parsed.user_identity.unwrap();
        assert_eq!(identity.crm_lead_id.as_deref(), Some("SF-LEAD-99120"));
    }

    #[test]
    fn test_backwards_compatibility_with_tenant_id() {
        let json_payload = r#"{
            "tenant_id": "ten_legacy_brand",
            "session_id": "sess_111",
            "event_id": "evt_222",
            "event_name": "page_view",
            "client_timestamp": "2026-09-20T12:00:00Z"
        }"#;

        let parsed: RawClientPayload = serde_json::from_str(json_payload).unwrap();
        assert_eq!(parsed.tenant_id.as_deref(), Some("ten_legacy_brand"));
        assert_eq!(parsed.account_id, None);
    }
}
