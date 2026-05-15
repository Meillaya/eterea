//! Storage module for bookmark persistence
//!
//! Uses SQLite with FTS5 for lightning-fast full-text search.

mod database;
mod queries;
mod schema;

pub use database::Database;
pub use queries::{AuthorStats, BookmarkStats};
