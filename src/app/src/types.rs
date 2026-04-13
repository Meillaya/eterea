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

pub type BookmarkPage = PaginatedResponse<Bookmark>;
