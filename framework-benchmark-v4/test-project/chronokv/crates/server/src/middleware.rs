use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

/// Simple API key authentication middleware.
pub struct AuthState {
    pub api_keys: Vec<String>,
}

impl AuthState {
    pub fn new(keys: Vec<String>) -> Self {
        Self { api_keys: keys }
    }

    pub fn validate(&self, key: &str) -> bool {
        self.api_keys.contains(&key.to_string())
    }
}

/// Rate limiter using a token bucket algorithm.
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    tokens_per_second: u32,
    max_tokens: u32,
}

struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(tokens_per_second: u32, max_tokens: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            tokens_per_second,
            max_tokens,
        }
    }

    /// Try to consume one token for the given client identifier.
    /// Returns true if the request should be allowed.
    pub async fn try_acquire(&self, client_id: &str) -> bool {
        let mut buckets = self.buckets.write().await;

        let bucket = buckets.entry(client_id.to_string()).or_insert(TokenBucket {
            tokens: self.max_tokens as f64,
            last_refill: Instant::now(),
        });

        // Refill tokens based on elapsed time
        let elapsed = bucket.last_refill.elapsed().as_secs_f64();
        let refill = self.tokens_per_second as f64 * elapsed;
        bucket.tokens = (bucket.tokens + refill).min(self.max_tokens as f64);
        bucket.last_refill = Instant::now();

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_validation() {
        let auth = AuthState::new(vec!["key-123".to_string(), "key-456".to_string()]);
        assert!(auth.validate("key-123"));
        assert!(!auth.validate("invalid"));
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10, 10);

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(limiter.try_acquire("client1").await);
        }

        // 11th should fail (bucket empty)
        assert!(!limiter.try_acquire("client1").await);
    }
}
