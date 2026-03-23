use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Core error type for Nexus operations.
#[derive(Debug, thiserror::Error)]
pub enum NexusError {
    #[error("pipeline error: {0}")]
    Pipeline(String),
    #[error("plugin error in {plugin}: {message}")]
    Plugin { plugin: String, message: String },
    #[error("config error: {0}")]
    Config(String),
    #[error("route not found: {method} {path}")]
    RouteNotFound { method: String, path: String },
    #[error("middleware rejected request: {0}")]
    Rejected(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("timeout after {0}ms")]
    Timeout(u64),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// HTTP method enum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Options => write!(f, "OPTIONS"),
            HttpMethod::Head => write!(f, "HEAD"),
        }
    }
}

/// Represents an HTTP request flowing through the middleware pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub query_params: HashMap<String, String>,
    /// Metadata attached by middleware (e.g., auth info, request ID).
    pub extensions: HashMap<String, serde_json::Value>,
}

impl Request {
    pub fn new(method: HttpMethod, path: &str) -> Self {
        Self {
            method,
            path: path.to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            query_params: HashMap::new(),
            extensions: HashMap::new(),
        }
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn body_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}

/// Represents an HTTP response from the pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body,
        }
    }

    pub fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: message.as_bytes().to_vec(),
        }
    }

    pub fn json<T: Serialize>(data: &T) -> Result<Self, NexusError> {
        let body = serde_json::to_vec(data)
            .map_err(|e| NexusError::Serialization(e.to_string()))?;
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(Self {
            status: 200,
            headers,
            body,
        })
    }

    pub fn body_as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}

/// The core middleware trait. Middleware can inspect and modify requests,
/// and optionally short-circuit the pipeline by returning a response.
///
/// Middleware is applied in order. Each middleware receives the request,
/// can modify it, then calls `next` to continue the chain. After the
/// downstream chain completes, the middleware can also modify the response.
#[async_trait::async_trait]
pub trait Middleware: Send + Sync {
    /// Process the request. Return Ok(None) to continue the chain,
    /// or Ok(Some(response)) to short-circuit.
    async fn on_request(&self, req: &mut Request) -> Result<Option<Response>, NexusError>;

    /// Process the response after the downstream chain completes.
    /// Default implementation passes through unchanged.
    async fn on_response(&self, _req: &Request, resp: &mut Response) -> Result<(), NexusError> {
        Ok(())
    }

    /// The name of this middleware (for logging and error reporting).
    fn name(&self) -> &str;
}

/// A route handler that produces a response for a matched request.
#[async_trait::async_trait]
pub trait Handler: Send + Sync {
    async fn handle(&self, req: &Request) -> Result<Response, NexusError>;
}

/// Route definition: method + path pattern + handler.
pub struct Route {
    pub method: HttpMethod,
    pub path_pattern: String,
    pub handler: Box<dyn Handler>,
}

/// Plugin configuration passed from the config system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Metrics collected by the pipeline.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineMetrics {
    pub total_requests: u64,
    pub total_errors: u64,
    pub total_rejections: u64,
    pub avg_latency_ms: f64,
    pub active_connections: u64,
    pub middleware_timings: HashMap<String, f64>,
}

/// Admin API event types for real-time updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum AdminEvent {
    RouteAdded { method: String, path: String },
    RouteRemoved { method: String, path: String },
    PluginEnabled { name: String },
    PluginDisabled { name: String },
    ConfigReloaded,
    MetricsSnapshot(PipelineMetrics),
}

/// Rate limit entry for tracking request counts.
#[derive(Debug, Clone)]
pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: std::time::Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_builder() {
        let req = Request::new(HttpMethod::Post, "/api/data")
            .with_header("content-type", "application/json")
            .with_body(b"hello".to_vec());

        assert_eq!(req.method, HttpMethod::Post);
        assert_eq!(req.path, "/api/data");
        assert_eq!(req.body_as_str(), Some("hello"));
    }

    #[test]
    fn test_response_json() {
        let data = serde_json::json!({"status": "ok"});
        let resp = Response::json(&data).unwrap();
        assert_eq!(resp.status, 200);
        assert!(resp.headers.get("content-type").unwrap().contains("json"));
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
    }

    #[test]
    fn test_admin_event_serialization() {
        let event = AdminEvent::RouteAdded {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("RouteAdded"));
    }
}
