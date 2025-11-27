//! CSV parsing for Twitter bookmark exports
//!
//! Handles two formats:
//! 1. Legacy (Dewey): Tweet Date, Posted By, Profile Pic, Profile URL, Handle, Tweet URL, Content, Tags, Comments, Media
//! 2. New (Twitter/X): profile_image_url_https, screen_name, name, full_text, note_tweet_text, tweeted_at, tweet_url

use crate::models::{Bookmark, BookmarkBuilder};
use crate::{Error, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use csv::ReaderBuilder;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, warn};

/// Detected CSV format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CsvFormat {
    /// Dewey export format with "Tweet Date" header
    Legacy,
    /// Twitter/X export format with "screen_name" header
    New,
}

impl CsvFormat {
    /// Detect the CSV format by examining the header row
    pub fn detect(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(BufReader::new(file));
        
        let headers = reader.headers()?;
        let header_str = headers.iter().collect::<Vec<_>>().join(",").to_lowercase();
        
        if header_str.contains("tweet date") || header_str.contains("posted by") {
            Ok(CsvFormat::Legacy)
        } else if header_str.contains("screen_name") || header_str.contains("tweeted_at") {
            Ok(CsvFormat::New)
        } else {
            Err(Error::InvalidFormat(format!(
                "Could not detect CSV format. Headers: {}",
                header_str
            )))
        }
    }
}

/// Parser for legacy Dewey CSV exports
pub struct LegacyCsvParser;

impl LegacyCsvParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let file = File::open(path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(BufReader::new(file));
        
        let mut bookmarks = Vec::new();
        
        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    match self.parse_record(&record) {
                        Ok(bookmark) => bookmarks.push(bookmark),
                        Err(e) => {
                            warn!("Skipping row {}: {}", idx + 2, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("CSV error at row {}: {}", idx + 2, e);
                }
            }
        }
        
        debug!("Parsed {} bookmarks from legacy CSV", bookmarks.len());
        Ok(bookmarks)
    }
    
    fn parse_record(&self, record: &csv::StringRecord) -> Result<Bookmark> {
        // Legacy format columns:
        // 0: Tweet Date, 1: Posted By, 2: Profile Pic, 3: Profile URL,
        // 4: Twitter Handle, 5: Tweet URL, 6: Content, 7: Tags, 8: Comments, 9: Media
        
        let tweet_date = record.get(0).unwrap_or("");
        let posted_by = record.get(1).unwrap_or("");
        let profile_pic = record.get(2).unwrap_or("");
        let profile_url = record.get(3).unwrap_or("");
        let handle = record.get(4).unwrap_or("");
        let tweet_url = record.get(5).unwrap_or("");
        let content = record.get(6).unwrap_or("");
        let tags = record.get(7).unwrap_or("");
        let comments = record.get(8).unwrap_or("");
        let media = record.get(9).unwrap_or("");
        
        let tweeted_at = self.parse_legacy_date(tweet_date)?;
        
        let mut builder = BookmarkBuilder::new()
            .tweet_url(tweet_url)
            .content(content)
            .tweeted_at(tweeted_at)
            .author_handle(handle)
            .author_name(posted_by)
            .author_profile_url(profile_url)
            .author_profile_image(profile_pic)
            .comments(comments);
        
        // Parse tags (comma-separated in legacy format)
        if !tags.is_empty() {
            for tag in tags.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    builder = builder.add_tag(tag);
                }
            }
        }
        
        // Parse media URLs (semicolon-separated in legacy format)
        if !media.is_empty() {
            for url in media.split(';') {
                let url = url.trim();
                if !url.is_empty() {
                    builder = builder.add_media(url);
                }
            }
        }
        
        builder.build().map_err(|e| Error::Other(e.to_string()))
    }
    
    /// Parse legacy date format: "02:51 PM, May 01, 2024"
    fn parse_legacy_date(&self, s: &str) -> Result<DateTime<Utc>> {
        let s = s.trim().trim_matches('"');
        
        // Try format: "02:51 PM, May 01, 2024"
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%I:%M %p, %b %d, %Y") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        
        // Try format: "May 01, 2024 02:51 PM"
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%b %d, %Y %I:%M %p") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        
        // Try ISO format as fallback
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        Err(Error::Other(format!("Could not parse date: {}", s)))
    }
}

impl Default for LegacyCsvParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for new Twitter/X CSV exports
pub struct NewCsvParser;

impl NewCsvParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let file = File::open(path)?;
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(BufReader::new(file));
        
        let mut bookmarks = Vec::new();
        
        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    match self.parse_record(&record) {
                        Ok(bookmark) => bookmarks.push(bookmark),
                        Err(e) => {
                            warn!("Skipping row {}: {}", idx + 2, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("CSV error at row {}: {}", idx + 2, e);
                }
            }
        }
        
        debug!("Parsed {} bookmarks from new CSV", bookmarks.len());
        Ok(bookmarks)
    }
    
    fn parse_record(&self, record: &csv::StringRecord) -> Result<Bookmark> {
        // New format columns:
        // 0: profile_image_url_https, 1: screen_name, 2: name,
        // 3: full_text, 4: note_tweet_text, 5: tweeted_at, 6: tweet_url
        
        let profile_image = record.get(0).unwrap_or("");
        let screen_name = record.get(1).unwrap_or("");
        let name = record.get(2).unwrap_or("");
        let full_text = record.get(3).unwrap_or("");
        let note_text = record.get(4).unwrap_or("");
        let tweeted_at = record.get(5).unwrap_or("");
        let tweet_url = record.get(6).unwrap_or("");
        
        let parsed_date = self.parse_new_date(tweeted_at)?;
        
        let builder = BookmarkBuilder::new()
            .tweet_url(tweet_url)
            .content(full_text)
            .note_text(note_text)
            .tweeted_at(parsed_date)
            .author_handle(screen_name)
            .author_name(name)
            .author_profile_image(profile_image);
        
        builder.build().map_err(|e| Error::Other(e.to_string()))
    }
    
    /// Parse new date format: "2025-08-25T10:52:35.000Z"
    fn parse_new_date(&self, s: &str) -> Result<DateTime<Utc>> {
        let s = s.trim().trim_matches('"');
        
        // ISO 8601 format
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        // Try without timezone
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
            return Ok(Utc.from_utc_datetime(&dt));
        }
        
        Err(Error::Other(format!("Could not parse date: {}", s)))
    }
}

impl Default for NewCsvParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_legacy_date_parsing() {
        let parser = LegacyCsvParser::new();
        
        let result = parser.parse_legacy_date("02:51 PM, May 01, 2024");
        assert!(result.is_ok());
        
        let dt = result.unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-05-01");
    }
    
    #[test]
    fn test_new_date_parsing() {
        let parser = NewCsvParser::new();
        
        let result = parser.parse_new_date("2025-08-25T10:52:35.000Z");
        assert!(result.is_ok());
        
        let dt = result.unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2025-08-25");
    }
}

