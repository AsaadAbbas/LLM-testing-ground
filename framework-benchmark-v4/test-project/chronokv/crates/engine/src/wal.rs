use chronokv_core::{ChronoError, Entry, WalEntry};
use crc32fast::Hasher;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Write-ahead log for crash recovery.
///
/// Each entry is serialized as JSON on a single line, with a CRC32 checksum
/// for integrity verification during recovery.
pub struct WriteAheadLog {
    path: PathBuf,
    file: Option<File>,
}

impl WriteAheadLog {
    pub fn new(path: &str) -> Result<Self, ChronoError> {
        let path = PathBuf::from(path);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self {
            path,
            file: Some(file),
        })
    }

    /// Compute checksum for an entry.
    /// We hash the serialized value data to detect corruption.
    fn compute_checksum(entry: &Entry) -> u32 {
        let mut hasher = Hasher::new();
        // Hash the value payload for integrity
        hasher.update(&entry.value);
        hasher.finalize()
    }

    /// Append an entry to the WAL.
    pub fn append(&mut self, entry: &Entry) -> Result<(), ChronoError> {
        let wal_entry = WalEntry {
            entry: entry.clone(),
            checksum: Self::compute_checksum(entry),
        };

        let serialized = serde_json::to_string(&wal_entry)
            .map_err(|e| ChronoError::SerializationError(e.to_string()))?;

        if let Some(ref mut file) = self.file {
            writeln!(file, "{}", serialized)?;
            file.flush()?;
        }

        Ok(())
    }

    /// Recover entries from the WAL file, verifying checksums.
    pub fn recover(&self) -> Result<Vec<Entry>, ChronoError> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let wal_entry: WalEntry = serde_json::from_str(&line)
                .map_err(|e| ChronoError::WalCorruption(
                    format!("line {}: parse error: {}", line_num + 1, e)
                ))?;

            // Verify checksum integrity
            let expected = Self::compute_checksum(&wal_entry.entry);
            if wal_entry.checksum != expected {
                tracing::warn!(
                    "WAL corruption detected at line {}: checksum mismatch (expected {}, got {})",
                    line_num + 1,
                    expected,
                    wal_entry.checksum
                );
                continue; // Skip corrupted entries
            }

            entries.push(wal_entry.entry);
        }

        Ok(entries)
    }

    /// Clear the WAL (after successful compaction/snapshot).
    pub fn clear(&mut self) -> Result<(), ChronoError> {
        // Close current file
        self.file.take();

        // Truncate
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        self.file = Some(file);
        Ok(())
    }

    /// Get the WAL file size in bytes.
    pub fn size(&self) -> Result<u64, ChronoError> {
        let metadata = std::fs::metadata(&self.path)?;
        Ok(metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronokv_core::Entry;
    use std::fs;

    fn temp_wal_path() -> String {
        let id = uuid::Uuid::new_v4();
        format!("/tmp/chronokv_test_wal_{}", id)
    }

    #[test]
    fn test_wal_append_and_recover() {
        let path = temp_wal_path();

        {
            let mut wal = WriteAheadLog::new(&path).unwrap();
            let entry = Entry::put("key1".to_string(), b"value1".to_vec(), 1000.0);
            wal.append(&entry).unwrap();
        }

        {
            let wal = WriteAheadLog::new(&path).unwrap();
            let entries = wal.recover().unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].key, "key1");
            assert_eq!(entries[0].value, b"value1");
        }

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_wal_multiple_entries() {
        let path = temp_wal_path();

        {
            let mut wal = WriteAheadLog::new(&path).unwrap();
            for i in 0..5 {
                let entry = Entry::put(
                    format!("key{}", i),
                    format!("value{}", i).into_bytes(),
                    1000.0 + i as f64,
                );
                wal.append(&entry).unwrap();
            }
        }

        let wal = WriteAheadLog::new(&path).unwrap();
        let entries = wal.recover().unwrap();
        assert_eq!(entries.len(), 5);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_wal_clear() {
        let path = temp_wal_path();

        let mut wal = WriteAheadLog::new(&path).unwrap();
        let entry = Entry::put("key1".to_string(), b"value1".to_vec(), 1000.0);
        wal.append(&entry).unwrap();
        wal.clear().unwrap();

        let entries = wal.recover().unwrap();
        assert_eq!(entries.len(), 0);

        fs::remove_file(&path).ok();
    }
}
