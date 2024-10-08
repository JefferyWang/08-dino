mod config;
mod engine;
mod error;
mod middleware;
mod pool;
mod router;

use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::{request::Parts, Response},
    response::IntoResponse,
    routing::any,
    Router,
};
use dashmap::DashMap;
use indexmap::IndexMap;
use matchit::Match;
use middleware::ServerTimeLayer;
use tokio::net::TcpListener;
use tracing::info;

use anyhow::{anyhow, Result};

pub use config::*;
pub use engine::*;
pub use error::*;
pub use pool::*;
pub use router::*;

type ProjectRoutes = IndexMap<String, Vec<ProjectRoute>>;

#[derive(Clone)]
pub struct AppState {
    // key is hostname
    router: DashMap<String, SwappalbeAppRouter>,
    worker_pool: Arc<WorkerPool>,
}

#[derive(Clone)]
pub struct TenentRouter {
    host: String,
    router: SwappalbeAppRouter,
}

pub async fn start_server(port: u16, router: Vec<TenentRouter>) -> Result<()> {
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;
    let map = DashMap::new();
    for TenentRouter { host, router } in router {
        map.insert(host, router);
    }

    let worker_pool = WorkerPool::new(12);

    let state = AppState::new(map, worker_pool);
    let app = Router::new()
        .route("/*path", any(handler))
        .layer(ServerTimeLayer)
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
    Query(query): Query<HashMap<String, String>>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    let router = get_router_by_host(host, state.clone())?;
    let matched = router.match_it(parts.method.clone(), parts.uri.path())?;
    let req: Req = assemble_req(&matched, &parts, query, body)?;
    let handler = matched.value;
    // TODO: build a worker pool, and send req via mpsc channel and get res from oneshot channel
    // but if code changed we need to recreate the worker pool
    // let worker = JsWorker::try_new(&router.code)?;
    // let res = worker.run(handler, req)?;

    let (sender, receiver) = oneshot::channel();
    let params = Params::new(router.code.clone(), handler.to_string(), req, sender);
    state
        .worker_pool
        .sender
        .send(params)
        .map_err(|e| anyhow!("send failed, {:?}", e));

    match receiver.recv() {
        Ok(res) => Ok(Response::from(res?)),
        Err(e) => Err(anyhow!("{:?}", e).into()),
    }
}

impl AppState {
    pub fn new(router: DashMap<String, SwappalbeAppRouter>, worker_pool: WorkerPool) -> Self {
        Self {
            router,
            worker_pool: Arc::new(worker_pool),
        }
    }
}

fn get_router_by_host(mut host: String, state: AppState) -> Result<AppRouter, AppError> {
    let _ = host.split_off(host.find(':').unwrap_or(host.len()));
    info!("host: {:?}", host);

    let router = state
        .router
        .get(&host)
        .ok_or(AppError::HostNotFound(host.to_string()))?
        .load();

    Ok(router)
}

impl TenentRouter {
    pub fn new(host: impl Into<String>, router: SwappalbeAppRouter) -> Self {
        Self {
            host: host.into(),
            router,
        }
    }
}

fn assemble_req(
    matched: &Match<&str>,
    parts: &Parts,
    query: HashMap<String, String>,
    body: Option<Bytes>,
) -> Result<Req, AppError> {
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    // convert request data into Req
    let headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap().to_string()))
        .collect();
    let body = body.and_then(|v| String::from_utf8(v.into()).ok());

    let req = Req::builder()
        .method(parts.method.to_string())
        .url(parts.uri.to_string())
        .query(query)
        .params(params)
        .headers(headers)
        .body(body)
        .build();

    Ok(req)
}
