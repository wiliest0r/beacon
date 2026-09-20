use serde::{Deserialize, Serialize};

/// High-level unit economics calculations for marketing campaigns and channels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitEconomics {
    pub spend: f64,
    pub revenue: f64,
    pub conversions: u64,
    pub roas: f64,
    pub cpa: f64,
    pub average_order_value: f64,
}

impl UnitEconomics {
    /// Computes unit economics safely avoiding division by zero
    pub fn calculate(spend: f64, revenue: f64, conversions: u64) -> Self {
        let roas = if spend > 0.0 { revenue / spend } else { 0.0 };
        let cpa = if conversions > 0 {
            spend / (conversions as f64)
        } else {
            spend
        };
        let average_order_value = if conversions > 0 {
            revenue / (conversions as f64)
        } else {
            0.0
        };

        Self {
            spend,
            revenue,
            conversions,
            roas,
            cpa,
            average_order_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_economics_calculation() {
        let eco = UnitEconomics::calculate(1000.0, 4500.0, 20);
        assert_eq!(eco.roas, 4.5);
        assert_eq!(eco.cpa, 50.0);
        assert_eq!(eco.average_order_value, 225.0);
    }
}
