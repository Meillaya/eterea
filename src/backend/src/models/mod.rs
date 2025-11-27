//! Data models for Eterea
//!
//! Provides a unified bookmark model that can be populated from various
//! source formats (legacy CSV, new CSV, JSON exports).

mod bookmark;

pub use bookmark::{Bookmark, BookmarkBuilder, Media, MediaType, Author};

