use serde::{Deserialize, Serialize};

/// High-level channel groupings according to standard marketing taxonomy
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketingChannel {
    PaidSearch,
    PaidSocial,
    PaidDisplay,
    OrganicSearch,
    OrganicSocial,
    Email,
    Referral,
    Affiliate,
    Direct,
    Custom(String),
}

impl MarketingChannel {
    /// Categorizes marketing channel based on UTM medium, source, and click identifiers
    pub fn infer_from(
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        gclid: Option<&str>,
        fbclid: Option<&str>,
        msclkid: Option<&str>,
        ttclid: Option<&str>,
    ) -> Self {
        if gclid.is_some() || msclkid.is_some() {
            return MarketingChannel::PaidSearch;
        }
        if fbclid.is_some() || ttclid.is_some() {
            return MarketingChannel::PaidSocial;
        }

        let medium = utm_medium.unwrap_or("").to_lowercase();
        let source = utm_source.unwrap_or("").to_lowercase();

        if medium == "cpc" || medium == "ppc" || medium == "paidsearch" {
            MarketingChannel::PaidSearch
        } else if medium == "paidsocial"
            || medium == "paid-social"
            || (medium == "social" && (source.contains("facebook") || source.contains("instagram")))
        {
            MarketingChannel::PaidSocial
        } else if medium == "display" || medium == "banner" || medium == "cpm" {
            MarketingChannel::PaidDisplay
        } else if medium == "organic" || medium == "search" {
            MarketingChannel::OrganicSearch
        } else if medium == "social" {
            MarketingChannel::OrganicSocial
        } else if medium == "email" || medium == "newsletter" {
            MarketingChannel::Email
        } else if medium == "referral" {
            MarketingChannel::Referral
        } else if medium == "affiliate" {
            MarketingChannel::Affiliate
        } else if utm_source.is_none() && utm_medium.is_none() {
            MarketingChannel::Direct
        } else {
            MarketingChannel::Custom(format!("{}/{}", source, medium))
        }
    }
}

/// Normalized marketing dimensions across attribution planes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct MarketingDimensions {
    pub channel: Option<MarketingChannel>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_term: Option<String>,
    pub utm_content: Option<String>,
    pub gclid: Option<String>,
    pub fbclid: Option<String>,
    pub msclkid: Option<String>,
    pub ttclid: Option<String>,
    pub landing_page: Option<String>,
    pub referrer: Option<String>,
    pub country: Option<String>,
    pub device_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_channel_from_gclid() {
        let channel = MarketingChannel::infer_from(
            Some("google"),
            Some("cpc"),
            Some("gclid123"),
            None,
            None,
            None,
        );
        assert_eq!(channel, MarketingChannel::PaidSearch);
    }

    #[test]
    fn test_infer_channel_from_fbclid() {
        let channel = MarketingChannel::infer_from(
            Some("meta"),
            Some("cpm"),
            None,
            Some("fbclid456"),
            None,
            None,
        );
        assert_eq!(channel, MarketingChannel::PaidSocial);
    }

    #[test]
    fn test_infer_direct_channel() {
        let channel = MarketingChannel::infer_from(None, None, None, None, None, None);
        assert_eq!(channel, MarketingChannel::Direct);
    }
}
