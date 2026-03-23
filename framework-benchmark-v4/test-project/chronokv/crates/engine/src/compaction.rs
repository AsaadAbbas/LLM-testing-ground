use chronokv_core::{ChronoError, Timestamp, now_timestamp};
use crate::memtable::Memtable;

/// Run compaction on the memtable, removing old tombstones that have exceeded
/// the retention period.
///
/// Tombstones (deletion markers) are kept for a retention period so that
/// replication and queries can see that a key was deleted. After the retention
/// period, the tombstone can be safely removed.
///
/// Returns the number of entries removed.
pub fn compact(memtable: &mut Memtable, retention_seconds: f64) -> Result<usize, ChronoError> {
    let now = now_timestamp();

    let removed = memtable.remove_where(|entry| {
        if !entry.is_tombstone() {
            return false;
        }

        // Check if the tombstone is old enough to remove.
        // We compare the entry's creation timestamp against the retention window.
        let age = now - entry.timestamp;
        age > retention_seconds
    });

    tracing::debug!("Compaction removed {} tombstones", removed);
    Ok(removed)
}

/// Check if an entry has expired based on its TTL.
/// Returns true if the entry should be treated as expired.
pub fn is_expired(entry: &chronokv_core::Entry, now: Timestamp) -> bool {
    if let Some(ttl) = entry.ttl {
        let expiry = entry.timestamp + ttl as f64;
        now > expiry
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronokv_core::Entry;
    use crate::memtable::Memtable;

    #[test]
    fn test_compaction_removes_old_tombstones() {
        let mut memtable = Memtable::new();

        // Insert a put entry and an old tombstone
        memtable.insert(Entry::put("key1".to_string(), b"v1".to_vec(), 100.0));

        let mut tombstone = Entry::delete("key2".to_string(), 50.0);
        tombstone.deleted_at = Some(50.0); // deleted a long time ago
        memtable.insert(tombstone);

        assert_eq!(memtable.len(), 2);

        // With a very short retention (0.0), the old tombstone should be removed
        let removed = compact(&mut memtable, 0.0).unwrap();

        // Should remove the tombstone (ts=50 is old enough)
        assert!(removed > 0);
        assert_eq!(memtable.len(), 1);
    }

    #[test]
    fn test_compaction_keeps_recent_tombstones() {
        let mut memtable = Memtable::new();

        let now = now_timestamp();
        memtable.insert(Entry::put("key1".to_string(), b"v1".to_vec(), now));

        let tombstone = Entry::delete("key2".to_string(), now);
        memtable.insert(tombstone);

        assert_eq!(memtable.len(), 2);

        // With long retention, recent tombstone should be kept
        let removed = compact(&mut memtable, 86400.0).unwrap();
        assert_eq!(removed, 0);
        assert_eq!(memtable.len(), 2);
    }
}
