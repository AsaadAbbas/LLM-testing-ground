use chronokv_core::{Entry, Timestamp};
use std::collections::HashMap;

/// A point-in-time snapshot of the memtable state.
///
/// Snapshots provide consistent read views — entries written after the
/// snapshot was taken are not visible.
pub struct Snapshot {
    pub timestamp: Timestamp,
    entries: HashMap<String, Entry>,
}

impl Snapshot {
    /// Create a snapshot from a list of entries at a given timestamp.
    pub fn from_entries(entries: Vec<Entry>, timestamp: Timestamp) -> Self {
        let mut latest: HashMap<String, Entry> = HashMap::new();

        for entry in entries {
            // Only include entries at or before the snapshot timestamp
            if entry.timestamp <= timestamp {
                let should_replace = match latest.get(&entry.key) {
                    None => true,
                    Some(existing) => entry.timestamp > existing.timestamp,
                };

                if should_replace {
                    latest.insert(entry.key.clone(), entry);
                }
            }
        }

        Self {
            timestamp,
            entries: latest,
        }
    }

    /// Get a value from the snapshot.
    pub fn get(&self, key: &str) -> Option<&Entry> {
        self.entries.get(key).filter(|e| !e.is_tombstone())
    }

    /// Get all non-tombstone entries in the snapshot.
    pub fn entries(&self) -> Vec<&Entry> {
        self.entries
            .values()
            .filter(|e| !e.is_tombstone())
            .collect()
    }

    /// Get the number of live entries.
    pub fn len(&self) -> usize {
        self.entries.values().filter(|e| !e.is_tombstone()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronokv_core::Entry;

    #[test]
    fn test_snapshot_point_in_time() {
        let entries = vec![
            Entry::put("a".to_string(), b"v1".to_vec(), 100.0),
            Entry::put("a".to_string(), b"v2".to_vec(), 200.0),
            Entry::put("b".to_string(), b"v3".to_vec(), 300.0),
        ];

        // Snapshot at 150 should only see a@v1
        let snap = Snapshot::from_entries(entries, 150.0);
        assert_eq!(snap.len(), 1);

        let a = snap.get("a").unwrap();
        assert_eq!(a.value, b"v1");

        assert!(snap.get("b").is_none());
    }
}
