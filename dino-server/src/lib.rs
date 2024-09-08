mod config;
mod error;
mod router;

use std::collections::HashMap;

use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::request::Parts,
    response::IntoResponse,
    routing::any,
    Json, Router,
};
use dashmap::DashMap;
use indexmap::IndexMap;
use tokio::net::TcpListener;

use anyhow::Result;

pub use config::*;
pub use error::*;
pub use router::*;
use tracing::info;

type ProjectRoutes = IndexMap<String, Vec<ProjectRoute>>;

#[derive(Clone)]
pub struct AppState {
    // key is hostname
    router: DashMap<String, SwappalbeAppRouter>,
}

pub async fn start_server(port: u16, router: DashMap<String, SwappalbeAppRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    let state = AppState::new(router);
    let app = Router::new()
        .route("/*path", any(handler))
        .with_state(state);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

#[allow(unused)]
// we only support JSON requests and return JSON responses
async fn handler(
    State(state): State<AppState>,
    parts: Parts,
    Host(mut host): Host,
    Query(query): Query<serde_json::Value>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    // get router from state
    info!("host: {:?}", host);
    host.split_off(host.find(':').unwrap_or(host.len()));
    let router = state
        .router
        .get(&host)
        .ok_or(AppError::HostNotFound(host.to_string()))?
        .load();
    // match router with parts.path get a handler
    let matched = router.match_it(parts.method, parts.uri.path())?;
    let handler = matched.value;
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    // convert request data into Req and call handler with a js runtime
    // convert Req into response and return
    let body = if let Some(body) = body {
        if body.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&body)?
        }
    } else {
        serde_json::Value::Null
    };
    Ok(Json(serde_json::json!({
        "handler": handler,
        "params": params,
        "query": query,
        "body": body
    })))
}

impl AppState {
    pub fn new(router: DashMap<String, SwappalbeAppRouter>) -> Self {
        Self { router }
    }
}
