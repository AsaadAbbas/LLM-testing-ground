use axum::{extract::Path, extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use cratesync_core::*;
use cratesync_resolver::Resolver;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    packages: Arc<RwLock<HashMap<String, Package>>>,
    resolver: Arc<Resolver>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        packages: Arc::new(RwLock::new(HashMap::new())),
        resolver: Arc::new(Resolver::new()),
    };

    let app = Router::new()
        .route("/packages", get(list_packages))
        .route("/packages/:name", get(get_package))
        .route("/sync", post(sync_packages))
        .route("/resolve", post(resolve_deps))
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3100".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("CrateSync server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "cratesync",
        "version": "0.1.0"
    }))
}

async fn list_packages(State(state): State<AppState>) -> Json<Vec<String>> {
    let packages = state.packages.read().await;
    let names: Vec<String> = packages.keys().cloned().collect();
    Json(names)
}

async fn get_package(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Package>, StatusCode> {
    let packages = state.packages.read().await;
    match packages.get(&name) {
        Some(pkg) => Ok(Json(pkg.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn sync_packages(
    State(state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> Json<SyncResponse> {
    let mut synced = Vec::new();
    let failed = Vec::new();

    // In a real implementation, this would fetch from an upstream registry
    // For now, just acknowledge the request
    for pkg_name in &req.packages {
        synced.push(pkg_name.clone());
    }

    Json(SyncResponse { synced, failed })
}

async fn resolve_deps(
    State(state): State<AppState>,
    Json(req): Json<ResolveRequest>,
) -> Result<Json<ResolveResponse>, StatusCode> {
    match state.resolver.resolve(&req.root_dependencies).await {
        Ok(lockfile) => Ok(Json(ResolveResponse { lockfile })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
