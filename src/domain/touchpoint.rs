use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::dimensions::MarketingDimensions;

/// Individual touchpoint (impression, click, session visit) along customer conversion journey
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Touchpoint {
    pub touchpoint_id: String,
    pub session_id: String,
    pub device_id: String,
    pub device_fp: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub dimensions: MarketingDimensions,
    pub is_direct: bool,
}

/// Conversion event representing transaction, lead generation, or signup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversion {
    pub conversion_id: String,
    pub device_id: String,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub currency: String,
    pub order_id: Option<String>,
}

/// Ordered collection of touchpoints leading up to a conversion
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConversionPath {
    pub account_id: String,
    pub conversion: Conversion,
    pub touchpoints: Vec<Touchpoint>,
}

impl ConversionPath {
    pub fn new(
        account_id: String,
        conversion: Conversion,
        mut touchpoints: Vec<Touchpoint>,
    ) -> Self {
        touchpoints.sort_by_key(|t| t.timestamp);
        Self {
            account_id,
            conversion,
            touchpoints,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.touchpoints.is_empty()
    }

    pub fn len(&self) -> usize {
        self.touchpoints.len()
    }
}
