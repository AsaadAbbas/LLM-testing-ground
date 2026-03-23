use nexus_core::{HttpMethod, Middleware, NexusError, Request, Response};

/// CORS (Cross-Origin Resource Sharing) middleware.
///
/// Handles preflight OPTIONS requests and adds CORS headers to all responses.
pub struct CorsMiddleware {
    allowed_origins: Vec<String>,
    allowed_methods: Vec<HttpMethod>,
    allowed_headers: Vec<String>,
    max_age: u32,
}

impl CorsMiddleware {
    pub fn permissive() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                HttpMethod::Get,
                HttpMethod::Post,
                HttpMethod::Put,
                HttpMethod::Delete,
                HttpMethod::Options,
            ],
            allowed_headers: vec![
                "content-type".to_string(),
                "authorization".to_string(),
            ],
            max_age: 86400,
        }
    }

    pub fn with_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.allowed_origins.iter().any(|o| o == "*" || o == origin)
    }
}

#[async_trait::async_trait]
impl Middleware for CorsMiddleware {
    async fn on_request(&self, req: &mut Request) -> Result<Option<Response>, NexusError> {
        // Handle preflight OPTIONS request
        if req.method == HttpMethod::Options {
            let origin = req.headers.get("origin").cloned().unwrap_or_default();

            if self.is_origin_allowed(&origin) {
                let mut resp = Response::ok(Vec::new());
                resp.status = 204;
                resp.headers.insert(
                    "access-control-allow-origin".to_string(),
                    origin,
                );
                resp.headers.insert(
                    "access-control-allow-methods".to_string(),
                    self.allowed_methods.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", "),
                );
                resp.headers.insert(
                    "access-control-allow-headers".to_string(),
                    self.allowed_headers.join(", "),
                );
                resp.headers.insert(
                    "access-control-max-age".to_string(),
                    self.max_age.to_string(),
                );
                return Ok(Some(resp));
            }
        }

        Ok(None)
    }

    async fn on_response(&self, req: &Request, resp: &mut Response) -> Result<(), NexusError> {
        let origin = req.headers.get("origin").cloned().unwrap_or_default();

        if !origin.is_empty() && self.is_origin_allowed(&origin) {
            resp.headers.insert(
                "access-control-allow-origin".to_string(),
                origin,
            );
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "cors"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cors_preflight() {
        let cors = CorsMiddleware::permissive();
        let mut req = Request::new(HttpMethod::Options, "/api/data")
            .with_header("origin", "http://localhost:3000");

        let result = cors.on_request(&mut req).await.unwrap();
        assert!(result.is_some());
        let resp = result.unwrap();
        assert_eq!(resp.status, 204);
        assert!(resp.headers.contains_key("access-control-allow-origin"));
    }
}
