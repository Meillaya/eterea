//! Ingestion module for importing bookmarks from various formats
//!
//! Supports:
//! - Legacy CSV format (Dewey export)
//! - New CSV format (Twitter/X export)
//! - JSON format

mod csv_parser;
mod json_parser;

pub use csv_parser::{LegacyCsvParser, NewCsvParser, CsvFormat};
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
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        match extension.as_str() {
            "csv" => self.ingest_csv(path, db),
            "json" => self.ingest_json(path, db),
            _ => Err(Error::UnsupportedFileType(extension)),
        }
    }
    
    /// Ingest from CSV, auto-detecting the format variant
    fn ingest_csv(&self, path: &Path, db: &Database) -> Result<usize> {
        let format = CsvFormat::detect(path)?;
        info!("Detected CSV format: {:?}", format);
        
        let bookmarks: Vec<Bookmark> = match format {
            CsvFormat::Legacy => {
                let parser = LegacyCsvParser::new();
                parser.parse(path)?
            }
            CsvFormat::New => {
                let parser = NewCsvParser::new();
                parser.parse(path)?
            }
        };
        
        self.insert_bookmarks(bookmarks, db)
    }
    
    /// Ingest from JSON
    fn ingest_json(&self, path: &Path, db: &Database) -> Result<usize> {
        let parser = JsonParser::new();
        let bookmarks = parser.parse(path)?;
        self.insert_bookmarks(bookmarks, db)
    }
    
    /// Insert bookmarks in batches for optimal performance
    fn insert_bookmarks(&self, bookmarks: Vec<Bookmark>, db: &Database) -> Result<usize> {
        let total = bookmarks.len();
        info!("Inserting {} bookmarks in batches of {}", total, self.batch_size);
        
        let mut inserted = 0;
        for chunk in bookmarks.chunks(self.batch_size) {
            inserted += db.insert_bookmarks(chunk)?;
        }
        
        info!("Successfully inserted {} bookmarks", inserted);
        Ok(inserted)
    }
}

