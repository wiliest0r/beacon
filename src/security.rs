use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const TOKEN_EXPIRATION_SECONDS: i64 = 1800; // 30 minutes validity window

/// Generate ephemeral HMAC-SHA256 token for client handshake
pub fn generate_ephemeral_token(secret: &str, app_id: &str, timestamp: i64) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");

    let message = format!("{}:{}", app_id, timestamp);
    mac.update(message.as_bytes());

    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}:{}:{}", app_id, timestamp, signature)
}

/// Validate incoming client token against expiration and cryptographic signature
pub fn validate_ephemeral_token(
    secret: &str,
    token_str: &str,
    expected_app_id: &str,
) -> Result<(), &'static str> {
    let parts: Vec<&str> = token_str.split(':').collect();
    if parts.len() != 3 {
        return Err("Malformed token format");
    }

    let token_app_id = parts[0];
    let timestamp_str = parts[1];
    let signature = parts[2];

    if token_app_id != expected_app_id {
        return Err("Token app_id mismatch");
    }

    let token_time: i64 = timestamp_str
        .parse()
        .map_err(|_| "Invalid timestamp in token")?;
    let now = Utc::now().timestamp();

    // Check expiration window (+/- 5 mins drift allowed)
    if now - token_time > TOKEN_EXPIRATION_SECONDS || token_time - now > 300 {
        return Err("Token expired or invalid timestamp drift");
    }

    // Verify signature
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    let message = format!("{}:{}", token_app_id, timestamp_str);
    mac.update(message.as_bytes());

    let expected_sig = hex::encode(mac.finalize().into_bytes());
    if expected_sig != signature {
        return Err("Cryptographic signature mismatch");
    }

    Ok(())
}

/// Check if origin/referrer header matches configured domain whitelist
pub fn is_origin_allowed(origin_or_referer: Option<&str>, allowed_domains: &[String]) -> bool {
    guard_domain(origin_or_referer, allowed_domains)
}

fn guard_domain(raw_url: Option<&str>, allowed: &[String]) -> bool {
    let Some(url) = raw_url else { return false };

    // Extract host portion
    let host = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");

    allowed.iter().any(|pattern| {
        if let Some(suffix) = pattern.strip_prefix("*.") {
            host == suffix || host.ends_with(&format!(".{}", suffix))
        } else {
            host == pattern
        }
    })
}
