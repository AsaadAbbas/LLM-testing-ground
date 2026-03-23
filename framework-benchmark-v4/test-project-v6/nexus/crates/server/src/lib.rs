use nexus_core::{
    AdminEvent, Handler, HttpMethod, Middleware, NexusError, PipelineMetrics, Request, Response, Route,
};
use nexus_pipeline::Pipeline;
use nexus_plugins::{
    auth::AuthMiddleware, cors::CorsMiddleware, logging::LoggingMiddleware,
    rate_limit::RateLimitMiddleware, transform::TransformMiddleware,
};
use nexus_config::NexusConfig;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// The Nexus API gateway server.
pub struct NexusServer {
    pipeline: Arc<RwLock<Pipeline>>,
    config: Arc<RwLock<NexusConfig>>,
    event_tx: broadcast::Sender<AdminEvent>,
}

impl NexusServer {
    /// Create a new server from configuration.
    pub fn from_config(config: NexusConfig) -> Result<Self, NexusError> {
        let (event_tx, _) = broadcast::channel(100);
        let mut pipeline = Pipeline::new();

        // Add middleware in the configured order
        if config.logging.enabled {
            pipeline.add_middleware(Arc::new(LoggingMiddleware::new()));
        }

        if config.cors.enabled {
            let cors = if config.cors.permissive {
                CorsMiddleware::permissive()
            } else {
                CorsMiddleware::permissive()
                    .with_origins(config.cors.allowed_origins.clone())
            };
            pipeline.add_middleware(Arc::new(cors));
        }

        if config.auth.enabled {
            pipeline.add_middleware(Arc::new(
                AuthMiddleware::new(config.auth.api_keys.clone()),
            ));
        }

        if config.rate_limit.enabled {
            pipeline.add_middleware(Arc::new(
                RateLimitMiddleware::new(
                    config.rate_limit.max_requests,
                    config.rate_limit.refill_rate,
                ),
            ));
        }

        if config.transform.enabled {
            pipeline.add_middleware(Arc::new(
                TransformMiddleware::new(config.transform.envelope)
                    .with_strip_fields(config.transform.strip_fields.clone()),
            ));
        }

        // Register configured routes
        for route_config in &config.routes {
            let handler: Box<dyn Handler> = match route_config.backend.as_str() {
                "echo" => Box::new(EchoHandler),
                "static" => Box::new(StaticHandler {
                    body: route_config
                        .settings
                        .get("body")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                }),
                "proxy" => Box::new(ProxyHandler {
                    target: route_config
                        .settings
                        .get("target")
                        .and_then(|v| v.as_str())
                        .unwrap_or("http://localhost:8080")
                        .to_string(),
                }),
                _ => {
                    return Err(NexusError::Config(format!(
                        "unknown backend: {}",
                        route_config.backend
                    )));
                }
            };

            let method = match route_config.method.to_uppercase().as_str() {
                "GET" => HttpMethod::Get,
                "POST" => HttpMethod::Post,
                "PUT" => HttpMethod::Put,
                "DELETE" => HttpMethod::Delete,
                _ => HttpMethod::Get,
            };

            pipeline.add_route(Route {
                method,
                path_pattern: route_config.path.clone(),
                handler,
            });
        }

        Ok(Self {
            pipeline: Arc::new(RwLock::new(pipeline)),
            config: Arc::new(RwLock::new(config)),
            event_tx,
        })
    }

    /// Process a request through the pipeline.
    pub async fn handle_request(&self, req: Request) -> Result<Response, NexusError> {
        let pipeline = self.pipeline.read().await;
        pipeline.process(req).await
    }

    /// Get current metrics.
    pub async fn metrics(&self) -> PipelineMetrics {
        let pipeline = self.pipeline.read().await;
        pipeline.metrics().await
    }

    /// Subscribe to admin events.
    pub fn subscribe(&self) -> broadcast::Receiver<AdminEvent> {
        self.event_tx.subscribe()
    }

    /// Get the current config.
    pub async fn config(&self) -> NexusConfig {
        self.config.read().await.clone()
    }

    /// Reload configuration. Rebuilds the pipeline.
    pub async fn reload_config(&self, new_config: NexusConfig) -> Result<(), NexusError> {
        let new_server = Self::from_config(new_config.clone())?;
        *self.pipeline.write().await = Arc::try_unwrap(new_server.pipeline)
            .map_err(|_| NexusError::Pipeline("failed to unwrap pipeline".to_string()))?
            .into_inner();
        *self.config.write().await = new_config;

        let _ = self.event_tx.send(AdminEvent::ConfigReloaded);
        Ok(())
    }
}

/// Echo handler — returns the request body as-is.
struct EchoHandler;

#[async_trait::async_trait]
impl Handler for EchoHandler {
    async fn handle(&self, req: &Request) -> Result<Response, NexusError> {
        let mut resp = Response::ok(req.body.clone());
        resp.headers.insert(
            "content-type".to_string(),
            req.headers
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "text/plain".to_string()),
        );
        Ok(resp)
    }
}

/// Static handler — returns a fixed response body.
struct StaticHandler {
    body: String,
}

#[async_trait::async_trait]
impl Handler for StaticHandler {
    async fn handle(&self, _req: &Request) -> Result<Response, NexusError> {
        Response::json(&serde_json::json!({ "message": self.body }))
    }
}

/// Proxy handler — forwards requests to a backend.
/// Currently returns a placeholder.
struct ProxyHandler {
    target: String,
}

#[async_trait::async_trait]
impl Handler for ProxyHandler {
    async fn handle(&self, req: &Request) -> Result<Response, NexusError> {
        // TODO: Implement actual HTTP proxying
        Response::json(&serde_json::json!({
            "proxied_to": self.target,
            "method": req.method.to_string(),
            "path": req.path,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> NexusConfig {
        NexusConfig {
            server: nexus_config::ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            logging: nexus_config::PluginToggle { enabled: true },
            cors: nexus_config::CorsConfig {
                enabled: true,
                permissive: true,
                allowed_origins: vec![],
            },
            auth: nexus_config::AuthConfig {
                enabled: true,
                api_keys: vec!["test-key".to_string()],
            },
            rate_limit: nexus_config::RateLimitConfig {
                enabled: true,
                max_requests: 100,
                refill_rate: 10,
            },
            transform: nexus_config::TransformConfig {
                enabled: false,
                envelope: false,
                strip_fields: vec![],
            },
            routes: vec![
                nexus_config::RouteConfig {
                    method: "GET".to_string(),
                    path: "/api/echo".to_string(),
                    backend: "echo".to_string(),
                    settings: Default::default(),
                },
            ],
        }
    }

    #[tokio::test]
    async fn test_server_from_config() {
        let config = test_config();
        let server = NexusServer::from_config(config).unwrap();

        let req = Request::new(HttpMethod::Get, "/api/echo")
            .with_header("authorization", "Bearer test-key")
            .with_body(b"hello".to_vec());

        let resp = server.handle_request(req).await.unwrap();
        assert_eq!(resp.status, 200);
    }

    #[tokio::test]
    async fn test_server_auth_required() {
        let config = test_config();
        let server = NexusServer::from_config(config).unwrap();

        // No auth header
        let req = Request::new(HttpMethod::Get, "/api/echo");
        let resp = server.handle_request(req).await.unwrap();
        assert_eq!(resp.status, 401);
    }
}
