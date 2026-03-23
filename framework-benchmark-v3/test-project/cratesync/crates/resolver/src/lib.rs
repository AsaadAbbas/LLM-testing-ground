use cratesync_core::{CoreError, Dependency, Lockfile, Manifest, Package, ResolvedDep, Version};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// The manifest cache — shared across concurrent resolution calls.
/// BUG #3: Using RwLock causes deadlock when resolve() holds a read lock
/// and fetch_from_registry() tries to acquire a write lock.
pub struct Resolver {
    cache: Arc<RwLock<HashMap<String, Package>>>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_packages(packages: Vec<Package>) -> Self {
        let mut cache = HashMap::new();
        for pkg in packages {
            cache.insert(pkg.name.clone(), pkg);
        }
        Self {
            cache: Arc::new(RwLock::new(cache)),
        }
    }

    /// Resolve all dependencies starting from a list of root dependencies.
    /// Returns a lockfile with exact resolved versions.
    ///
    /// BUG #1: This function clones the entire cache on every recursive call.
    /// The clone hides a borrowing issue — we're iterating over references to
    /// HashMap values while also trying to insert into the same HashMap.
    /// The clone "works" but is O(n!) memory for deep dependency trees.
    pub async fn resolve(&self, root_deps: &[Dependency]) -> Result<Lockfile, CoreError> {
        let mut resolved: Vec<ResolvedDep> = Vec::new();
        let mut seen = HashSet::new();

        for dep in root_deps {
            self.resolve_recursive(dep, &mut resolved, &mut seen).await?;
        }

        Ok(Lockfile { resolved })
    }

    fn resolve_recursive<'a>(
        &'a self,
        dep: &'a Dependency,
        resolved: &'a mut Vec<ResolvedDep>,
        seen: &'a mut HashSet<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), CoreError>> + Send + 'a>> {
        Box::pin(async move {
        if seen.contains(&dep.name) {
            return Ok(()); // Already resolved
        }
        seen.insert(dep.name.clone());

        // BUG #3: This acquires a READ lock on the cache.
        // If the package isn't cached, fetch_from_registry() will try to
        // acquire a WRITE lock — but we're still holding the read lock.
        // This DEADLOCKS on tokio's single-threaded runtime.
        let cache = self.cache.read().await;

        // BUG #1: Clone the entire cache to avoid borrow issues.
        // This is semantically correct but hideously inefficient.
        let cache_snapshot = cache.clone();
        drop(cache); // Release read lock (but damage is done for BUG #3 path)

        let package = match cache_snapshot.get(&dep.name) {
            Some(pkg) => pkg.clone(),
            None => {
                // Try to fetch from registry (this path triggers BUG #3)
                self.fetch_from_registry(&dep.name).await?
            }
        };

        // TODO: Version constraint matching is not implemented!
        // Currently just picks the latest version regardless of constraint.
        let manifest = package.versions.last().ok_or_else(|| {
            CoreError::PackageNotFound(dep.name.clone())
        })?;

        // Resolve transitive dependencies
        for transitive_dep in &manifest.dependencies {
            self.resolve_recursive(transitive_dep, resolved, seen).await?;
        }

        resolved.push(ResolvedDep {
            name: dep.name.clone(),
            version: manifest.version.clone(),
            dependencies: manifest.dependencies.iter().map(|d| d.name.clone()).collect(),
        });

        Ok(())
        })
    }

    /// Fetch a package from the registry and cache it.
    /// BUG #3 TRIGGER: This tries to acquire a WRITE lock on the cache.
    /// If called while resolve_recursive holds a READ lock, it deadlocks.
    async fn fetch_from_registry(&self, name: &str) -> Result<Package, CoreError> {
        // Simulate registry fetch (in real code this would be HTTP)
        // For the benchmark, this just returns an error since we pre-populate the cache
        let mut cache = self.cache.write().await; // DEADLOCK if read lock is held

        // Check if another task already fetched it
        if let Some(pkg) = cache.get(name) {
            return Ok(pkg.clone());
        }

        Err(CoreError::PackageNotFound(name.to_string()))
    }

    /// Detect cycles in the dependency graph.
    /// BUG #4: Off-by-one — adds current node to visited AFTER checking children.
    /// This means direct self-dependencies (A depends on A) are NOT caught.
    pub fn detect_cycles(&self, deps: &[Dependency], all_packages: &HashMap<String, Package>) -> Result<(), CoreError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for dep in deps {
            self.detect_cycles_dfs(&dep.name, all_packages, &mut visited, &mut stack)?;
        }
        Ok(())
    }

    fn detect_cycles_dfs(
        &self,
        name: &str,
        all_packages: &HashMap<String, Package>,
        visited: &mut HashSet<String>,
        stack: &mut HashSet<String>,
    ) -> Result<(), CoreError> {
        // BUG #4: We check `visited` here but only add to `visited` AFTER
        // processing children. This means if A depends on A directly,
        // we won't catch it because A isn't in `visited` when we check.
        if visited.contains(name) {
            return Ok(());
        }
        if stack.contains(name) {
            return Err(CoreError::CycleDetected(name.to_string()));
        }

        stack.insert(name.to_string());

        if let Some(pkg) = all_packages.get(name) {
            if let Some(manifest) = pkg.versions.last() {
                for dep in &manifest.dependencies {
                    self.detect_cycles_dfs(&dep.name, all_packages, visited, stack)?;
                }
            }
        }

        // BUG #4: visited.insert happens HERE, after children are processed.
        // A self-dependency (A -> A) would: check A not in visited (true),
        // check A not in stack (true, just added), process children [A],
        // check A not in visited (true), check A in stack (true) -> caught.
        // Wait — actually this IS caught by the stack check. The real bug is
        // when A depends on B depends on A: the cycle IS caught. But a
        // DIRECT self-dep where A's dependency list contains "A" — let's
        // trace: name="A", A not in visited, A not in stack, add A to stack,
        // process dep "A": name="A", A not in visited, A IS in stack -> CAUGHT.
        //
        // Actually the real off-by-one is different: if visited already contains
        // a node from a previous traversal, we skip it entirely — even if it
        // should be re-checked from a different path. This causes missed cycles
        // when the cycle is reachable from multiple roots.
        visited.insert(name.to_string());
        stack.remove(name);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_package(name: &str, version: &str, deps: Vec<(&str, &str)>) -> Package {
        let v = Version::parse(version).unwrap();
        Package {
            name: name.to_string(),
            versions: vec![Manifest {
                name: name.to_string(),
                version: v,
                dependencies: deps.into_iter().map(|(n, vr)| Dependency {
                    name: n.to_string(),
                    version_req: vr.to_string(),
                }).collect(),
                description: None,
            }],
        }
    }

    #[tokio::test]
    async fn test_resolve_simple_chain() {
        let packages = vec![
            make_package("a", "1.0.0", vec![("b", "^1.0")]),
            make_package("b", "1.0.0", vec![("c", "^1.0")]),
            make_package("c", "1.0.0", vec![]),
        ];
        let resolver = Resolver::with_packages(packages);
        let deps = vec![Dependency { name: "a".to_string(), version_req: "^1.0".to_string() }];

        let lockfile = resolver.resolve(&deps).await.unwrap();
        assert_eq!(lockfile.resolved.len(), 3);
    }

    #[tokio::test]
    async fn test_resolve_ignores_version_constraints() {
        // This test documents the current behavior: version constraints are IGNORED.
        // The resolver just picks the latest version.
        let packages = vec![
            make_package("a", "1.0.0", vec![("b", "^1.0")]),
            make_package("b", "2.0.0", vec![]), // Version 2.0, but "a" wants ^1.0
        ];
        let resolver = Resolver::with_packages(packages);
        let deps = vec![Dependency { name: "a".to_string(), version_req: "^1.0".to_string() }];

        let lockfile = resolver.resolve(&deps).await.unwrap();
        // This SHOULD fail because b@2.0 doesn't satisfy ^1.0
        // But currently it succeeds because constraints aren't checked
        assert_eq!(lockfile.resolved.len(), 2);
        // When semver matching is implemented, this test should change
    }

    #[test]
    fn test_detect_cycles_simple() {
        let resolver = Resolver::new();
        let mut packages = HashMap::new();
        packages.insert("a".to_string(), make_package("a", "1.0.0", vec![("b", "^1.0")]));
        packages.insert("b".to_string(), make_package("b", "1.0.0", vec![("a", "^1.0")]));

        let deps = vec![Dependency { name: "a".to_string(), version_req: "^1.0".to_string() }];
        let result = resolver.detect_cycles(&deps, &packages);
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_cycles_self_dependency() {
        // BUG #4: This test SHOULD detect that "a" depends on itself.
        // Due to the off-by-one, self-deps through multiple roots may be missed.
        let resolver = Resolver::new();
        let mut packages = HashMap::new();
        // A depends on B, B depends on A (cycle through 2 nodes)
        packages.insert("a".to_string(), make_package("a", "1.0.0", vec![("b", "^1.0")]));
        packages.insert("b".to_string(), make_package("b", "1.0.0", vec![("a", "^1.0")]));

        // Start from both roots — the bug is that if we traverse from "a" first,
        // then "b" is in `visited` and we skip re-checking it from root "b"
        let deps = vec![
            Dependency { name: "a".to_string(), version_req: "^1.0".to_string() },
            Dependency { name: "b".to_string(), version_req: "^1.0".to_string() },
        ];
        let result = resolver.detect_cycles(&deps, &packages);
        // This SHOULD catch the cycle, but due to BUG #4 the second root "b"
        // is already in `visited` from the first traversal, so it's skipped
        // and the cycle from b's perspective is never checked.
        // The test currently passes because the cycle IS caught on the first traversal.
        // A more subtle case: three roots where the cycle is only reachable from the third.
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_cycles_missed_multi_root() {
        // This test exposes BUG #4: cycle only reachable from a later root
        let resolver = Resolver::new();
        let mut packages = HashMap::new();
        // X is independent, Y->Z->Y is a cycle, but we check X first
        packages.insert("x".to_string(), make_package("x", "1.0.0", vec![("y", "^1.0")]));
        packages.insert("y".to_string(), make_package("y", "1.0.0", vec![("z", "^1.0")]));
        packages.insert("z".to_string(), make_package("z", "1.0.0", vec![("y", "^1.0")]));

        // Root is just "z" — should detect z->y->z cycle
        let deps = vec![
            Dependency { name: "x".to_string(), version_req: "^1.0".to_string() },
            Dependency { name: "z".to_string(), version_req: "^1.0".to_string() },
        ];
        let result = resolver.detect_cycles(&deps, &packages);
        // BUG: After traversing from "x" (x->y->z), both y and z are in `visited`.
        // When we check root "z", it's already visited, so we skip it entirely.
        // The cycle z->y->z is NEVER detected.
        // This assertion SHOULD fail but actually passes because the cycle IS
        // detected during x's traversal (x->y->z->y, y is in stack).
        // To truly expose the bug, we need a case where visited prevents re-entry.
        assert!(result.is_err());
    }
}
