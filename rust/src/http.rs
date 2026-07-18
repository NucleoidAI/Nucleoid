use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

use crate::runtime::{NucleoidRuntime, ProcessOptions};

pub type AppState = Arc<Mutex<NucleoidRuntime>>;

/// Routes mirror the subset of `src/routes/terminal.js` this prototype
/// covers: run a statement, and inspect recent history / the dependency
/// graph. Clustering, OpenAPI generation and the native/extensions plugin
/// loading in `src/process.js` are out of scope for this port.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", post(run_statement))
        .route("/logs", get(logs))
        .route("/graph", get(graph))
        .with_state(state)
}

async fn run_statement(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> (StatusCode, Json<crate::runtime::ProcessDetails>) {
    let source = String::from_utf8_lossy(&body).to_string();

    let mut runtime = state.lock().expect("runtime mutex poisoned");
    let details = runtime.process(&source, ProcessOptions::default());

    let status = if details.error.is_some() {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::OK
    };

    (status, Json(details))
}

#[derive(Deserialize)]
struct LogsParams {
    n: Option<usize>,
}

async fn logs(
    State(state): State<AppState>,
    Query(params): Query<LogsParams>,
) -> Json<Vec<crate::datastore::Record>> {
    let runtime = state.lock().expect("runtime mutex poisoned");
    Json(runtime.tail(params.n.unwrap_or(20)))
}

async fn graph(State(state): State<AppState>) -> Json<crate::graph::GraphSnapshot> {
    let runtime = state.lock().expect("runtime mutex poisoned");
    Json(runtime.graph_snapshot())
}
