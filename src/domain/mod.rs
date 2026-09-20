pub mod attribution;
pub mod dimensions;
pub mod economics;
pub mod touchpoint;

pub use attribution::{
    AttributedTouchpoint, AttributionEvaluator, FirstTouchAttributor, LastTouchAttributor,
    LinearAttributor, PositionBasedAttributor,
};
pub use dimensions::{MarketingChannel, MarketingDimensions};
pub use economics::UnitEconomics;
pub use touchpoint::{Conversion, ConversionPath, Touchpoint};
