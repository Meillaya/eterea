//! Query result types and helpers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkStats {
    pub total_bookmarks: i64,
    pub unique_authors: i64,
    pub unique_tags: i64,
    pub favorite_bookmarks: i64,
    pub earliest_date: Option<DateTime<Utc>>,
    pub latest_date: Option<DateTime<Utc>>,
    pub top_tags: Vec<(String, i64)>,
}

/// Search filter options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    /// Full-text search query
    pub query: Option<String>,
    
    /// Filter by specific tags
    pub tags: Vec<String>,
    
    /// Filter by author handle
    pub author: Option<String>,
    
    /// Filter by date range
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    
    /// Only show bookmarks with media
    pub has_media: Option<bool>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub offset: usize,
    pub limit: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 50,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub offset: usize,
    pub limit: usize,
    pub has_more: bool,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, offset: usize, limit: usize) -> Self {
        let has_more = (offset + items.len()) < total as usize;
        Self {
            items,
            total,
            offset,
            limit,
            has_more,
        }
    }
}

