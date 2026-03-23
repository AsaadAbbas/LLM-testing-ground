use nexus_core::{Middleware, NexusError, Request, Response};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Token bucket rate limiter middleware.
///
/// Tracks request counts per client (identified by IP or API key).
/// When a client exceeds the rate limit, requests are rejected with 429.
pub struct RateLimitMiddleware {
    buckets: Arc<Mutex<HashMap<String, TokenBucket>>>,
    max_tokens: u32,
    refill_rate: u32, // tokens per second
    window: Duration,
}

struct TokenBucket {
    tokens: u32,
    last_refill: Instant,
}

impl RateLimitMiddleware {
    pub fn new(max_tokens: u32, refill_rate: u32) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens,
            refill_rate,
            window: Duration::from_secs(60),
        }
    }

    pub fn with_window(mut self, window: Duration) -> Self {
        self.window = window;
        self
    }

    fn get_client_id(req: &Request) -> String {
        // Use API key if available, otherwise use a default
        req.extensions
            .get("auth.key")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous")
            .to_string()
    }
}

#[async_trait::async_trait]
impl Middleware for RateLimitMiddleware {
    async fn on_request(&self, req: &mut Request) -> Result<Option<Response>, NexusError> {
        let client_id = Self::get_client_id(req);
        let mut buckets = self.buckets.lock().await;

        let bucket = buckets.entry(client_id).or_insert(TokenBucket {
            tokens: self.max_tokens,
            last_refill: Instant::now(),
        });

        // Refill tokens based on elapsed time
        let elapsed = bucket.last_refill.elapsed();
        let elapsed_secs = elapsed.as_secs() as u32;
        let new_tokens = self.refill_rate * elapsed_secs;
        bucket.tokens = std::cmp::min(bucket.tokens + new_tokens, self.max_tokens);
        bucket.last_refill = Instant::now();

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            // Add remaining quota to response headers
            req.extensions.insert(
                "rate_limit.remaining".to_string(),
                serde_json::json!(bucket.tokens),
            );
            Ok(None)
        } else {
            let mut resp = Response::error(429, "Rate limit exceeded");
            resp.headers.insert(
                "retry-after".to_string(),
                format!("{}", self.window.as_secs()),
            );
            Ok(Some(resp))
        }
    }

    async fn on_response(&self, _req: &Request, resp: &mut Response) -> Result<(), NexusError> {
        resp.headers.insert(
            "x-rate-limit-limit".to_string(),
            self.max_tokens.to_string(),
        );
        Ok(())
    }

    fn name(&self) -> &str {
        "rate-limit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core::HttpMethod;

    #[tokio::test]
    async fn test_rate_limit_allows_within_limit() {
        let limiter = RateLimitMiddleware::new(5, 1);

        for _ in 0..5 {
            let mut req = Request::new(HttpMethod::Get, "/api/data");
            let result = limiter.on_request(&mut req).await.unwrap();
            assert!(result.is_none());
        }
    }

    #[tokio::test]
    async fn test_rate_limit_rejects_over_limit() {
        let limiter = RateLimitMiddleware::new(2, 0); // No refill

        // Exhaust tokens
        for _ in 0..2 {
            let mut req = Request::new(HttpMethod::Get, "/api/data");
            limiter.on_request(&mut req).await.unwrap();
        }

        // Next should be rejected
        let mut req = Request::new(HttpMethod::Get, "/api/data");
        let result = limiter.on_request(&mut req).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, 429);
    }
}
