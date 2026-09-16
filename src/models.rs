use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Client configuration loaded based on domain or app_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub app_id: String,
    pub allowed_domains: Vec<String>,
    pub hmac_secret: String,
    pub enable_ecommerce: bool,
    pub enable_spa: bool,
}

/// Incoming raw payload from client web tag
#[derive(Debug, Deserialize)]
pub struct RawClientPayload {
    pub app_id: String,
    pub anonymous_id: String,
    pub session_id: String,
    pub user_id: Option<String>,
    pub event_id: String,
    pub event_name: String,
    pub client_timestamp: String,
    pub token: Option<String>,
    pub context: Option<RawContext>,
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

/// Fully-enriched, flattened event model ready for Parquet/Blob ingestion
#[derive(Debug, Serialize)]
pub struct IngestedParquetEvent {
    // Primary Partitioning and Identity Keys
    pub event_id: String,
    pub app_id: String,
    pub event_name: String,
    pub client_timestamp: String,
    pub server_timestamp: DateTime<Utc>,

    // Traffic Integrity & Anti-Spam (Partition Key: is_quarantined)
    pub is_quarantined: bool,
    pub quarantine_reason: Option<String>,

    // Identity Dimensions
    pub anonymous_id: String,
    pub session_id: String,
    pub user_id: Option<String>,

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
