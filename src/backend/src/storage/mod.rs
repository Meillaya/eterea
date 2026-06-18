//! Storage module for bookmark persistence
//!
//! Uses SQLite with FTS5 for lightning-fast full-text search.

mod connection;
mod database;
mod directory_queries;
mod filtered_queries;
mod hydration;
mod metadata;
mod migrations;
mod queries;
mod read_queries;
mod schema;
mod stats;
mod writes;

pub use database::Database;
pub use queries::{AuthorStats, BookmarkStats};
