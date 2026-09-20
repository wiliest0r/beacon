use chrono::Utc;
use spin_sdk::http::{Request, Response};
use std::fs;

use crate::account_store::AccountConfigProvider;
use crate::models::{AccountProfile, IngestedParquetEvent, RawClientPayload, UniversalCrmEvent};
use crate::observability::{extract_trace_context, log_structured};
use crate::security::{generate_ephemeral_token, is_origin_allowed, validate_ephemeral_token};

/// Serve tag.js enriched with ephemeral handshake token and account config
pub fn handle_serve_tag(req: &Request, store: &dyn AccountConfigProvider) -> Response {
    let (trace_id, span_id) = extract_trace_context(
        req.header("x-cloud-trace-context").and_then(|h| h.as_str()),
        req.header("traceparent").and_then(|h| h.as_str()),
        std::env::var("GCP_PROJECT").ok().as_deref(),
    );

    // Extract account_id from query parameters (aid, account_id, or fallback tid, tenant_id)
    let query_str = req.query();
    let query_account_id = query_str.split('&').find_map(|param| {
        let mut parts = param.split('=');
        let key = parts.next()?;
        let val = parts.next()?;
        if key == "aid" || key == "account_id" || key == "tid" || key == "tenant_id" {
            Some(val.to_string())
        } else {
            None
        }
    });

    let account_id = req
        .header("x-account-id")
        .or_else(|| req.header("x-tenant-id"))
        .and_then(|h| h.as_str())
        .map(String::from)
        .or(query_account_id)
        .unwrap_or_else(|| "acc_playtests_dev".to_string());

    let profile = store.get_account(&account_id).unwrap_or_else(|_| {
        // Fallback default dev profile if not yet registered in store
        AccountProfile {
            account_id: account_id.clone(),
            name: format!("Auto-Provisioned: {}", account_id),
            status: crate::models::AccountStatus::Active,
            security: crate::models::AccountSecurityConfig {
                allowed_domains: vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                    "*.example.com".to_string(),
                    "playtests.io".to_string(),
                ],
                hmac_secret: "secret-key-poc-32-bytes-long!".to_string(),
                enforce_domain_check: false,
            },
            crm: crate::models::AccountCrmConfig::default(),
            enable_ecommerce: true,
            enable_spa: true,
        }
    });

    let now = Utc::now().timestamp();
    let token = generate_ephemeral_token(&profile.security.hmac_secret, &profile.account_id, now);

    // Read base tag script from WASM bundle files
    let base_script = fs::read_to_string("t.min.js")
        .unwrap_or_else(|_| "console.error('Tag bundle not found');".to_string());

    // Inject initial configuration and token (maintains backwards compatibility with tenantId)
    let injected_config = format!(
        "window.__BEACON_CONFIG__={{accountId:\"{}\",tenantId:\"{}\",appId:\"{}\",token:\"{}\",spa:{},ecommerce:{}}};window.__OP_CONFIG__=window.__BEACON_CONFIG__;",
        profile.account_id, profile.account_id, profile.account_id, token, profile.enable_spa, profile.enable_ecommerce
    );

    let final_js = format!("{}{}", injected_config, base_script);

    log_structured(
        "INFO",
        "serve_tag",
        &format!(
            "Served client tag.js bundle for account {}",
            profile.account_id
        ),
        trace_id.as_deref(),
        span_id.as_deref(),
        Some(serde_json::json!({
            "account_id": profile.account_id,
            "tenant_id": profile.account_id,
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
pub fn handle_collect_event(req: &Request, store: &dyn AccountConfigProvider) -> Response {
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

    // Resolve account_id (with fallbacks: X-Account-ID -> payload.account_id -> X-Tenant-ID -> payload.tenant_id -> app_id)
    let account_id = req
        .header("x-account-id")
        .and_then(|h| h.as_str())
        .map(String::from)
        .or_else(|| payload.account_id.clone())
        .or_else(|| {
            req.header("x-tenant-id")
                .and_then(|h| h.as_str())
                .map(String::from)
        })
        .or_else(|| payload.tenant_id.clone())
        .or_else(|| payload.app_id.clone())
        .unwrap_or_else(|| "acc_playtests_dev".to_string());

    let app_id = payload.app_id.clone().unwrap_or_else(|| account_id.clone());

    let device_id = payload
        .device_id
        .clone()
        .or_else(|| payload.visitor_id.clone())
        .or_else(|| payload.anonymous_id.clone())
        .unwrap_or_else(|| "anonymous_device".to_string());

    let device_fp = payload.device_fp.clone();
    let visitor_id = device_id.clone();
    let anonymous_id = device_id.clone();

    // Query Account Store
    let account_profile = store
        .get_account(&account_id)
        .unwrap_or_else(|_| AccountProfile {
            account_id: account_id.clone(),
            name: format!("Auto-Provisioned: {}", account_id),
            status: crate::models::AccountStatus::Active,
            security: crate::models::AccountSecurityConfig {
                allowed_domains: vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                    "*.example.com".to_string(),
                    "playtests.io".to_string(),
                ],
                hmac_secret: "secret-key-poc-32-bytes-long!".to_string(),
                enforce_domain_check: false,
            },
            crm: crate::models::AccountCrmConfig::default(),
            enable_ecommerce: true,
            enable_spa: true,
        });

    // 3. Security evaluation: determine if event should be quarantined
    let mut is_quarantined = false;
    let mut quarantine_reason = None;

    // Check account status
    if account_profile.status != crate::models::AccountStatus::Active {
        is_quarantined = true;
        quarantine_reason = Some("account_suspended_or_inactive".to_string());
    }

    // Check origin/referer if domain check is enforced
    if !is_quarantined && account_profile.security.enforce_domain_check {
        let domain_source = origin.or(referer);
        if !is_origin_allowed(domain_source, &account_profile.security.allowed_domains) {
            is_quarantined = true;
            quarantine_reason = Some("domain_not_whitelisted".to_string());
        }
    }

    // Check ephemeral HMAC token
    if !is_quarantined {
        if let Some(ref token) = payload.token {
            if let Err(err) =
                validate_ephemeral_token(&account_profile.security.hmac_secret, token, &account_id)
                    .or_else(|_| {
                        validate_ephemeral_token(
                            &account_profile.security.hmac_secret,
                            token,
                            &app_id,
                        )
                    })
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

    // Extract Marketing & AdTech dimensions
    let marketing = payload.marketing.as_ref();
    let gclid = marketing.and_then(|m| m.gclid.clone());
    let fbclid = marketing.and_then(|m| m.fbclid.clone());
    let gbraid = marketing.and_then(|m| m.gbraid.clone());
    let wbraid = marketing.and_then(|m| m.wbraid.clone());
    let msclkid = marketing.and_then(|m| m.msclkid.clone());
    let ttclid = marketing.and_then(|m| m.ttclid.clone());
    let utm_source = marketing.and_then(|m| m.utm_source.clone());
    let utm_medium = marketing.and_then(|m| m.utm_medium.clone());
    let utm_campaign = marketing.and_then(|m| m.utm_campaign.clone());
    let utm_term = marketing.and_then(|m| m.utm_term.clone());
    let utm_content = marketing.and_then(|m| m.utm_content.clone());

    let has_ad_attribution = gclid.is_some()
        || fbclid.is_some()
        || gbraid.is_some()
        || wbraid.is_some()
        || msclkid.is_some()
        || ttclid.is_some();

    let is_conversion = matches!(
        payload.event_name.to_lowercase().as_str(),
        "purchase" | "lead" | "sign_up" | "signup" | "subscribe" | "conversion" | "submit_form"
    );

    // Extract User Identity dimensions
    let user_identity = payload.user_identity.as_ref();
    let hashed_email = user_identity.and_then(|u| u.hashed_email.clone());
    let hashed_phone = user_identity.and_then(|u| u.hashed_phone.clone());
    let crm_lead_id = user_identity.and_then(|u| u.crm_lead_id.clone());

    let now_utc = Utc::now();

    let parquet_record = IngestedParquetEvent {
        account_id: account_id.clone(),
        tenant_id: account_id.clone(), // Kept for 100% backwards compatibility
        app_id: app_id.clone(),
        event_id: payload.event_id.clone(),
        event_name: payload.event_name.clone(),
        client_timestamp: payload.client_timestamp,
        server_timestamp: now_utc,
        is_quarantined,
        quarantine_reason,
        device_id: device_id.clone(),
        device_fp,
        visitor_id: visitor_id.clone(),
        anonymous_id,
        session_id: payload.session_id.clone(),
        user_id: payload.user_id,
        hashed_email: hashed_email.clone(),
        hashed_phone: hashed_phone.clone(),
        crm_lead_id: crm_lead_id.clone(),
        has_ad_attribution,
        is_conversion,
        gclid: gclid.clone(),
        fbclid: fbclid.clone(),
        gbraid,
        wbraid,
        msclkid,
        ttclid,
        utm_source: utm_source.clone(),
        utm_medium: utm_medium.clone(),
        utm_campaign: utm_campaign.clone(),
        utm_term,
        utm_content,
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

    // 5. If conversion or CRM lead ID is present, format universal CRM event for dispatch
    if is_conversion || crm_lead_id.is_some() {
        let _universal_crm_event = UniversalCrmEvent {
            account_id: account_id.clone(),
            event_id: payload.event_id,
            event_name: payload.event_name,
            timestamp: now_utc,
            device_id: device_id.clone(),
            visitor_id,
            session_id: Some(payload.session_id),
            hashed_email,
            hashed_phone,
            crm_lead_id,
            gclid,
            fbclid,
            utm_source,
            utm_medium,
            utm_campaign,
            is_conversion,
            conversion_value: payload.properties.get("value").and_then(|v| v.as_f64()),
            currency: payload
                .properties
                .get("currency")
                .and_then(|c| c.as_str().map(String::from)),
            properties: payload.properties,
        };
    }

    // 6. Emit structured JSON to stdout (for K8s Vector to route to Pub/Sub & Parquet Lakehouse)
    if let Ok(serialized) = serde_json::to_string(&parquet_record) {
        println!("{}", serialized);
    }

    // 7. Emit structured log to stderr for Google Cloud Logging & APM tracing
    if is_quarantined {
        log_structured(
            "WARNING",
            "event_quarantined",
            &format!(
                "Event {} quarantined for account {}: {}",
                parquet_record.event_id,
                parquet_record.account_id,
                parquet_record
                    .quarantine_reason
                    .as_deref()
                    .unwrap_or("unknown")
            ),
            trace_id.as_deref(),
            span_id.as_deref(),
            Some(serde_json::json!({
                "event_id": parquet_record.event_id,
                "event_name": parquet_record.event_name,
                "account_id": parquet_record.account_id,
                "tenant_id": parquet_record.tenant_id,
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
                "Event {} ({}) ingested for account {}",
                parquet_record.event_id, parquet_record.event_name, parquet_record.account_id
            ),
            trace_id.as_deref(),
            span_id.as_deref(),
            Some(serde_json::json!({
                "event_id": parquet_record.event_id,
                "event_name": parquet_record.event_name,
                "account_id": parquet_record.account_id,
                "tenant_id": parquet_record.tenant_id,
                "app_id": parquet_record.app_id,
                "has_ad_attribution": parquet_record.has_ad_attribution,
                "is_conversion": parquet_record.is_conversion,
                "is_quarantined": false,
            })),
        );
    }

    // 8. Return standard 204 No Content response
    Response::builder()
        .status(204)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, X-Account-ID, X-Tenant-ID",
        )
        .build()
}
