//! Database connection opening and path selection.

use super::database::{Database, SQLITE_BUSY_TIMEOUT};
use crate::Result;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tracing::info;

impl Database {
    pub fn open_default() -> Result<Self> {
        let path = Self::default_path();
        Self::open(&path)
    }

    /// Get the default database path
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eterea")
            .join("bookmarks.db")
    }

    /// Open or create database at the specified path
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!(target: "eterea::db", "opening database");
        let conn = Connection::open(path)?;
        conn.busy_timeout(SQLITE_BUSY_TIMEOUT)?;

        let db = Self { conn };
        db.initialize()?;

        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.busy_timeout(SQLITE_BUSY_TIMEOUT)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }
}
