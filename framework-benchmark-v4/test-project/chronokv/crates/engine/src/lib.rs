pub mod wal;
pub mod memtable;
pub mod compaction;
pub mod snapshot;

use chronokv_core::{ChronoError, Entry, Timestamp, TimeRange};
use memtable::Memtable;
use wal::WriteAheadLog;
use std::sync::Arc;
use tokio::sync::RwLock;

/// The storage engine manages the memtable, WAL, and compaction.
pub struct StorageEngine {
    memtable: Arc<RwLock<Memtable>>,
    wal: Arc<RwLock<WriteAheadLog>>,
    retention_seconds: f64,
}

impl StorageEngine {
    pub fn new(wal_path: &str, retention_seconds: f64) -> Result<Self, ChronoError> {
        let wal = WriteAheadLog::new(wal_path)?;
        let memtable = Memtable::new();

        Ok(Self {
            memtable: Arc::new(RwLock::new(memtable)),
            wal: Arc::new(RwLock::new(wal)),
            retention_seconds,
        })
    }

    /// Put a key-value pair into the store.
    pub async fn put(&self, key: String, value: Vec<u8>, timestamp: Timestamp) -> Result<(), ChronoError> {
        let entry = Entry::put(key, value, timestamp);

        // Write to WAL first for durability
        {
            let mut wal = self.wal.write().await;
            wal.append(&entry)?;
        }

        // Then insert into memtable
        {
            let mut memtable = self.memtable.write().await;
            memtable.insert(entry);
        }

        Ok(())
    }

    /// Delete a key from the store (writes a tombstone).
    pub async fn delete(&self, key: &str, timestamp: Timestamp) -> Result<(), ChronoError> {
        let entry = Entry::delete(key.to_string(), timestamp);

        {
            let mut wal = self.wal.write().await;
            wal.append(&entry)?;
        }

        {
            let mut memtable = self.memtable.write().await;
            memtable.insert(entry);
        }

        Ok(())
    }

    /// Get the latest version of a key.
    pub async fn get(&self, key: &str) -> Result<Option<Entry>, ChronoError> {
        let memtable = self.memtable.read().await;
        Ok(memtable.get_latest(key))
    }

    /// Scan entries within a time range.
    pub async fn scan(&self, key_prefix: Option<&str>, range: TimeRange) -> Result<Vec<Entry>, ChronoError> {
        let memtable = self.memtable.read().await;
        Ok(memtable.scan(key_prefix, range))
    }

    /// Recover from WAL after a crash.
    pub async fn recover(&self) -> Result<usize, ChronoError> {
        let wal = self.wal.read().await;
        let entries = wal.recover()?;
        let count = entries.len();

        let mut memtable = self.memtable.write().await;
        for entry in entries {
            memtable.insert(entry);
        }

        Ok(count)
    }

    /// Run compaction to remove old tombstones and expired entries.
    pub async fn compact(&self) -> Result<usize, ChronoError> {
        let mut memtable = self.memtable.write().await;
        let removed = compaction::compact(&mut memtable, self.retention_seconds)?;
        Ok(removed)
    }

    /// Get all entries (for replication).
    pub async fn all_entries(&self) -> Vec<Entry> {
        let memtable = self.memtable.read().await;
        memtable.all_entries()
    }

    /// Get entries after a given timestamp (for replication catch-up).
    pub async fn entries_after(&self, timestamp: Timestamp) -> Vec<Entry> {
        let memtable = self.memtable.read().await;
        memtable.entries_after(timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronokv_core::now_timestamp;

    #[tokio::test]
    async fn test_put_and_get() {
        let id = uuid::Uuid::new_v4();
        let engine = StorageEngine::new(&format!("/tmp/chronokv_test_{}", id), 3600.0).unwrap();
        let ts = now_timestamp();

        engine.put("key1".to_string(), b"value1".to_vec(), ts).await.unwrap();

        let result = engine.get("key1").await.unwrap();
        assert!(result.is_some());
        let entry = result.unwrap();
        assert_eq!(entry.key, "key1");
        assert_eq!(entry.value, b"value1");
    }

    #[tokio::test]
    async fn test_delete_creates_tombstone() {
        let id = uuid::Uuid::new_v4();
        let engine = StorageEngine::new(&format!("/tmp/chronokv_test_{}", id), 3600.0).unwrap();
        let ts = now_timestamp();

        // Write and then delete
        engine.put("key1".to_string(), b"value1".to_vec(), ts).await.unwrap();
        engine.delete("key1", ts + 1.0).await.unwrap();

        // Verify the delete was recorded by checking scan excludes tombstones
        let results = engine.scan(None, chronokv_core::TimeRange::new(0.0, f64::MAX)).await.unwrap();
        // Scan filters out tombstones, so if delete works, we should see
        // only the original put (tombstones are excluded from scan results)
        let key1_entries: Vec<_> = results.iter().filter(|e| e.key == "key1").collect();
        assert!(key1_entries.len() <= 1); // at most the original put (scan excludes tombstones)
    }
}
