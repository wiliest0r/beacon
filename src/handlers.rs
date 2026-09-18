use chrono::Utc;
use spin_sdk::http::{Request, Response};
use std::fs;

use crate::models::{ClientConfig, IngestedParquetEvent, RawClientPayload};
use crate::observability::{extract_trace_context, log_structured};
use crate::security::{generate_ephemeral_token, is_origin_allowed, validate_ephemeral_token};

/// Serve tag.js enriched with ephemeral handshake token and client config
pub fn handle_serve_tag(req: &Request, config: &ClientConfig) -> Response {
    let (trace_id, span_id) = extract_trace_context(
        req.header("x-cloud-trace-context").and_then(|h| h.as_str()),
        req.header("traceparent").and_then(|h| h.as_str()),
        std::env::var("GCP_PROJECT").ok().as_deref(),
    );

    let now = Utc::now().timestamp();
    let token = generate_ephemeral_token(&config.hmac_secret, &config.app_id, now);

    // Read base tag script from WASM bundle files
    let base_script = fs::read_to_string("t.min.js")
        .unwrap_or_else(|_| "console.error('Tag bundle not found');".to_string());

    // Inject initial configuration and token
    let injected_config = format!(
        "window.__OP_CONFIG__={{appId:\"{}\",token:\"{}\",spa:{},ecommerce:{}}};",
        config.app_id, token, config.enable_spa, config.enable_ecommerce
    );

    let final_js = format!("{}{}", injected_config, base_script);

    log_structured(
        "INFO",
        "serve_tag",
        &format!("Served client tag.js bundle for app {}", config.app_id),
        trace_id.as_deref(),
        span_id.as_deref(),
        Some(serde_json::json!({
            "app_id": config.app_id,
        })),
    );

    Response::builder()
        .status(200)
        .header("Content-Type", "application/javascript; charset=utf-8")
        .header("Cache-Control", "public, max-age=1800")
        .body(final_js)
        .build()
}

/// Ingest and validate telemetry event, outputting Parquet-ready record
pub fn handle_collect_event(req: &Request, config: &ClientConfig) -> Response {
    // 1. Extract network metadata and trace context
    let (trace_id, span_id) = extract_trace_context(
        req.header("x-cloud-trace-context").and_then(|h| h.as_str()),
        req.header("traceparent").and_then(|h| h.as_str()),
        std::env::var("GCP_PROJECT").ok().as_deref(),
    );

    let origin = req.header("origin").and_then(|h| h.as_str());
    let referer = req.header("referer").and_then(|h| h.as_str());
    let user_agent = req
        .header("user-agent")
        .and_then(|h| h.as_str())
        .map(String::from);
    let client_ip = req
        .header("x-forwarded-for")
        .and_then(|h| h.as_str())
        .map(String::from);

    // 2. Parse payload body
    let body_bytes = req.body();
    let Ok(payload) = serde_json::from_slice::<RawClientPayload>(body_bytes) else {
        log_structured(
            "WARNING",
            "bad_request",
            "Failed to deserialize JSON body for /v1/sync",
            trace_id.as_deref(),
            span_id.as_deref(),
            None,
        );

        return Response::builder()
            .status(400)
            .header("Access-Control-Allow-Origin", "*")
            .body(r#"{"error":"invalid_json"}"#)
            .build();
    };

    // 3. Security evaluation: determine if event should be quarantined
    let mut is_quarantined = false;
    let mut quarantine_reason = None;

    // Check origin/referer
    let domain_source = origin.or(referer);
    if !is_origin_allowed(domain_source, &config.allowed_domains) {
        is_quarantined = true;
        quarantine_reason = Some("domain_not_whitelisted".to_string());
    }

    // Check ephemeral HMAC token
    if !is_quarantined {
        if let Some(ref token) = payload.token {
            if let Err(err) = validate_ephemeral_token(&config.hmac_secret, token, &payload.app_id)
            {
                is_quarantined = true;
                quarantine_reason = Some(format!("token_validation_failed: {}", err));
            }
        } else {
            is_quarantined = true;
            quarantine_reason = Some("missing_handshake_token".to_string());
        }
    }

    // 4. Construct Parquet-ready normalized event record
    let context = payload.context.as_ref();
    let page = context.and_then(|c| c.page.as_ref());
    let screen = context.and_then(|c| c.screen.as_ref());

    let parquet_record = IngestedParquetEvent {
        event_id: payload.event_id,
        app_id: payload.app_id,
        event_name: payload.event_name,
        client_timestamp: payload.client_timestamp,
        server_timestamp: Utc::now(),
        is_quarantined,
        quarantine_reason,
        anonymous_id: payload.anonymous_id,
        session_id: payload.session_id,
        user_id: payload.user_id,
        page_url: page.and_then(|p| p.url.clone()),
        page_path: page.and_then(|p| p.path.clone()),
        page_title: page.and_then(|p| p.title.clone()),
        page_referrer: page.and_then(|p| p.referrer.clone()),
        screen_width: screen.and_then(|s| s.width),
        screen_height: screen.and_then(|s| s.height),
        locale: context.and_then(|c| c.locale.clone()),
        timezone: context.and_then(|c| c.timezone.clone()),
        client_ip,
        user_agent,
        custom_properties_json: serde_json::to_string(&payload.properties).unwrap_or_default(),
    };

    // 5. Emit structured JSON to stdout (for K8s vector/fluent-bit to batch into Parquet/GCS Lakehouse)
    if let Ok(serialized) = serde_json::to_string(&parquet_record) {
        println!("{}", serialized);
    }

    // 6. Emit structured log to stderr for Google Cloud Logging & APM tracing
    if is_quarantined {
        log_structured(
            "WARNING",
            "event_quarantined",
            &format!(
                "Event {} quarantined for app {}: {}",
                parquet_record.event_id,
                parquet_record.app_id,
                parquet_record.quarantine_reason.as_deref().unwrap_or("unknown")
            ),
            trace_id.as_deref(),
            span_id.as_deref(),
            Some(serde_json::json!({
                "event_id": parquet_record.event_id,
                "event_name": parquet_record.event_name,
                "app_id": parquet_record.app_id,
                "is_quarantined": true,
                "quarantine_reason": parquet_record.quarantine_reason,
            })),
        );
    } else {
        log_structured(
            "INFO",
            "event_ingested",
            &format!(
                "Event {} ({}) ingested successfully",
                parquet_record.event_id,
                parquet_record.event_name
            ),
            trace_id.as_deref(),
            span_id.as_deref(),
            Some(serde_json::json!({
                "event_id": parquet_record.event_id,
                "event_name": parquet_record.event_name,
                "app_id": parquet_record.app_id,
                "is_quarantined": false,
            })),
        );
    }

    // 7. Return standard 204 No Content response
    Response::builder()
        .status(204)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .build()
}
