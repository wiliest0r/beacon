use super::{AttributedTouchpoint, AttributionEvaluator};
use crate::domain::touchpoint::ConversionPath;

/// Last-Touch Attribution: Attributes 100% of conversion credit to final touchpoint prior to purchase
#[derive(Debug, Default, Clone, Copy)]
pub struct LastTouchAttributor;

impl AttributionEvaluator for LastTouchAttributor {
    fn model_name(&self) -> &'static str {
        "last_touch"
    }

    fn evaluate(&self, path: &ConversionPath) -> Vec<AttributedTouchpoint> {
        let count = path.len();
        if count == 0 {
            return Vec::new();
        }

        let total_value = path.conversion.value;
        path.touchpoints
            .iter()
            .enumerate()
            .map(|(idx, tp)| {
                let weight = if idx == count - 1 { 1.0 } else { 0.0 };
                AttributedTouchpoint {
                    touchpoint: tp.clone(),
                    weight,
                    attributed_value: total_value * weight,
                    model_name: self.model_name(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dimensions::MarketingDimensions;
    use crate::domain::touchpoint::{Conversion, Touchpoint};
    use chrono::Utc;

    #[test]
    fn test_last_touch_attribution() {
        let conv = Conversion {
            conversion_id: "c1".into(),
            visitor_id: "v1".into(),
            session_id: "s2".into(),
            timestamp: Utc::now(),
            value: 150.0,
            currency: "USD".into(),
            order_id: None,
        };
        let t1 = Touchpoint {
            touchpoint_id: "t1".into(),
            session_id: "s1".into(),
            visitor_id: "v1".into(),
            timestamp: Utc::now(),
            dimensions: MarketingDimensions::default(),
            is_direct: false,
        };
        let t2 = Touchpoint {
            touchpoint_id: "t2".into(),
            session_id: "s2".into(),
            visitor_id: "v1".into(),
            timestamp: Utc::now(),
            dimensions: MarketingDimensions::default(),
            is_direct: false,
        };

        let path = ConversionPath::new("acc_test".into(), conv, vec![t1, t2]);
        let evaluator = LastTouchAttributor;
        let attributed = evaluator.evaluate(&path);

        assert_eq!(attributed[0].weight, 0.0);
        assert_eq!(attributed[0].attributed_value, 0.0);
        assert_eq!(attributed[1].weight, 1.0);
        assert_eq!(attributed[1].attributed_value, 150.0);
    }
}
