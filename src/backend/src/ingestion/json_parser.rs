//! JSON parsing for Twitter bookmark exports

use crate::models::{Bookmark, BookmarkBuilder};
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, warn};

/// Parser for JSON bookmark exports
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        
        // Try to parse as array first
        let raw_bookmarks: Vec<RawJsonBookmark> = serde_json::from_reader(reader)?;
        
        let mut bookmarks = Vec::new();
        for (idx, raw) in raw_bookmarks.into_iter().enumerate() {
            match self.convert_raw(raw) {
                Ok(bookmark) => bookmarks.push(bookmark),
                Err(e) => {
                    warn!("Skipping JSON entry {}: {}", idx, e);
                }
            }
        }
        
        debug!("Parsed {} bookmarks from JSON", bookmarks.len());
        Ok(bookmarks)
    }
    
    fn convert_raw(&self, raw: RawJsonBookmark) -> Result<Bookmark> {
        let tweeted_at = self.parse_date(&raw.tweeted_at.or(raw.created_at))?;
        
        let tweet_url = raw.tweet_url
            .or(raw.url)
            .ok_or_else(|| Error::Other("Missing tweet URL".into()))?;
        
        let author_handle = raw.screen_name
            .or(raw.author_handle)
            .or(raw.username)
            .unwrap_or_default();
        
        let author_name = raw.name
            .or(raw.author_name)
            .or(raw.display_name)
            .unwrap_or_else(|| author_handle.clone());
        
        let content = raw.full_text
            .or(raw.text)
            .or(raw.content)
            .unwrap_or_default();
        
        let mut builder = BookmarkBuilder::new()
            .tweet_url(tweet_url)
            .content(content)
            .note_text(raw.note_tweet_text.unwrap_or_default())
            .tweeted_at(tweeted_at)
            .author_handle(author_handle)
            .author_name(author_name);
        
        if let Some(img) = raw.profile_image_url_https.or(raw.profile_image) {
            builder = builder.author_profile_image(img);
        }
        
        if let Some(tags) = raw.tags {
            for tag in tags {
                builder = builder.add_tag(tag);
            }
        }
        
        if let Some(media) = raw.media {
            for m in media {
                if let Some(url) = m.url.or(m.media_url) {
                    builder = builder.add_media(url);
                }
            }
        }
        
        builder.build().map_err(|e| Error::Other(e.to_string()))
    }
    
    fn parse_date(&self, date_str: &Option<String>) -> Result<DateTime<Utc>> {
        let s = date_str.as_ref()
            .ok_or_else(|| Error::Other("Missing date".into()))?;
        
        // Try RFC3339 first
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        // Try Twitter's format: "Wed Oct 10 20:19:24 +0000 2018"
        if let Ok(dt) = DateTime::parse_from_str(s, "%a %b %d %H:%M:%S %z %Y") {
            return Ok(dt.with_timezone(&Utc));
        }
        
        Err(Error::Other(format!("Could not parse date: {}", s)))
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Raw JSON bookmark structure that handles multiple formats
#[derive(Debug, Deserialize)]
struct RawJsonBookmark {
    // Tweet URL variants
    tweet_url: Option<String>,
    url: Option<String>,
    
    // Content variants
    full_text: Option<String>,
    text: Option<String>,
    content: Option<String>,
    
    // Note text
    note_tweet_text: Option<String>,
    
    // Date variants
    tweeted_at: Option<String>,
    created_at: Option<String>,
    
    // Author handle variants
    screen_name: Option<String>,
    author_handle: Option<String>,
    username: Option<String>,
    
    // Author name variants
    name: Option<String>,
    author_name: Option<String>,
    display_name: Option<String>,
    
    // Profile image variants
    profile_image_url_https: Option<String>,
    profile_image: Option<String>,
    
    // Tags
    tags: Option<Vec<String>>,
    
    // Media
    media: Option<Vec<RawMedia>>,
}

#[derive(Debug, Deserialize)]
struct RawMedia {
    url: Option<String>,
    media_url: Option<String>,
    #[serde(rename = "type")]
    _media_type: Option<String>,
}

