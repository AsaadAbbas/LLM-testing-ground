use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chronokv_core::{
    now_timestamp, GetResponse, PutRequest, QueryRequest, QueryResponse,
    Timestamp,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// API response wrapper for consistent error handling.
#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Json<ApiResponse<T>> {
        Json(ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        })
    }

    fn err(msg: &str) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(msg.to_string()),
            }),
        )
    }
}

/// Query parameters for time-range filtering.
#[derive(Deserialize)]
pub struct TimeRangeParams {
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub limit: Option<usize>,
    pub prefix: Option<String>,
}

/// Response for put operations.
#[derive(Serialize)]
pub struct PutResponse {
    pub key: String,
    pub timestamp: Timestamp,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/v1/kv/{key}", get(get_key))
        .route("/api/v1/kv/{key}", post(put_key))
        .route("/api/v1/kv/{key}", delete(delete_key))
        .route("/api/v1/query", get(query_range))
        .route("/api/v1/health", get(health_check))
        .with_state(state)
}

/// GET /api/v1/kv/:key — Get the latest value for a key.
async fn get_key(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<ApiResponse<GetResponse>>, (StatusCode, Json<ApiResponse<GetResponse>>)> {
    match state.query_engine.get_latest(&key).await {
        Ok(Some(entry)) => Ok(ApiResponse::ok(entry)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some(format!("key not found: {}", key)),
            }),
        )),
        Err(e) => Err(ApiResponse::err(&e.to_string())),
    }
}

/// POST /api/v1/kv/:key — Put a value for a key.
async fn put_key(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(body): Json<PutRequest>,
) -> Result<Json<ApiResponse<PutResponse>>, (StatusCode, Json<ApiResponse<PutResponse>>)> {
    let timestamp = now_timestamp();

    match state.engine.put(key.clone(), body.value, timestamp).await {
        Ok(()) => {
            // Replicate to followers
            if state.replication.is_leader().await {
                let entry = chronokv_core::Entry::put(key.clone(), vec![], timestamp);
                let _ = state.replication.commit_entry(entry).await;
            }

            // Notify subscribers
            state.subscriptions.notify(&key, timestamp).await;

            Ok(ApiResponse::ok(PutResponse {
                key,
                timestamp, // Serialized as seconds since epoch (f64)
            }))
        }
        Err(e) => Err(ApiResponse::err(&e.to_string())),
    }
}

/// DELETE /api/v1/kv/:key — Delete a key.
async fn delete_key(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<ApiResponse<PutResponse>>, (StatusCode, Json<ApiResponse<PutResponse>>)> {
    let timestamp = now_timestamp();

    match state.engine.delete(&key, timestamp).await {
        Ok(()) => Ok(ApiResponse::ok(PutResponse { key, timestamp })),
        Err(e) => Err(ApiResponse::err(&e.to_string())),
    }
}

/// GET /api/v1/query — Query entries with time range and prefix filtering.
async fn query_range(
    State(state): State<Arc<AppState>>,
    Query(params): Query<TimeRangeParams>,
) -> Result<Json<ApiResponse<QueryResponse>>, (StatusCode, Json<ApiResponse<QueryResponse>>)> {
    let request = QueryRequest {
        key_prefix: params.prefix,
        time_range: match (params.start, params.end) {
            (Some(s), Some(e)) => Some((s, e)),
            _ => None,
        },
        limit: params.limit,
    };

    match state.query_engine.query(&request).await {
        Ok(response) => Ok(ApiResponse::ok(response)),
        Err(e) => Err(ApiResponse::err(&e.to_string())),
    }
}

/// GET /api/v1/health — Health check endpoint.
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "0.1.0"
    }))
}

#[cfg(test)]
mod tests {
    // Route-level tests would require full server setup
    // Integration tests should be added separately
}
