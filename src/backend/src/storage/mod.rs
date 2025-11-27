//! Storage module for bookmark persistence
//!
//! Uses SQLite with FTS5 for lightning-fast full-text search.

mod database;
mod schema;
mod queries;

pub use database::Database;
pub use queries::BookmarkStats;

