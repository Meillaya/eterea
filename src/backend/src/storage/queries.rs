//! Query result types and helpers

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookmarkStats {
    pub total_bookmarks: i64,
    pub unique_authors: i64,
    pub unique_tags: i64,
    pub favorite_bookmarks: i64,
    pub earliest_date: Option<DateTime<Utc>>,
    pub latest_date: Option<DateTime<Utc>>,
    pub top_tags: Vec<(String, i64)>,
}
