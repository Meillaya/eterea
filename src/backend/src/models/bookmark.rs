//! Core bookmark data model
//!
//! This unified model supports data from multiple Twitter export formats.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a single Twitter/X bookmark with all associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Unique identifier (UUID v4)
    pub id: String,
    
    /// Tweet URL (serves as natural unique key)
    pub tweet_url: String,
    
    /// Tweet content/text
    pub content: String,
    
    /// Extended note tweet text (if available)
    pub note_text: Option<String>,
    
    /// When the tweet was originally posted
    pub tweeted_at: DateTime<Utc>,
    
    /// When this bookmark was imported into Eterea
    pub imported_at: DateTime<Utc>,
    
    /// Author information
    pub author_handle: String,
    pub author_name: String,
    pub author_profile_url: Option<String>,
    pub author_profile_image: Option<String>,
    
    /// Categorization
    pub tags: Vec<String>,
    
    /// User-added comments/notes
    pub comments: Option<String>,
    
    /// Media attachments (images, videos)
    pub media: Vec<Media>,
    
    /// Whether this bookmark is marked as favorite
    pub is_favorite: bool,
    
    /// Full-text search content (precomputed for FTS5)
    #[serde(skip)]
    pub search_text: String,
}

/// Media attachment (image or video)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Media {
    pub url: String,
    pub media_type: MediaType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MediaType {
    Image,
    Video,
    Gif,
    Unknown,
}

/// Author information (denormalized for speed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub handle: String,
    pub name: String,
    pub profile_url: Option<String>,
    pub profile_image: Option<String>,
}

impl Bookmark {
    /// Create a new bookmark with generated ID and current import timestamp
    pub fn new(
        tweet_url: String,
        content: String,
        tweeted_at: DateTime<Utc>,
        author_handle: String,
        author_name: String,
    ) -> Self {
        let mut bookmark = Self {
            id: Uuid::new_v4().to_string(),
            tweet_url,
            content,
            note_text: None,
            tweeted_at,
            imported_at: Utc::now(),
            author_handle,
            author_name,
            author_profile_url: None,
            author_profile_image: None,
            tags: Vec::new(),
            comments: None,
            media: Vec::new(),
            is_favorite: false,
            search_text: String::new(),
        };
        bookmark.compute_search_text();
        bookmark
    }
    
    /// Compute the full-text search content
    pub fn compute_search_text(&mut self) {
        let mut parts = vec![
            self.content.clone(),
            self.author_handle.clone(),
            self.author_name.clone(),
        ];
        
        if let Some(ref note) = self.note_text {
            parts.push(note.clone());
        }
        
        if let Some(ref comments) = self.comments {
            parts.push(comments.clone());
        }
        
        for tag in &self.tags {
            parts.push(tag.clone());
        }
        
        self.search_text = parts.join(" ");
    }
    
    /// Extract hashtags from content
    pub fn extract_hashtags(&self) -> Vec<String> {
        let re = regex::Regex::new(r"#(\w+)").unwrap();
        re.captures_iter(&self.content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_lowercase()))
            .collect()
    }
    
    /// Extract mentions from content
    pub fn extract_mentions(&self) -> Vec<String> {
        let re = regex::Regex::new(r"@(\w+)").unwrap();
        re.captures_iter(&self.content)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_lowercase()))
            .collect()
    }
}

/// Builder pattern for constructing bookmarks from various sources
#[derive(Debug, Default)]
pub struct BookmarkBuilder {
    tweet_url: Option<String>,
    content: Option<String>,
    note_text: Option<String>,
    tweeted_at: Option<DateTime<Utc>>,
    author_handle: Option<String>,
    author_name: Option<String>,
    author_profile_url: Option<String>,
    author_profile_image: Option<String>,
    tags: Vec<String>,
    comments: Option<String>,
    media: Vec<Media>,
}

impl BookmarkBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn tweet_url(mut self, url: impl Into<String>) -> Self {
        self.tweet_url = Some(url.into());
        self
    }
    
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
    
    pub fn note_text(mut self, note: impl Into<String>) -> Self {
        let note = note.into();
        if !note.is_empty() {
            self.note_text = Some(note);
        }
        self
    }
    
    pub fn tweeted_at(mut self, dt: DateTime<Utc>) -> Self {
        self.tweeted_at = Some(dt);
        self
    }
    
    pub fn author_handle(mut self, handle: impl Into<String>) -> Self {
        self.author_handle = Some(handle.into());
        self
    }
    
    pub fn author_name(mut self, name: impl Into<String>) -> Self {
        self.author_name = Some(name.into());
        self
    }
    
    pub fn author_profile_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        if !url.is_empty() {
            self.author_profile_url = Some(url);
        }
        self
    }
    
    pub fn author_profile_image(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        if !url.is_empty() {
            self.author_profile_image = Some(url);
        }
        self
    }
    
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        if !tag.is_empty() && !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }
    
    pub fn comments(mut self, comments: impl Into<String>) -> Self {
        let comments = comments.into();
        if !comments.is_empty() {
            self.comments = Some(comments);
        }
        self
    }
    
    pub fn media(mut self, media: Vec<Media>) -> Self {
        self.media = media;
        self
    }
    
    pub fn add_media(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        if !url.is_empty() {
            let media_type = Self::detect_media_type(&url);
            self.media.push(Media { url, media_type });
        }
        self
    }
    
    fn detect_media_type(url: &str) -> MediaType {
        let lower = url.to_lowercase();
        if lower.contains(".gif") || lower.contains("gif") {
            MediaType::Gif
        } else if lower.contains(".mp4") || lower.contains("video") {
            MediaType::Video
        } else if lower.contains(".jpg") || lower.contains(".jpeg") 
            || lower.contains(".png") || lower.contains(".webp") 
            || lower.contains("pbs.twimg.com") {
            MediaType::Image
        } else {
            MediaType::Unknown
        }
    }
    
    pub fn build(self) -> Result<Bookmark, &'static str> {
        let tweet_url = self.tweet_url.ok_or("tweet_url is required")?;
        let content = self.content.unwrap_or_default();
        let tweeted_at = self.tweeted_at.ok_or("tweeted_at is required")?;
        let author_handle = self.author_handle.ok_or("author_handle is required")?;
        let author_name = self.author_name.unwrap_or_else(|| author_handle.clone());
        
        let mut bookmark = Bookmark::new(
            tweet_url,
            content,
            tweeted_at,
            author_handle,
            author_name,
        );
        
        bookmark.note_text = self.note_text;
        bookmark.author_profile_url = self.author_profile_url;
        bookmark.author_profile_image = self.author_profile_image;
        bookmark.tags = self.tags;
        bookmark.comments = self.comments;
        bookmark.media = self.media;
        bookmark.is_favorite = false;
        bookmark.compute_search_text();
        
        Ok(bookmark)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bookmark_builder() {
        let bookmark = BookmarkBuilder::new()
            .tweet_url("https://twitter.com/user/status/123")
            .content("Hello world! #rust @rustlang")
            .tweeted_at(Utc::now())
            .author_handle("rustacean")
            .author_name("Rust Developer")
            .add_tag("programming")
            .build()
            .unwrap();
        
        assert_eq!(bookmark.author_handle, "rustacean");
        assert_eq!(bookmark.tags, vec!["programming"]);
        
        let hashtags = bookmark.extract_hashtags();
        assert_eq!(hashtags, vec!["rust"]);
        
        let mentions = bookmark.extract_mentions();
        assert_eq!(mentions, vec!["rustlang"]);
    }
}

