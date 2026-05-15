use eterea_core::Bookmark;
use serde::{Deserialize, Serialize};

pub use eterea_core::storage::BookmarkStats;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, offset: usize, limit: usize) -> Self {
        let has_more = offset + items.len() < total as usize;
        Self {
            items,
            total,
            offset,
            limit,
            has_more,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BookmarkQuery {
    pub query: Option<String>,
    pub tag: Option<String>,
    pub author: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub favorites_only: bool,
    pub has_media: Option<bool>,
    pub offset: usize,
    pub limit: usize,
}

impl BookmarkQuery {
    pub fn is_filtered(&self) -> bool {
        self.query
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            || self.tag.is_some()
            || self.author.is_some()
            || self.from_date.is_some()
            || self.to_date.is_some()
            || self.favorites_only
            || self.has_media.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthorSummary {
    pub handle: String,
    pub name: String,
    pub profile_image: Option<String>,
    pub bookmark_count: i64,
    pub favorite_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopicSummary {
    pub tag: String,
    pub bookmark_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportPreviewItem {
    pub author_handle: String,
    pub content: String,
    pub tag_count: usize,
    pub has_media: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportPreview {
    pub source_label: String,
    pub format: String,
    pub bookmark_count: usize,
    pub sample: Vec<ImportPreviewItem>,
    pub duplicate_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportSummary {
    pub preview: ImportPreview,
    pub imported_count: usize,
}

pub type BookmarkPage = PaginatedResponse<Bookmark>;
