// Types matching the CrateSync API protocol
// BUG #2: Version type doesn't match Rust's serialization.
// Rust serializes: { major: u32, minor: u32, patch: u32 }
// This defines: version as a single string "1.2.3"
// Result: When deserializing server responses, version fields are undefined

export interface Version {
  // BUG: Should be { major: number, minor: number, patch: number }
  // to match Rust's serde derive output. Instead we use a flat string.
  version: string; // e.g., "1.2.3"
}

export interface Dependency {
  name: string;
  version_req: string;
}

export interface Manifest {
  name: string;
  version: Version; // BUG: This doesn't match Rust's { major, minor, patch }
  dependencies: Dependency[];
  description?: string;
}

export interface Package {
  name: string;
  versions: Manifest[];
}

export interface ResolvedDep {
  name: string;
  version: Version; // BUG: Same mismatch
  dependencies: string[];
}

export interface Lockfile {
  resolved: ResolvedDep[];
}

export interface SyncRequest {
  packages: string[];
}

export interface SyncResponse {
  synced: string[];
  failed: string[];
}

export interface ResolveRequest {
  root_dependencies: Dependency[];
}

export interface ResolveResponse {
  lockfile: Lockfile;
}

// Helper to format a version — silently returns "undefined.undefined.undefined"
// when the Rust server sends { major, minor, patch } instead of { version: "..." }
export function formatVersion(v: Version): string {
  // BUG: v.version is undefined when data comes from Rust server
  // because Rust sends { major: 1, minor: 0, patch: 0 }
  // not { version: "1.0.0" }
  return v.version;
}
