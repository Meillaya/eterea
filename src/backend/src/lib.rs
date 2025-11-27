//! Eterea Core - Lightning-fast Twitter bookmarks manager
//!
//! This library provides the core functionality for ingesting, processing,
//! storing, and searching Twitter bookmarks with maximum performance.

pub mod models;
pub mod ingestion;
pub mod storage;
pub mod search;
pub mod error;

pub use error::{Error, Result};
pub use models::Bookmark;
pub use storage::Database;
pub use ingestion::Ingester;

