use spin_sdk::http::{Method, Request, Response};
use spin_sdk::http_component;

pub mod handlers;
pub mod models;
pub mod observability;
pub mod security;

use handlers::{handle_collect_event, handle_serve_tag};
use models::ClientConfig;

fn get_default_config() -> ClientConfig {
    ClientConfig {
        app_id: "APP-XYZ-123".to_string(),
        allowed_domains: vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "*.example.com".to_string(),
        ],
        hmac_secret: "secret-key-poc-32-bytes-long!".to_string(),
        enable_ecommerce: true,
        enable_spa: true,
    }
}

/// Main HTTP entrypoint for the Spin WebAssembly component
#[http_component]
fn handle_beacon(req: Request) -> Response {
    let method = req.method();
    let path = req.path();
    let config = get_default_config();

    match (method, path) {
        (&Method::Get, "/client.js") | (&Method::Get, "/tag.js") => handle_serve_tag(&req, &config),
        (&Method::Options, "/v1/sync") | (&Method::Options, "/v1/collect") => Response::builder()
            .status(204)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .header("Access-Control-Max-Age", "86400")
            .build(),
        (&Method::Post, "/v1/sync") | (&Method::Post, "/v1/collect") => {
            handle_collect_event(&req, &config)
        }
        (&Method::Get, "/healthz") => Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"healthy"}"#)
            .build(),
        _ => Response::builder().status(404).body("Not Found").build(),
    }
}
