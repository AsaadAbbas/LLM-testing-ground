use nexus_core::{Handler, HttpMethod, Middleware, NexusError, PipelineMetrics, Request, Response, Route};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// The middleware pipeline. Processes requests through a chain of middleware,
/// then dispatches to the matched route handler.
pub struct Pipeline {
    /// Middleware applied to ALL requests, in order.
    global_middleware: Vec<Arc<dyn Middleware>>,
    /// Per-route middleware (path pattern → middleware list).
    route_middleware: HashMap<String, Vec<Arc<dyn Middleware>>>,
    /// Route table.
    routes: Vec<Route>,
    /// Collected metrics.
    metrics: Arc<RwLock<PipelineMetrics>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            global_middleware: Vec::new(),
            route_middleware: HashMap::new(),
            routes: Vec::new(),
            metrics: Arc::new(RwLock::new(PipelineMetrics::default())),
        }
    }

    /// Add a global middleware that applies to all requests.
    /// Middleware executes in the order added.
    pub fn add_middleware(&mut self, mw: Arc<dyn Middleware>) {
        self.global_middleware.push(mw);
    }

    /// Add middleware for a specific route pattern only.
    pub fn add_route_middleware(&mut self, pattern: &str, mw: Arc<dyn Middleware>) {
        self.route_middleware
            .entry(pattern.to_string())
            .or_default()
            .push(mw);
    }

    /// Register a route.
    pub fn add_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Process a request through the full pipeline.
    ///
    /// 1. Run global middleware on_request (in order)
    /// 2. Match route
    /// 3. Run route-specific middleware on_request (in order)
    /// 4. Execute handler
    /// 5. Run route-specific middleware on_response (reverse order)
    /// 6. Run global middleware on_response (reverse order)
    pub async fn process(&self, mut req: Request) -> Result<Response, NexusError> {
        let start = Instant::now();

        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
            metrics.active_connections += 1;
        }

        // Phase 1: Global middleware on_request
        for mw in &self.global_middleware {
            let mw_start = Instant::now();
            match mw.on_request(&mut req).await {
                Ok(Some(response)) => {
                    // Short-circuit: middleware returned a response
                    self.record_metrics(start, Some(mw.name())).await;
                    return Ok(response);
                }
                Ok(None) => {
                    self.record_middleware_timing(mw.name(), mw_start.elapsed().as_secs_f64()).await;
                }
                Err(e) => {
                    self.record_error().await;
                    return Err(e);
                }
            }
        }

        // Phase 2: Route matching
        let matched_route = self.match_route(&req);
        let (handler, pattern) = match matched_route {
            Some((handler, pattern)) => (handler, pattern),
            None => {
                self.record_error().await;
                return Err(NexusError::RouteNotFound {
                    method: req.method.to_string(),
                    path: req.path.clone(),
                });
            }
        };

        // Phase 3: Route-specific middleware on_request
        let route_mws = self.route_middleware.get(&pattern).cloned().unwrap_or_default();
        for mw in &route_mws {
            match mw.on_request(&mut req).await {
                Ok(Some(response)) => {
                    self.record_metrics(start, None).await;
                    return Ok(response);
                }
                Ok(None) => {}
                Err(e) => {
                    self.record_error().await;
                    return Err(e);
                }
            }
        }

        // Phase 4: Execute handler
        let mut response = handler.handle(&req).await?;

        // Phase 5: Route-specific middleware on_response (should be reverse order)
        for mw in &route_mws {
            mw.on_response(&req, &mut response).await?;
        }

        // Phase 6: Global middleware on_response (should be reverse order)
        for mw in &self.global_middleware {
            mw.on_response(&req, &mut response).await?;
        }

        self.record_metrics(start, None).await;
        Ok(response)
    }

    /// Match a request to a route. Returns the handler and the pattern.
    fn match_route(&self, req: &Request) -> Option<(&dyn Handler, String)> {
        for route in &self.routes {
            if route.method != req.method {
                continue;
            }

            if self.path_matches(&route.path_pattern, &req.path) {
                return Some((route.handler.as_ref(), route.path_pattern.clone()));
            }
        }
        None
    }

    /// Simple path matching with :param support.
    /// "/api/:id/data" matches "/api/123/data"
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (p, a) in pattern_parts.iter().zip(path_parts.iter()) {
            if p.starts_with(':') {
                continue; // Wildcard match
            }
            if p != a {
                return false;
            }
        }

        true
    }

    /// Get current metrics.
    pub async fn metrics(&self) -> PipelineMetrics {
        self.metrics.read().await.clone()
    }

    async fn record_metrics(&self, start: Instant, _short_circuit_by: Option<&str>) {
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        let mut metrics = self.metrics.write().await;
        metrics.active_connections = metrics.active_connections.saturating_sub(1);
        // Update rolling average
        let n = metrics.total_requests as f64;
        metrics.avg_latency_ms = metrics.avg_latency_ms * ((n - 1.0) / n) + elapsed / n;
    }

    async fn record_error(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_errors += 1;
        metrics.active_connections = metrics.active_connections.saturating_sub(1);
    }

    async fn record_middleware_timing(&self, name: &str, duration: f64) {
        let mut metrics = self.metrics.write().await;
        let entry = metrics.middleware_timings.entry(name.to_string()).or_insert(0.0);
        *entry += duration;
    }

    /// Get the number of registered routes.
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Get the names of all registered middleware.
    pub fn middleware_names(&self) -> Vec<String> {
        self.global_middleware.iter().map(|m| m.name().to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoHandler;

    #[async_trait::async_trait]
    impl Handler for EchoHandler {
        async fn handle(&self, req: &Request) -> Result<Response, NexusError> {
            Ok(Response::ok(req.body.clone()))
        }
    }

    struct AddHeaderMiddleware {
        key: String,
        value: String,
    }

    #[async_trait::async_trait]
    impl Middleware for AddHeaderMiddleware {
        async fn on_request(&self, req: &mut Request) -> Result<Option<Response>, NexusError> {
            req.headers.insert(self.key.clone(), self.value.clone());
            Ok(None)
        }

        async fn on_response(&self, _req: &Request, resp: &mut Response) -> Result<(), NexusError> {
            resp.headers.insert(self.key.clone(), self.value.clone());
            Ok(())
        }

        fn name(&self) -> &str {
            "add-header"
        }
    }

    #[tokio::test]
    async fn test_basic_pipeline() {
        let mut pipeline = Pipeline::new();
        pipeline.add_route(Route {
            method: HttpMethod::Get,
            path_pattern: "/api/echo".to_string(),
            handler: Box::new(EchoHandler),
        });

        let req = Request::new(HttpMethod::Get, "/api/echo")
            .with_body(b"test".to_vec());

        let resp = pipeline.process(req).await.unwrap();
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"test");
    }

    #[tokio::test]
    async fn test_middleware_adds_header() {
        let mut pipeline = Pipeline::new();
        pipeline.add_middleware(Arc::new(AddHeaderMiddleware {
            key: "x-request-id".to_string(),
            value: "test-123".to_string(),
        }));
        pipeline.add_route(Route {
            method: HttpMethod::Get,
            path_pattern: "/api/echo".to_string(),
            handler: Box::new(EchoHandler),
        });

        let req = Request::new(HttpMethod::Get, "/api/echo");
        let resp = pipeline.process(req).await.unwrap();
        assert_eq!(resp.headers.get("x-request-id").unwrap(), "test-123");
    }

    #[tokio::test]
    async fn test_route_not_found() {
        let pipeline = Pipeline::new();
        let req = Request::new(HttpMethod::Get, "/nonexistent");
        let result = pipeline.process(req).await;
        assert!(matches!(result, Err(NexusError::RouteNotFound { .. })));
    }

    #[tokio::test]
    async fn test_path_matching() {
        let mut pipeline = Pipeline::new();
        pipeline.add_route(Route {
            method: HttpMethod::Get,
            path_pattern: "/api/:id/info".to_string(),
            handler: Box::new(EchoHandler),
        });

        let req = Request::new(HttpMethod::Get, "/api/123/info");
        let resp = pipeline.process(req).await.unwrap();
        assert_eq!(resp.status, 200);
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let mut pipeline = Pipeline::new();
        pipeline.add_route(Route {
            method: HttpMethod::Get,
            path_pattern: "/api/test".to_string(),
            handler: Box::new(EchoHandler),
        });

        let req = Request::new(HttpMethod::Get, "/api/test");
        pipeline.process(req).await.unwrap();

        let metrics = pipeline.metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.total_errors, 0);
    }
}
