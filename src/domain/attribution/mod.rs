use serde::{Deserialize, Serialize};

use super::touchpoint::Touchpoint;

/// Touchpoint with attributed credit weight and monetary revenue allocation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributedTouchpoint {
    pub touchpoint: Touchpoint,
    /// Credit weight between 0.0 and 1.0 (Sum of all weights for a conversion equals 1.0)
    pub weight: f64,
    /// Allocated monetary conversion value: (conversion.value * weight)
    pub attributed_value: f64,
    pub model_name: &'static str,
}

pub trait AttributionEvaluator: Send + Sync {
    /// Evaluates conversion path and produces attributed touchpoints
    fn evaluate(&self, path: &super::touchpoint::ConversionPath) -> Vec<AttributedTouchpoint>;
    /// Canonical name of the attribution algorithm
    fn model_name(&self) -> &'static str;
}

pub mod first_touch;
pub mod last_touch;
pub mod linear;
pub mod position_based;

pub use first_touch::FirstTouchAttributor;
pub use last_touch::LastTouchAttributor;
pub use linear::LinearAttributor;
pub use position_based::PositionBasedAttributor;
