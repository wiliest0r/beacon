use crate::domain::attribution::{
    AttributedTouchpoint, AttributionEvaluator, FirstTouchAttributor, LastTouchAttributor,
    LinearAttributor, PositionBasedAttributor,
};
use crate::domain::touchpoint::ConversionPath;

/// Application service orchestrating attribution calculation across multiple models
pub struct EvaluateAttributionUseCase;

impl EvaluateAttributionUseCase {
    pub fn evaluate_with_model(
        evaluator: &dyn AttributionEvaluator,
        path: &ConversionPath,
    ) -> Vec<AttributedTouchpoint> {
        evaluator.evaluate(path)
    }

    /// Evaluates conversion path across all canonical attribution models for multi-plane comparison
    pub fn evaluate_all(path: &ConversionPath) -> Vec<(&'static str, Vec<AttributedTouchpoint>)> {
        let first_touch = FirstTouchAttributor;
        let last_touch = LastTouchAttributor;
        let linear = LinearAttributor;
        let position = PositionBasedAttributor;

        vec![
            (first_touch.model_name(), first_touch.evaluate(path)),
            (last_touch.model_name(), last_touch.evaluate(path)),
            (linear.model_name(), linear.evaluate(path)),
            (position.model_name(), position.evaluate(path)),
        ]
    }
}
