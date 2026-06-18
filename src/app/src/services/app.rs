use anyhow::{Context, Result};
use eterea_core::Database;
use std::path::Path;

mod directory_detail;
mod import;
mod import_preview;
mod mutations;
mod query;

#[cfg(test)]
mod tests;

pub struct AppServices {
    db: Database,
}

impl AppServices {
    pub fn open_default() -> Result<Self> {
        let db = Database::open_default().context("failed to open default Eterea database")?;
        Ok(Self { db })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::open(path)
            .with_context(|| format!("failed to open database at {}", path.display()))?;
        Ok(Self { db })
    }

    pub fn open_memory() -> Result<Self> {
        let db = Database::open_memory().context("failed to open in-memory database")?;
        Ok(Self { db })
    }
}
