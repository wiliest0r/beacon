use spin_sdk::http::{Method, Request, Response};
use spin_sdk::http_component;

pub mod account_store;
pub mod adapters;
pub mod application;
pub mod domain;
pub mod handlers;
pub mod models;
pub mod observability;
pub mod ports;
pub mod security;

use account_store::InMemoryAccountStore;
use handlers::{handle_collect_event, handle_serve_tag};

/// Main HTTP entrypoint for the Spin WebAssembly component
#[http_component]
fn handle_beacon(req: Request) -> Response {
    let method = req.method();
    let path = req.path();
    let store = InMemoryAccountStore::new();

    match (method, path) {
        (&Method::Get, "/client.js") | (&Method::Get, "/tag.js") => handle_serve_tag(&req, &store),
        (&Method::Options, "/v1/sync") | (&Method::Options, "/v1/collect") => Response::builder()
            .status(204)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "POST, OPTIONS")
            .header(
                "Access-Control-Allow-Headers",
                "Content-Type, X-Tag-ID, X-Measurement-ID",
            )
            .header("Access-Control-Max-Age", "86400")
            .build(),
        (&Method::Post, "/v1/sync") | (&Method::Post, "/v1/collect") => {
            handle_collect_event(&req, &store)
        }
        (&Method::Get, "/healthz") => Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(r#"{"status":"healthy"}"#)
            .build(),
        _ => Response::builder().status(404).body("Not Found").build(),
    }
}
