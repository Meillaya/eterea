//! Ingestion module for importing bookmarks from various formats
//!
//! Supports:
//! - Legacy CSV format (Dewey export)
//! - New CSV format (Twitter/X export)
//! - JSON format
//! - Archive JS format (`window.YTD... = [...]`)

mod csv_parser;
mod json_parser;

pub use csv_parser::{CsvFormat, LegacyCsvParser, NewCsvParser};
pub use json_parser::JsonParser;

use crate::models::Bookmark;
use crate::storage::Database;
use crate::{Error, Result};
use std::path::Path;
use tracing::info;

/// Main ingestion engine that auto-detects format and imports bookmarks
pub struct Ingester {
    batch_size: usize,
}

impl Default for Ingester {
    fn default() -> Self {
        Self::new()
    }
}

impl Ingester {
    pub fn new() -> Self {
        Self { batch_size: 1000 }
    }

    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Ingest bookmarks from a file, auto-detecting the format
    pub fn ingest_file(&self, path: &Path, db: &Database) -> Result<usize> {
        let bookmarks = self.parse_file(path)?;
        self.insert_bookmarks(bookmarks, db)
    }

    /// Parse bookmarks from a file without inserting them.
    pub fn parse_file(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            "csv" => self.parse_csv(path),
            "json" | "js" => self.parse_json(path),
            _ => Err(Error::UnsupportedFileType(extension)),
        }
    }

    /// Parse bookmarks from raw content using the file extension.
    pub fn parse_content(&self, extension: &str, content: &str) -> Result<Vec<Bookmark>> {
        match extension.to_lowercase().as_str() {
            "csv" => self.parse_csv_content(content),
            "json" | "js" => self.parse_json_content(content),
            other => Err(Error::UnsupportedFileType(other.to_string())),
        }
    }

    /// Parse from CSV, auto-detecting the format variant
    fn parse_csv(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let format = CsvFormat::detect(path)?;
        info!("Detected CSV format: {:?}", format);

        match format {
            CsvFormat::Legacy => {
                let parser = LegacyCsvParser::new();
                parser.parse(path)
            }
            CsvFormat::New => {
                let parser = NewCsvParser::new();
                parser.parse(path)
            }
        }
    }

    /// Parse from JSON / archive-JS
    fn parse_json(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let parser = JsonParser::new();
        parser.parse(path)
    }

    fn parse_csv_content(&self, content: &str) -> Result<Vec<Bookmark>> {
        match CsvFormat::detect_from_content(content)? {
            CsvFormat::Legacy => LegacyCsvParser::new().parse_str(content),
            CsvFormat::New => NewCsvParser::new().parse_str(content),
        }
    }

    fn parse_json_content(&self, content: &str) -> Result<Vec<Bookmark>> {
        JsonParser::new().parse_str(content)
    }

    /// Insert bookmarks in batches for optimal performance
    fn insert_bookmarks(&self, bookmarks: Vec<Bookmark>, db: &Database) -> Result<usize> {
        let total = bookmarks.len();
        info!(
            "Inserting {} bookmarks in batches of {}",
            total, self.batch_size
        );

        let mut inserted = 0;
        for chunk in bookmarks.chunks(self.batch_size) {
            inserted += db.insert_bookmarks(chunk)?;
        }

        info!("Successfully inserted {} bookmarks", inserted);
        Ok(inserted)
    }
}
