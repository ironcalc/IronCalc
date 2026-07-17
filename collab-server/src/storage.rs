//! Per-room persistence: a snapshot (one full-state yrs update) plus an
//! append-only log of incremental updates, compacted when the log grows.
//!
//! Only the document is persisted — awareness (presence) is ephemeral by
//! design. Log entries are length-prefixed; a torn tail (crash mid-append)
//! is detected on load and truncated away.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Compact once the log outgrows this many bytes.
const DEFAULT_COMPACT_LIMIT: u64 = 1024 * 1024;

pub struct Storage {
    snapshot_path: PathBuf,
    log: File,
    log_bytes: u64,
    compact_limit: u64,
}

impl Storage {
    /// Opens (or creates) the persistence files for `room` under `dir` and
    /// returns the storage handle plus the updates to replay, in order
    /// (snapshot first, then the surviving log entries).
    pub fn open(dir: &Path, room: &str) -> io::Result<(Storage, Vec<Vec<u8>>)> {
        Self::open_with_limit(dir, room, DEFAULT_COMPACT_LIMIT)
    }

    pub fn open_with_limit(
        dir: &Path,
        room: &str,
        compact_limit: u64,
    ) -> io::Result<(Storage, Vec<Vec<u8>>)> {
        fs::create_dir_all(dir)?;
        let snapshot_path = dir.join(format!("{room}.snapshot"));
        let log_path = dir.join(format!("{room}.log"));

        let mut replay = Vec::new();
        match fs::read(&snapshot_path) {
            Ok(bytes) if !bytes.is_empty() => replay.push(bytes),
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }

        let (entries, good_bytes) = match fs::read(&log_path) {
            Ok(bytes) => read_log(&bytes),
            Err(e) if e.kind() == io::ErrorKind::NotFound => (Vec::new(), 0),
            Err(e) => return Err(e),
        };
        replay.extend(entries);

        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        // Drop a torn tail so future appends start at a clean boundary.
        if log.metadata()?.len() > good_bytes {
            log.set_len(good_bytes)?;
        }

        Ok((
            Storage {
                snapshot_path,
                log,
                log_bytes: good_bytes,
                compact_limit,
            },
            replay,
        ))
    }

    /// Appends one update to the log.
    pub fn append(&mut self, update: &[u8]) -> io::Result<()> {
        let len = u32::try_from(update.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "update too large"))?;
        self.log.write_all(&len.to_le_bytes())?;
        self.log.write_all(update)?;
        self.log.flush()?;
        self.log_bytes += 4 + update.len() as u64;
        Ok(())
    }

    pub fn needs_compaction(&self) -> bool {
        self.log_bytes > self.compact_limit
    }

    /// Replaces the snapshot with `full_state` (written to a temporary file
    /// and renamed, so a crash never leaves a half-written snapshot) and
    /// truncates the log.
    pub fn compact(&mut self, full_state: &[u8]) -> io::Result<()> {
        let tmp_path = self.snapshot_path.with_extension("snapshot.tmp");
        {
            let mut tmp = File::create(&tmp_path)?;
            tmp.write_all(full_state)?;
            tmp.sync_all()?;
        }
        fs::rename(&tmp_path, &self.snapshot_path)?;
        self.log.set_len(0)?;
        self.log_bytes = 0;
        Ok(())
    }

    #[cfg(test)]
    pub fn log_len(&self) -> u64 {
        self.log_bytes
    }
}

/// Parses length-prefixed log entries; returns them plus the byte count of
/// the well-formed prefix (everything after it is a torn tail).
fn read_log(bytes: &[u8]) -> (Vec<Vec<u8>>, u64) {
    let mut entries = Vec::new();
    let mut offset = 0usize;
    while let Some(header) = bytes.get(offset..offset + 4) {
        let len = u32::from_le_bytes(header.try_into().expect("4 bytes")) as usize;
        let Some(entry) = bytes.get(offset + 4..offset + 4 + len) else {
            break;
        };
        entries.push(entry.to_vec());
        offset += 4 + len;
    }
    (entries, offset as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ironcalc-collab-storage-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn roundtrip_snapshot_and_log() {
        let dir = scratch_dir("roundtrip");
        {
            let (mut storage, replay) = Storage::open(&dir, "room").unwrap();
            assert!(replay.is_empty());
            storage.append(b"one").unwrap();
            storage.append(b"two").unwrap();
        }
        let (mut storage, replay) = Storage::open(&dir, "room").unwrap();
        assert_eq!(replay, vec![b"one".to_vec(), b"two".to_vec()]);
        storage.compact(b"snapshot").unwrap();
        storage.append(b"three").unwrap();
        let (_, replay) = Storage::open(&dir, "room").unwrap();
        assert_eq!(replay, vec![b"snapshot".to_vec(), b"three".to_vec()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn torn_tail_is_truncated() {
        let dir = scratch_dir("torn");
        {
            let (mut storage, _) = Storage::open(&dir, "room").unwrap();
            storage.append(b"good").unwrap();
        }
        let log_path = dir.join("room.log");
        // A crash mid-append: length prefix promises more bytes than exist.
        let mut log = OpenOptions::new().append(true).open(&log_path).unwrap();
        log.write_all(&[9, 0, 0, 0, b'x']).unwrap();
        drop(log);
        let (mut storage, replay) = Storage::open(&dir, "room").unwrap();
        assert_eq!(replay, vec![b"good".to_vec()]);
        // The torn bytes are gone and appends resume cleanly.
        storage.append(b"next").unwrap();
        let (_, replay) = Storage::open(&dir, "room").unwrap();
        assert_eq!(replay, vec![b"good".to_vec(), b"next".to_vec()]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compaction_threshold() {
        let dir = scratch_dir("threshold");
        let (mut storage, _) = Storage::open_with_limit(&dir, "room", 16).unwrap();
        assert!(!storage.needs_compaction());
        storage.append(b"0123456789abcdef").unwrap();
        assert!(storage.needs_compaction());
        storage.compact(b"full").unwrap();
        assert!(!storage.needs_compaction());
        assert_eq!(storage.log_len(), 0);
        let _ = fs::remove_dir_all(&dir);
    }
}
