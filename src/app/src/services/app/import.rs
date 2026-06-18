use super::import_preview::build_import_preview;
use super::AppServices;
use crate::types::{ImportPreview, ImportSummary};
use anyhow::{Context, Result};
use eterea_core::Ingester;
use std::path::Path;

impl AppServices {
    pub fn import_file(&self, path: &Path) -> Result<usize> {
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_file(path)
            .with_context(|| format!("failed to parse import file at {}", path.display()))?;
        self.db
            .insert_bookmarks(&bookmarks)
            .with_context(|| format!("failed to store imported bookmarks from {}", path.display()))
    }

    pub fn preview_import_file(&self, path: &Path) -> Result<ImportPreview> {
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_file(path)
            .with_context(|| format!("failed to preview import file at {}", path.display()))?;
        Ok(build_import_preview(path, &bookmarks))
    }

    pub fn import_file_with_preview(&self, path: &Path) -> Result<ImportSummary> {
        let preview = self.preview_import_file(path)?;
        let imported_count = self.import_file(path)?;
        Ok(ImportSummary {
            preview,
            imported_count,
        })
    }

    pub fn import_content(&self, filename: &str, content: &str) -> Result<usize> {
        let extension = Path::new(filename)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_content(extension, content)
            .with_context(|| format!("failed to parse imported content for {filename}"))?;
        self.db
            .insert_bookmarks(&bookmarks)
            .with_context(|| format!("failed to store imported bookmarks for {filename}"))
    }

    pub fn preview_import_content(&self, filename: &str, content: &str) -> Result<ImportPreview> {
        let path = Path::new(filename);
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_content(extension, content)
            .with_context(|| format!("failed to preview imported content for {filename}"))?;
        Ok(build_import_preview(path, &bookmarks))
    }

    pub fn import_content_with_preview(
        &self,
        filename: &str,
        content: &str,
    ) -> Result<ImportSummary> {
        let preview = self.preview_import_content(filename, content)?;
        let imported_count = self.import_content(filename, content)?;
        Ok(ImportSummary {
            preview,
            imported_count,
        })
    }
}
