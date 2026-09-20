use super::{AttributedTouchpoint, AttributionEvaluator};
use crate::domain::touchpoint::ConversionPath;

/// Linear Attribution: Distributes conversion credit equally across all journey touchpoints
#[derive(Debug, Default, Clone, Copy)]
pub struct LinearAttributor;

impl AttributionEvaluator for LinearAttributor {
    fn model_name(&self) -> &'static str {
        "linear"
    }

    fn evaluate(&self, path: &ConversionPath) -> Vec<AttributedTouchpoint> {
        let count = path.len();
        if count == 0 {
            return Vec::new();
        }

        let weight = 1.0 / count as f64;
        let total_value = path.conversion.value;
        let attributed_val = total_value * weight;

        path.touchpoints
            .iter()
            .map(|tp| AttributedTouchpoint {
                touchpoint: tp.clone(),
                weight,
                attributed_value: attributed_val,
                model_name: self.model_name(),
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
    fn test_linear_attribution() {
        let conv = Conversion {
            conversion_id: "c1".into(),
            visitor_id: "v1".into(),
            session_id: "s3".into(),
            timestamp: Utc::now(),
            value: 300.0,
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
        let t3 = Touchpoint {
            touchpoint_id: "t3".into(),
            session_id: "s3".into(),
            visitor_id: "v1".into(),
            timestamp: Utc::now(),
            dimensions: MarketingDimensions::default(),
            is_direct: false,
        };

        let path = ConversionPath::new("acc_test".into(), conv, vec![t1, t2, t3]);
        let evaluator = LinearAttributor;
        let attributed = evaluator.evaluate(&path);

        assert_eq!(attributed.len(), 3);
        for item in attributed {
            assert!((item.weight - 0.3333333333333333).abs() < 1e-6);
            assert!((item.attributed_value - 100.0).abs() < 1e-6);
        }
    }
}
