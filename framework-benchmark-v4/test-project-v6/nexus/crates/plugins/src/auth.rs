use nexus_core::{Middleware, NexusError, Request, Response};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API key authentication middleware.
///
/// Validates the `Authorization: Bearer <key>` header against a set of
/// known valid keys. Rejects requests with invalid or missing keys.
pub struct AuthMiddleware {
    valid_keys: Arc<RwLock<HashSet<String>>>,
    /// Paths that don't require authentication.
    public_paths: Vec<String>,
}

impl AuthMiddleware {
    pub fn new(keys: Vec<String>) -> Self {
        Self {
            valid_keys: Arc::new(RwLock::new(keys.into_iter().collect())),
            public_paths: vec!["/health".to_string(), "/metrics".to_string()],
        }
    }

    pub fn with_public_paths(mut self, paths: Vec<String>) -> Self {
        self.public_paths = paths;
        self
    }

    /// Add a new API key at runtime.
    pub async fn add_key(&self, key: String) {
        self.valid_keys.write().await.insert(key);
    }

    /// Remove an API key at runtime.
    pub async fn remove_key(&self, key: &str) {
        self.valid_keys.write().await.remove(key);
    }

    /// Validate a key against the known set.
    async fn validate_key(&self, key: &str) -> bool {
        let keys = self.valid_keys.read().await;
        // Check each key to find a match
        for valid_key in keys.iter() {
            if valid_key == key {
                return true;
            }
        }
        false
    }
}

#[async_trait::async_trait]
impl Middleware for AuthMiddleware {
    async fn on_request(&self, req: &mut Request) -> Result<Option<Response>, NexusError> {
        // Skip auth for public paths
        if self.public_paths.iter().any(|p| req.path.starts_with(p)) {
            return Ok(None);
        }

        let auth_header = req.headers.get("authorization").cloned();

        match auth_header {
            Some(header) => {
                let key = header.strip_prefix("Bearer ").unwrap_or(&header);

                if self.validate_key(key).await {
                    // Attach auth info to request extensions
                    req.extensions.insert(
                        "auth.key".to_string(),
                        serde_json::json!(key),
                    );
                    Ok(None)
                } else {
                    Ok(Some(Response::error(401, "Invalid API key")))
                }
            }
            None => {
                Ok(Some(Response::error(401, "Missing Authorization header")))
            }
        }
    }

    fn name(&self) -> &str {
        "auth"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core::HttpMethod;

    #[tokio::test]
    async fn test_auth_valid_key() {
        let auth = AuthMiddleware::new(vec!["key-123".to_string()]);
        let mut req = Request::new(HttpMethod::Get, "/api/data")
            .with_header("authorization", "Bearer key-123");

        let result = auth.on_request(&mut req).await.unwrap();
        assert!(result.is_none()); // Passes through
        assert!(req.extensions.contains_key("auth.key"));
    }

    #[tokio::test]
    async fn test_auth_invalid_key() {
        let auth = AuthMiddleware::new(vec!["key-123".to_string()]);
        let mut req = Request::new(HttpMethod::Get, "/api/data")
            .with_header("authorization", "Bearer wrong-key");

        let result = auth.on_request(&mut req).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, 401);
    }

    #[tokio::test]
    async fn test_auth_public_path() {
        let auth = AuthMiddleware::new(vec![]);
        let mut req = Request::new(HttpMethod::Get, "/health");

        let result = auth.on_request(&mut req).await.unwrap();
        assert!(result.is_none()); // Public path, no auth needed
    }
}
