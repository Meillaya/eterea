use super::AppServices;
use anyhow::{Context, Result};

impl AppServices {
    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        self.db
            .toggle_favorite(id)
            .with_context(|| format!("failed to toggle favorite for bookmark {id}"))
    }

    pub fn delete_bookmark(&self, id: &str) -> Result<bool> {
        self.db
            .delete_bookmark(id)
            .with_context(|| format!("failed to delete bookmark {id}"))
    }
}
