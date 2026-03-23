use cratesync_core::{CoreError, Package};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Registry {
    base_url: String,
    cache: Arc<RwLock<HashMap<String, Package>>>,
}

impl Registry {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn fetch_package(&self, name: &str) -> Result<Package, CoreError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(pkg) = cache.get(name) {
                return Ok(pkg.clone());
            }
        }

        // Fetch from remote
        let url = format!("{}/packages/{}", self.base_url, name);
        let response = reqwest::get(&url).await
            .map_err(|e| CoreError::RegistryError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CoreError::PackageNotFound(name.to_string()));
        }

        let package: Package = response.json().await
            .map_err(|e| CoreError::RegistryError(e.to_string()))?;

        // Cache it
        let mut cache = self.cache.write().await;
        cache.insert(name.to_string(), package.clone());

        Ok(package)
    }

    pub async fn list_packages(&self) -> Result<Vec<String>, CoreError> {
        let url = format!("{}/packages", self.base_url);
        let response = reqwest::get(&url).await
            .map_err(|e| CoreError::RegistryError(e.to_string()))?;

        let names: Vec<String> = response.json().await
            .map_err(|e| CoreError::RegistryError(e.to_string()))?;

        Ok(names)
    }
}
