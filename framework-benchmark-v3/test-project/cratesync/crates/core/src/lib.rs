use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn parse(s: &str) -> Result<Self, CoreError> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(CoreError::InvalidVersion(s.to_string()));
        }
        Ok(Self {
            major: parts[0].parse().map_err(|_| CoreError::InvalidVersion(s.to_string()))?,
            minor: parts[1].parse().map_err(|_| CoreError::InvalidVersion(s.to_string()))?,
            patch: parts[2].parse().map_err(|_| CoreError::InvalidVersion(s.to_string()))?,
        })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: String, // e.g., "^1.2.0", "~1.0", ">=1.0 <2.0"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<Dependency>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub versions: Vec<Manifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDep {
    pub name: String,
    pub version: Version,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    pub resolved: Vec<ResolvedDep>,
}

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid version: {0}")]
    InvalidVersion(String),
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    #[error("Dependency cycle detected: {0}")]
    CycleDetected(String),
    #[error("No compatible version for {package} matching {constraint}")]
    NoCompatibleVersion { package: String, constraint: String },
    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),
    #[error("Registry error: {0}")]
    RegistryError(String),
}

// API request/response types for the server
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub packages: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub synced: Vec<String>,
    pub failed: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveRequest {
    pub root_dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResolveResponse {
    pub lockfile: Lockfile,
}
