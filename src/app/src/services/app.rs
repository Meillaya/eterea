use crate::types::{BookmarkPage, BookmarkQuery, BookmarkStats};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use eterea_core::{Database, Ingester};
use std::path::Path;

pub struct AppServices {
    db: Database,
}

impl AppServices {
    pub fn open_default() -> Result<Self> {
        let db = Database::open_default().context("failed to open default Eterea database")?;
        Ok(Self { db })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::open(path)
            .with_context(|| format!("failed to open database at {}", path.display()))?;
        Ok(Self { db })
    }

    pub fn open_memory() -> Result<Self> {
        let db = Database::open_memory().context("failed to open in-memory database")?;
        Ok(Self { db })
    }

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

    pub fn stats(&self) -> Result<BookmarkStats> {
        self.db.get_stats().context("failed to load bookmark stats")
    }

    pub fn import_file(&self, path: &Path) -> Result<usize> {
        let ingester = Ingester::new();
        ingester
            .ingest_file(path, &self.db)
            .with_context(|| format!("failed to import file at {}", path.display()))
    }

    pub fn import_content(&self, filename: &str, content: &str) -> Result<usize> {
        let extension = Path::new(filename)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_content(extension, content)
            .with_context(|| format!("failed to parse imported content for {filename}"))?;
        self.db
            .insert_bookmarks(&bookmarks)
            .with_context(|| format!("failed to store imported bookmarks for {filename}"))
    }

    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        self.db
            .toggle_favorite(id)
            .with_context(|| format!("failed to toggle favorite for bookmark {id}"))
    }

    pub fn delete_bookmark(&self, id: &str) -> Result<bool> {
        self.db
            .delete_bookmark(id)
            .with_context(|| format!("failed to delete bookmark {id}"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn sample_json() -> &'static str {
        include_str!("../../../legacy/new_bookmarks.json")
    }

    #[test]
    fn imports_content_and_surfaces_stats() {
        let services = AppServices::open_memory().expect("in-memory services should open");

        let imported = services
            .import_content("sample.json", sample_json())
            .expect("json import should succeed");
        assert!(imported > 0, "expected imported bookmarks");

        let stats = services.stats().expect("stats should load");
        assert!(stats.total_bookmarks >= imported as i64);
        assert!(stats.unique_authors > 0);
    }

    #[test]
    fn filters_bookmarks_by_query_and_tag() {
        let services = AppServices::open_memory().expect("in-memory services should open");
        services
            .import_content("sample.json", sample_json())
            .expect("json import should succeed");

        let first_page = services
            .query_bookmarks(&BookmarkQuery {
                query: Some("rust".to_string()),
                limit: 20,
                ..BookmarkQuery::default()
            })
            .expect("query should succeed");
        assert!(
            !first_page.items.is_empty(),
            "expected at least one rust result"
        );

        let tagged = services
            .query_bookmarks(&BookmarkQuery {
                tag: first_page.items[0].tags.first().cloned(),
                limit: 20,
                ..BookmarkQuery::default()
            })
            .expect("tag query should succeed");
        assert!(!tagged.items.is_empty(), "expected tagged results");
    }

    #[test]
    fn persists_to_disk_for_restart_like_workflow() {
        let file = NamedTempFile::new().expect("temp file should exist");
        let path = file.path().to_path_buf();
        drop(file);

        {
            let services = AppServices::open(&path).expect("disk-backed services should open");
            services
                .import_content("sample.json", sample_json())
                .expect("json import should succeed");
        }

        let reopened = AppServices::open(&path).expect("reopened services should open");
        let page = reopened.list_bookmarks(0, 20).expect("page should load");
        assert!(
            !page.items.is_empty(),
            "expected persisted bookmarks after reopen"
        );
    }
}
