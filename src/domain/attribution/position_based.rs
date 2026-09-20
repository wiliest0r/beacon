use super::{AttributedTouchpoint, AttributionEvaluator};
use crate::domain::touchpoint::ConversionPath;

/// Position-Based (U-Shaped) Attribution:
/// - 40% credit to first discovery touchpoint
/// - 40% credit to final conversion touchpoint
/// - 20% credit split equally among middle nurture touchpoints
#[derive(Debug, Default, Clone, Copy)]
pub struct PositionBasedAttributor;

impl AttributionEvaluator for PositionBasedAttributor {
    fn model_name(&self) -> &'static str {
        "position_based_u_shaped"
    }

    fn evaluate(&self, path: &ConversionPath) -> Vec<AttributedTouchpoint> {
        let count = path.len();
        if count == 0 {
            return Vec::new();
        }

        let total_value = path.conversion.value;

        if count == 1 {
            return vec![AttributedTouchpoint {
                touchpoint: path.touchpoints[0].clone(),
                weight: 1.0,
                attributed_value: total_value,
                model_name: self.model_name(),
            }];
        }

        if count == 2 {
            return path
                .touchpoints
                .iter()
                .map(|tp| AttributedTouchpoint {
                    touchpoint: tp.clone(),
                    weight: 0.5,
                    attributed_value: total_value * 0.5,
                    model_name: self.model_name(),
                })
                .collect();
        }

        let middle_count = count - 2;
        let middle_weight = 0.20 / middle_count as f64;

        path.touchpoints
            .iter()
            .enumerate()
            .map(|(idx, tp)| {
                let weight = if idx == 0 || idx == count - 1 {
                    0.40
                } else {
                    middle_weight
                };
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
    fn test_position_based_attribution_three_touchpoints() {
        let conv = Conversion {
            conversion_id: "c1".into(),
            device_id: "v1".into(),
            session_id: "s3".into(),
            timestamp: Utc::now(),
            value: 1000.0,
            currency: "USD".into(),
            order_id: None,
        };
        let t1 = Touchpoint {
            touchpoint_id: "t1".into(),
            session_id: "s1".into(),
            device_id: "v1".into(),
            timestamp: Utc::now(),
            device_fp: None,
            dimensions: MarketingDimensions::default(),
            is_direct: false,
        };
        let t2 = Touchpoint {
            touchpoint_id: "t2".into(),
            session_id: "s2".into(),
            device_id: "v1".into(),
            timestamp: Utc::now(),
            device_fp: None,
            dimensions: MarketingDimensions::default(),
            is_direct: false,
        };
        let t3 = Touchpoint {
            touchpoint_id: "t3".into(),
            session_id: "s3".into(),
            device_id: "v1".into(),
            timestamp: Utc::now(),
            device_fp: None,
            dimensions: MarketingDimensions::default(),
            is_direct: false,
        };

        let path = ConversionPath::new("acc_test".into(), conv, vec![t1, t2, t3]);
        let evaluator = PositionBasedAttributor;
        let attributed = evaluator.evaluate(&path);

        assert_eq!(attributed[0].weight, 0.40);
        assert_eq!(attributed[0].attributed_value, 400.0);

        assert_eq!(attributed[1].weight, 0.20);
        assert_eq!(attributed[1].attributed_value, 200.0);

        assert_eq!(attributed[2].weight, 0.40);
        assert_eq!(attributed[2].attributed_value, 400.0);
    }
}
