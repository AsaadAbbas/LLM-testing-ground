use nexus_core::{Middleware, NexusError, Request, Response};

/// Response body transformation middleware.
///
/// Can wrap responses in a standard envelope format:
/// { "success": true, "data": <original_body>, "metadata": { ... } }
pub struct TransformMiddleware {
    /// Whether to wrap responses in the envelope.
    envelope_enabled: bool,
    /// Fields to strip from response bodies.
    strip_fields: Vec<String>,
}

impl TransformMiddleware {
    pub fn new(envelope_enabled: bool) -> Self {
        Self {
            envelope_enabled,
            strip_fields: Vec::new(),
        }
    }

    pub fn with_strip_fields(mut self, fields: Vec<String>) -> Self {
        self.strip_fields = fields;
        self
    }
}

#[async_trait::async_trait]
impl Middleware for TransformMiddleware {
    async fn on_request(&self, _req: &mut Request) -> Result<Option<Response>, NexusError> {
        Ok(None)
    }

    async fn on_response(&self, _req: &Request, resp: &mut Response) -> Result<(), NexusError> {
        if !self.envelope_enabled {
            return Ok(());
        }

        // Try to parse body as JSON and wrap in envelope
        if let Ok(body_str) = std::str::from_utf8(&resp.body) {
            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body_str) {
                // Strip fields
                if let Some(obj) = value.as_object_mut() {
                    for field in &self.strip_fields {
                        obj.remove(field);
                    }
                }

                let envelope = serde_json::json!({
                    "success": resp.status >= 200 && resp.status < 300,
                    "data": value,
                    "metadata": {
                        "status": resp.status,
                    }
                });

                resp.body = serde_json::to_vec(&envelope)
                    .map_err(|e| NexusError::Serialization(e.to_string()))?;
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "transform"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core::HttpMethod;

    #[tokio::test]
    async fn test_transform_envelope() {
        let transform = TransformMiddleware::new(true);
        let req = Request::new(HttpMethod::Get, "/test");
        let mut resp = Response::ok(b"{\"key\": \"value\"}".to_vec());

        transform.on_response(&req, &mut resp).await.unwrap();

        let body: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert_eq!(body["success"], true);
        assert_eq!(body["data"]["key"], "value");
    }
}
