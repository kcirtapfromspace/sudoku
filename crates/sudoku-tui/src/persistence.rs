//! Centralized persistence utilities: app data directory and atomic file writes.

use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Return the application data directory, creating it if needed.
/// Panics if the platform has no local data directory (should never happen on
/// macOS/Linux/Windows where `dirs` is supported).
pub fn app_data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .expect("Platform has no local data directory")
        .join("sudoku");
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create app data directory");
    }
    dir
}

/// Write data to a file atomically: write to a temporary file in the same
/// directory, then rename into place. This prevents corruption if the process
/// crashes mid-write.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
    tmp.write_all(data)?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}
