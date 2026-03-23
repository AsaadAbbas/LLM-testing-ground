mod routes;
mod middleware;
mod ws;

use chronokv_engine::StorageEngine;
use chronokv_query::QueryEngine;
use chronokv_replication::ReplicationManager;
use chronokv_core::NodeRole;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Application state shared across handlers.
pub struct AppState {
    pub engine: Arc<StorageEngine>,
    pub query_engine: Arc<QueryEngine>,
    pub replication: Arc<ReplicationManager>,
    pub subscriptions: Arc<ws::SubscriptionManager>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let engine = Arc::new(
        StorageEngine::new("/tmp/chronokv_data/wal", 86400.0)
            .expect("failed to initialize storage engine"),
    );

    // Recover from WAL on startup
    match engine.recover().await {
        Ok(count) => tracing::info!("Recovered {} entries from WAL", count),
        Err(e) => tracing::warn!("WAL recovery failed: {}", e),
    }

    let query_engine = Arc::new(QueryEngine::new(engine.clone()));
    let replication = Arc::new(
        ReplicationManager::new("node-1".to_string(), NodeRole::Leader),
    );
    let subscriptions = Arc::new(ws::SubscriptionManager::new());

    let state = Arc::new(AppState {
        engine,
        query_engine,
        replication,
        subscriptions,
    });

    let app = routes::create_router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("ChronoKV server listening on port 3000");
    axum::serve(listener, app).await.unwrap();
}
