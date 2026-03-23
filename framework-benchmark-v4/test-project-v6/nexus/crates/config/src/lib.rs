use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level Nexus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexusConfig {
    pub server: ServerConfig,
    pub logging: PluginToggle,
    pub cors: CorsConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub transform: TransformConfig,
    pub routes: Vec<RouteConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginToggle {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub enabled: bool,
    pub permissive: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub api_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub max_requests: u32,
    pub refill_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    pub enabled: bool,
    pub envelope: bool,
    pub strip_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub method: String,
    pub path: String,
    pub backend: String,
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
}

impl NexusConfig {
    /// Load config from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Load config from a file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_json(&content)?)
    }

    /// Default development configuration.
    pub fn default_dev() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
            },
            logging: PluginToggle { enabled: true },
            cors: CorsConfig {
                enabled: true,
                permissive: true,
                allowed_origins: vec![],
            },
            auth: AuthConfig {
                enabled: false,
                api_keys: vec![],
            },
            rate_limit: RateLimitConfig {
                enabled: false,
                max_requests: 100,
                refill_rate: 10,
            },
            transform: TransformConfig {
                enabled: false,
                envelope: false,
                strip_fields: vec![],
            },
            routes: vec![],
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("server.port cannot be 0".to_string());
        }

        if self.auth.enabled && self.auth.api_keys.is_empty() {
            errors.push("auth is enabled but no API keys configured".to_string());
        }

        for (i, route) in self.routes.iter().enumerate() {
            if route.path.is_empty() {
                errors.push(format!("route[{}] has empty path", i));
            }
            if !route.path.starts_with('/') {
                errors.push(format!("route[{}] path must start with /", i));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dev_config() {
        let config = NexusConfig::default_dev();
        assert_eq!(config.server.port, 3000);
        assert!(!config.auth.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NexusConfig::default_dev();
        config.auth.enabled = true;
        config.auth.api_keys = vec![]; // enabled but no keys

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_json_roundtrip() {
        let config = NexusConfig::default_dev();
        let json = serde_json::to_string(&config).unwrap();
        let parsed = NexusConfig::from_json(&json).unwrap();
        assert_eq!(parsed.server.port, config.server.port);
    }
}
