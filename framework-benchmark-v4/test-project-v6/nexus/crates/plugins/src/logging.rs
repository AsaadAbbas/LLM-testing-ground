use nexus_core::{Middleware, NexusError, Request, Response};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Request logging middleware.
///
/// Logs each request with method, path, status code, and duration.
/// Also tracks total request count via an atomic counter.
pub struct LoggingMiddleware {
    request_counter: Arc<AtomicU64>,
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            request_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn total_requests(&self) -> u64 {
        self.request_counter.load(Ordering::Relaxed)
    }
}

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn on_request(&self, req: &mut Request) -> Result<Option<Response>, NexusError> {
        let request_id = self.request_counter.fetch_add(1, Ordering::Relaxed) + 1;
        req.extensions.insert(
            "request_id".to_string(),
            serde_json::json!(request_id),
        );
        req.extensions.insert(
            "request_start".to_string(),
            serde_json::json!(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()),
        );

        tracing::info!(
            request_id = request_id,
            method = %req.method,
            path = %req.path,
            "Incoming request"
        );

        Ok(None)
    }

    async fn on_response(&self, req: &Request, resp: &mut Response) -> Result<(), NexusError> {
        let request_id = req.extensions
            .get("request_id")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let start = req.extensions
            .get("request_start")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let duration_ms = (now - start) * 1000.0;

        tracing::info!(
            request_id = request_id,
            status = resp.status,
            duration_ms = duration_ms,
            "Request completed"
        );

        resp.headers.insert("x-request-id".to_string(), request_id.to_string());
        resp.headers.insert("x-response-time".to_string(), format!("{:.2}ms", duration_ms));

        Ok(())
    }

    fn name(&self) -> &str {
        "logging"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core::HttpMethod;

    #[tokio::test]
    async fn test_logging_assigns_request_id() {
        let logger = LoggingMiddleware::new();
        let mut req = Request::new(HttpMethod::Get, "/test");

        logger.on_request(&mut req).await.unwrap();
        assert!(req.extensions.contains_key("request_id"));
        assert_eq!(logger.total_requests(), 1);
    }
}
