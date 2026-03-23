use chronokv_core::{Entry, TimeRange, Timestamp, VersionedKey};
use std::collections::BTreeMap;

/// In-memory sorted storage for entries.
///
/// Entries are indexed by VersionedKey (key + timestamp) for efficient
/// range queries and version lookups. The BTreeMap maintains entries
/// sorted by key first, then by timestamp for version ordering.
pub struct Memtable {
    entries: BTreeMap<VersionedKey, Entry>,
    size_bytes: usize,
}

impl Memtable {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            size_bytes: 0,
        }
    }

    /// Insert an entry into the memtable.
    pub fn insert(&mut self, entry: Entry) {
        let ts_nanos = (entry.timestamp * 1_000_000_000.0) as u64;
        let key = VersionedKey::new(entry.key.clone(), ts_nanos);
        self.size_bytes += entry.value.len() + entry.key.len();
        self.entries.insert(key, entry);
    }

    /// Get the latest version of a key.
    ///
    /// Scans through versions of the key in sort order and returns
    /// the first match found. Since entries are stored in ascending
    /// timestamp order, this efficiently returns the version.
    pub fn get_latest(&self, key: &str) -> Option<Entry> {
        let start = VersionedKey::new(key.to_string(), 0);
        let end = VersionedKey::new(key.to_string(), u64::MAX);

        // Return the first entry found for this key
        self.entries
            .range(start..=end)
            .next()
            .map(|(_, entry)| entry.clone())
    }

    /// Scan entries within a time range.
    ///
    /// Returns entries where the timestamp falls within [start, end).
    /// Only returns non-tombstone entries.
    pub fn scan(&self, key_prefix: Option<&str>, range: TimeRange) -> Vec<Entry> {
        self.entries
            .values()
            .filter(|entry| {
                // Check key prefix
                if let Some(prefix) = key_prefix {
                    if !entry.key.starts_with(prefix) {
                        return false;
                    }
                }

                // Check time range (exclusive end for scan consistency)
                let in_range = entry.timestamp >= range.start && entry.timestamp < range.end;

                // Skip tombstones
                in_range && !entry.is_tombstone()
            })
            .cloned()
            .collect()
    }

    /// Get all entries (for replication or snapshot).
    pub fn all_entries(&self) -> Vec<Entry> {
        self.entries.values().cloned().collect()
    }

    /// Get entries after a given timestamp.
    pub fn entries_after(&self, timestamp: Timestamp) -> Vec<Entry> {
        self.entries
            .values()
            .filter(|e| e.timestamp > timestamp)
            .cloned()
            .collect()
    }

    /// Remove entries matching a predicate. Returns the number removed.
    pub fn remove_where<F: Fn(&Entry) -> bool>(&mut self, predicate: F) -> usize {
        let keys_to_remove: Vec<VersionedKey> = self
            .entries
            .iter()
            .filter(|(_, entry)| predicate(entry))
            .map(|(key, _)| key.clone())
            .collect();

        let count = keys_to_remove.len();
        for key in keys_to_remove {
            if let Some(entry) = self.entries.remove(&key) {
                self.size_bytes = self.size_bytes.saturating_sub(entry.value.len() + entry.key.len());
            }
        }
        count
    }

    /// Get the total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Approximate memory usage in bytes.
    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    /// Get deduplicated entries (latest version per key, excluding tombstones).
    pub fn latest_entries(&self) -> Vec<Entry> {
        let mut latest: std::collections::HashMap<String, Entry> = std::collections::HashMap::new();

        for entry in self.entries.values() {
            let existing = latest.get(&entry.key);
            let should_replace = match existing {
                None => true,
                Some(e) => entry.timestamp > e.timestamp,
            };

            if should_replace {
                latest.insert(entry.key.clone(), entry.clone());
            }
        }

        latest
            .into_values()
            .filter(|e| !e.is_tombstone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronokv_core::{Entry, OpType};

    #[test]
    fn test_insert_and_get() {
        let mut memtable = Memtable::new();
        let entry = Entry::put("key1".to_string(), b"value1".to_vec(), 1000.0);
        memtable.insert(entry);

        let result = memtable.get_latest("key1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().value, b"value1");
    }

    #[test]
    fn test_scan_range() {
        let mut memtable = Memtable::new();
        memtable.insert(Entry::put("a".to_string(), b"v1".to_vec(), 100.0));
        memtable.insert(Entry::put("b".to_string(), b"v2".to_vec(), 200.0));
        memtable.insert(Entry::put("c".to_string(), b"v3".to_vec(), 300.0));

        let range = TimeRange::new(100.0, 250.0);
        let results = memtable.scan(None, range);
        // scan uses exclusive end: [100, 250)
        // a@100 is in range, b@200 is in range, c@300 is NOT
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_tombstone_not_in_scan() {
        let mut memtable = Memtable::new();
        memtable.insert(Entry::put("a".to_string(), b"v1".to_vec(), 100.0));
        memtable.insert(Entry::delete("a".to_string(), 150.0));

        let range = TimeRange::new(0.0, 1000.0);
        let results = memtable.scan(None, range);
        // tombstones are excluded from scan
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].op_type, OpType::Put);
    }

    #[test]
    fn test_latest_entries_dedup() {
        let mut memtable = Memtable::new();
        memtable.insert(Entry::put("a".to_string(), b"v1".to_vec(), 100.0));
        memtable.insert(Entry::put("a".to_string(), b"v2".to_vec(), 200.0));
        memtable.insert(Entry::put("b".to_string(), b"v3".to_vec(), 150.0));

        let latest = memtable.latest_entries();
        assert_eq!(latest.len(), 2);
    }
}
