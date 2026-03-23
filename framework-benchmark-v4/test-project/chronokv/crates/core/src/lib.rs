use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Core error type for ChronoKV operations.
#[derive(Debug, thiserror::Error)]
pub enum ChronoError {
    #[error("key not found: {0}")]
    KeyNotFound(String),
    #[error("version conflict for key {key}: expected {expected}, got {actual}")]
    VersionConflict {
        key: String,
        expected: u64,
        actual: u64,
    },
    #[error("WAL corruption: {0}")]
    WalCorruption(String),
    #[error("compaction error: {0}")]
    CompactionError(String),
    #[error("replication error: {0}")]
    ReplicationError(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("query error: {0}")]
    QueryError(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Timestamp in seconds since Unix epoch.
pub type Timestamp = f64;

/// Get the current timestamp as seconds since epoch.
pub fn now_timestamp() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

/// A versioned key used for ordering entries in the memtable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedKey {
    pub key: String,
    pub timestamp: u64, // nanoseconds for ordering precision
}

impl VersionedKey {
    pub fn new(key: String, timestamp: u64) -> Self {
        Self { key, timestamp }
    }
}

impl Ord for VersionedKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key
            .cmp(&other.key)
            .then(self.timestamp.cmp(&other.timestamp))
    }
}

impl PartialOrd for VersionedKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// The type of operation recorded in the WAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    Put,
    Delete,
}

/// A single entry in the key-value store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub key: String,
    pub value: Vec<u8>,
    pub timestamp: Timestamp,
    pub op_type: OpType,
    /// When the entry was deleted (only set for tombstones).
    pub deleted_at: Option<Timestamp>,
    /// Optional TTL in seconds. None means no expiry.
    pub ttl: Option<u64>,
}

impl Entry {
    pub fn put(key: String, value: Vec<u8>, timestamp: Timestamp) -> Self {
        Self {
            key,
            value,
            timestamp,
            op_type: OpType::Put,
            deleted_at: None,
            ttl: None,
        }
    }

    pub fn delete(key: String, timestamp: Timestamp) -> Self {
        Self {
            key,
            value: Vec::new(),
            timestamp,
            op_type: OpType::Delete,
            deleted_at: Some(timestamp),
            ttl: None,
        }
    }

    pub fn is_tombstone(&self) -> bool {
        self.op_type == OpType::Delete
    }
}

/// A WAL entry includes the data entry plus a checksum for integrity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub entry: Entry,
    pub checksum: u32,
}

/// Time range for queries.
#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start: Timestamp,
    pub end: Timestamp,
}

impl TimeRange {
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self { start, end }
    }

    /// Check if a timestamp falls within this range (inclusive both ends).
    pub fn contains(&self, ts: Timestamp) -> bool {
        ts >= self.start && ts <= self.end
    }
}

/// Aggregation types for query results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AggregationType {
    Min,
    Max,
    Avg,
    Count,
    Sum,
}

/// Result of an aggregation query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationResult {
    pub agg_type: AggregationType,
    pub value: f64,
    pub count: u64,
}

/// Replication term number.
pub type Term = u64;

/// Replication node identifier.
pub type NodeId = String;

/// The state of a replication node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Leader,
    Follower,
    Candidate,
}

/// A replication message sent between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplicationMessage {
    /// Heartbeat from leader to follower.
    Heartbeat {
        term: Term,
        leader_id: NodeId,
        last_entry_timestamp: Timestamp,
    },
    /// Request to replicate entries.
    AppendEntries {
        term: Term,
        leader_id: NodeId,
        entries: Vec<Entry>,
    },
    /// Response to AppendEntries.
    AppendResponse {
        term: Term,
        node_id: NodeId,
        success: bool,
    },
    /// Catch-up request from follower.
    CatchUpRequest {
        term: Term,
        node_id: NodeId,
        last_known_timestamp: Timestamp,
    },
    /// Catch-up response from leader.
    CatchUpResponse {
        term: Term,
        entries: Vec<Entry>,
    },
}

/// API request/response types shared with TypeScript SDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PutRequest {
    pub key: String,
    pub value: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResponse {
    pub key: String,
    pub value: Vec<u8>,
    pub timestamp: Timestamp,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRequest {
    pub key_prefix: Option<String>,
    pub time_range: Option<(Timestamp, Timestamp)>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    pub entries: Vec<GetResponse>,
    pub total_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_key_ordering() {
        let k1 = VersionedKey::new("a".to_string(), 100);
        let k2 = VersionedKey::new("a".to_string(), 200);
        let k3 = VersionedKey::new("b".to_string(), 50);

        assert!(k1 < k2); // same key, k1 has lower timestamp
        assert!(k2 < k3); // different keys, "a" < "b"
    }

    #[test]
    fn test_entry_creation() {
        let entry = Entry::put("test".to_string(), b"value".to_vec(), 1000.0);
        assert_eq!(entry.key, "test");
        assert!(!entry.is_tombstone());

        let tombstone = Entry::delete("test".to_string(), 1001.0);
        assert!(tombstone.is_tombstone());
        assert_eq!(tombstone.deleted_at, Some(1001.0));
    }

    #[test]
    fn test_time_range() {
        let range = TimeRange::new(100.0, 200.0);
        assert!(range.contains(100.0));
        assert!(range.contains(150.0));
        assert!(range.contains(200.0));
        assert!(!range.contains(99.9));
        assert!(!range.contains(200.1));
    }
}
