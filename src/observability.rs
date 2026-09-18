use serde_json::json;

/// Extracts trace context from standard W3C `traceparent` or GCP `x-cloud-trace-context` header.
pub fn extract_trace_context(
    x_cloud_trace_context: Option<&str>,
    traceparent: Option<&str>,
    project_id: Option<&str>,
) -> (Option<String>, Option<String>) {
    // 1. Check Google Cloud Trace header: `TRACE_ID/SPAN_ID;o=TRACE_TRUE`
    if let Some(header_val) = x_cloud_trace_context {
        let parts: Vec<&str> = header_val.split(';').collect();
        let trace_and_span: Vec<&str> = parts[0].split('/').collect();
        let trace_id = trace_and_span.first().copied();
        let span_id = trace_and_span.get(1).copied();

        let formatted_trace = trace_id.map(|t| {
            if let Some(pid) = project_id {
                format!("projects/{}/traces/{}", pid, t)
            } else {
                t.to_string()
            }
        });

        return (formatted_trace, span_id.map(String::from));
    }

    // 2. Fallback to W3C `traceparent`: `version-trace_id-parent_id-trace_flags`
    if let Some(header_val) = traceparent {
        let parts: Vec<&str> = header_val.split('-').collect();
        if parts.len() >= 3 {
            let trace_id = parts[1];
            let span_id = parts[2];
            let formatted_trace = if let Some(pid) = project_id {
                format!("projects/{}/traces/{}", pid, trace_id)
            } else {
                trace_id.to_string()
            };
            return (Some(formatted_trace), Some(span_id.to_string()));
        }
    }

    (None, None)
}

/// Emit structured JSON log directly to stderr for native Google Cloud Logging ingestion.
pub fn log_structured(
    severity: &str,
    action: &str,
    message: &str,
    trace_id: Option<&str>,
    span_id: Option<&str>,
    extra_fields: Option<serde_json::Value>,
) {
    let mut log_entry = json!({
        "severity": severity,
        "component": "beacon-server",
        "action": action,
        "message": message,
    });

    if let Some(trace) = trace_id {
        log_entry["logging.googleapis.com/trace"] = json!(trace);
    }

    if let Some(span) = span_id {
        log_entry["logging.googleapis.com/spanId"] = json!(span);
    }

    if let Some(serde_json::Value::Object(map)) = extra_fields {
        if let Some(obj) = log_entry.as_object_mut() {
            for (k, v) in map {
                obj.insert(k, v);
            }
        }
    }

    eprintln!("{}", log_entry);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_trace_context_gcp() {
        let gcp_header = "105445aa7843bc8bf206b120001000/1;o=1";
        let (trace, span) = extract_trace_context(Some(gcp_header), None, Some("playtests-dev"));
        assert_eq!(
            trace,
            Some("projects/playtests-dev/traces/105445aa7843bc8bf206b120001000".to_string())
        );
        assert_eq!(span, Some("1".to_string()));
    }

    #[test]
    fn test_extract_trace_context_w3c() {
        let w3c_header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let (trace, span) = extract_trace_context(None, Some(w3c_header), None);
        assert_eq!(
            trace,
            Some("4bf92f3577b34da6a3ce929d0e0e4736".to_string())
        );
        assert_eq!(span, Some("00f067aa0ba902b7".to_string()));
    }

    #[test]
    fn test_extract_trace_context_empty() {
        let (trace, span) = extract_trace_context(None, None, None);
        assert!(trace.is_none());
        assert!(span.is_none());
    }
}
