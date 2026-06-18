use super::AppServices;
use crate::types::{BookmarkPage, BookmarkQuery};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};

impl AppServices {
    pub fn list_bookmarks(&self, offset: usize, limit: usize) -> Result<BookmarkPage> {
        let items = self
            .db
            .get_bookmarks(offset, limit)
            .context("failed to list bookmarks")?;
        let total = self
            .db
            .count_bookmarks()
            .context("failed to count bookmarks")?;
        Ok(BookmarkPage::new(items, total, offset, limit))
    }

    pub fn query_bookmarks(&self, query: &BookmarkQuery) -> Result<BookmarkPage> {
        if !query.is_filtered() {
            return self.list_bookmarks(query.offset, query.limit);
        }

        let from = parse_rfc3339(query.from_date.as_deref())?;
        let to = parse_rfc3339(query.to_date.as_deref())?;
        let (items, total) = self
            .db
            .search_with_filters_page(
                normalize_filter(query.query.as_deref()),
                query.tag.as_deref(),
                query.author.as_deref(),
                from,
                to,
                query.favorites_only,
                query.has_media,
                query.offset,
                query.limit,
            )
            .context("failed to query bookmarks")?;

        Ok(BookmarkPage::new(items, total, query.offset, query.limit))
    }
}

fn parse_rfc3339(value: Option<&str>) -> Result<Option<DateTime<Utc>>> {
    value
        .filter(|candidate| !candidate.trim().is_empty())
        .map(|candidate| {
            DateTime::parse_from_rfc3339(candidate)
                .with_context(|| format!("invalid RFC3339 date: {candidate}"))
                .map(|parsed| parsed.with_timezone(&Utc))
        })
        .transpose()
}

fn normalize_filter(value: Option<&str>) -> Option<&str> {
    value.and_then(|candidate| {
        let trimmed = candidate.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}
