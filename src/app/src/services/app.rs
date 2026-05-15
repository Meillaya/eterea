use crate::types::{
    AuthorSummary, BookmarkPage, BookmarkQuery, BookmarkStats, ImportPreview, ImportPreviewItem,
    ImportSummary, TopicSummary,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use eterea_core::{Bookmark, Database, Ingester};
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

    pub fn author_index(&self) -> Result<Vec<AuthorSummary>> {
        self.db
            .get_all_authors()
            .context("failed to load author index")
            .map(|authors| {
                authors
                    .into_iter()
                    .map(|author| AuthorSummary {
                        handle: author.handle,
                        name: author.name,
                        profile_image: author.profile_image,
                        bookmark_count: author.bookmark_count,
                        favorite_count: author.favorite_count,
                    })
                    .collect()
            })
    }

    pub fn topic_index(&self) -> Result<Vec<TopicSummary>> {
        self.db
            .get_all_tags()
            .context("failed to load topic index")
            .map(|topics| {
                topics
                    .into_iter()
                    .map(|(tag, bookmark_count)| TopicSummary {
                        tag,
                        bookmark_count,
                    })
                    .collect()
            })
    }

    pub fn bookmark_detail(&self, id: &str) -> Result<Option<eterea_core::Bookmark>> {
        self.db
            .get_bookmark(id)
            .with_context(|| format!("failed to load bookmark detail for {id}"))
    }

    pub fn bookmarks_by_author(
        &self,
        handle: &str,
        offset: usize,
        limit: usize,
    ) -> Result<BookmarkPage> {
        let (items, total) = self
            .db
            .search_with_filters_page(
                None,
                None,
                Some(handle),
                None,
                None,
                false,
                None,
                offset,
                limit,
            )
            .with_context(|| format!("failed to load bookmarks for author {handle}"))?;
        Ok(BookmarkPage::new(items, total, offset, limit))
    }

    pub fn bookmarks_by_tag(&self, tag: &str, offset: usize, limit: usize) -> Result<BookmarkPage> {
        let (items, total) = self
            .db
            .search_with_filters_page(
                None,
                Some(tag),
                None,
                None,
                None,
                false,
                None,
                offset,
                limit,
            )
            .with_context(|| format!("failed to load bookmarks for tag {tag}"))?;
        Ok(BookmarkPage::new(items, total, offset, limit))
    }

    pub fn stats(&self) -> Result<BookmarkStats> {
        self.db.get_stats().context("failed to load bookmark stats")
    }

    pub fn import_file(&self, path: &Path) -> Result<usize> {
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_file(path)
            .with_context(|| format!("failed to parse import file at {}", path.display()))?;
        self.db
            .insert_bookmarks(&bookmarks)
            .with_context(|| format!("failed to store imported bookmarks from {}", path.display()))
    }

    pub fn preview_import_file(&self, path: &Path) -> Result<ImportPreview> {
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_file(path)
            .with_context(|| format!("failed to preview import file at {}", path.display()))?;
        Ok(build_import_preview(path, &bookmarks))
    }

    pub fn import_file_with_preview(&self, path: &Path) -> Result<ImportSummary> {
        let preview = self.preview_import_file(path)?;
        let imported_count = self.import_file(path)?;
        Ok(ImportSummary {
            preview,
            imported_count,
        })
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

    pub fn preview_import_content(&self, filename: &str, content: &str) -> Result<ImportPreview> {
        let path = Path::new(filename);
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_content(extension, content)
            .with_context(|| format!("failed to preview imported content for {filename}"))?;
        Ok(build_import_preview(path, &bookmarks))
    }

    pub fn import_content_with_preview(
        &self,
        filename: &str,
        content: &str,
    ) -> Result<ImportSummary> {
        let preview = self.preview_import_content(filename, content)?;
        let imported_count = self.import_content(filename, content)?;
        Ok(ImportSummary {
            preview,
            imported_count,
        })
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

fn build_import_preview(path: &Path, bookmarks: &[Bookmark]) -> ImportPreview {
    let source_label = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("bookmark export")
        .to_string();
    let format = path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_ascii_uppercase();
    let sample = bookmarks
        .iter()
        .take(5)
        .map(|bookmark| ImportPreviewItem {
            author_handle: bookmark.author_handle.clone(),
            content: preview_content(&bookmark.content),
            tag_count: bookmark.tags.len(),
            has_media: !bookmark.media.is_empty(),
        })
        .collect();

    ImportPreview {
        source_label,
        format,
        bookmark_count: bookmarks.len(),
        sample,
        duplicate_policy:
            "Existing tweet URLs are skipped; a failed parse or write leaves the archive unchanged."
                .to_string(),
    }
}

fn preview_content(content: &str) -> String {
    const LIMIT: usize = 140;
    let mut preview: String = content.chars().take(LIMIT).collect();
    if content.chars().count() > LIMIT {
        preview.push('…');
    }
    preview
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Local, LocalResult, NaiveDate, TimeZone, Utc};
    use tempfile::NamedTempFile;

    fn sample_json() -> &'static str {
        include_str!("../../../legacy/new_bookmarks.json")
    }

    fn local_boundary(date: NaiveDate, end_of_day: bool) -> DateTime<Utc> {
        let naive = if end_of_day {
            date.and_hms_opt(23, 59, 59)
        } else {
            date.and_hms_opt(0, 0, 0)
        }
        .expect("date boundary should be valid");

        let local = match Local.from_local_datetime(&naive) {
            LocalResult::Single(boundary) => boundary,
            LocalResult::Ambiguous(earliest, latest) => {
                if end_of_day {
                    latest
                } else {
                    earliest
                }
            }
            LocalResult::None => panic!("local timezone could not represent boundary"),
        };

        local.with_timezone(&Utc)
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

    #[test]
    fn filters_by_author_date_and_media_and_supports_delete() {
        let services = AppServices::open_memory().expect("in-memory services should open");
        services
            .import_content("sample.json", sample_json())
            .expect("json import should succeed");

        let seed_page = services
            .list_bookmarks(0, 200)
            .expect("seed page should load");
        let target = seed_page
            .items
            .iter()
            .find(|bookmark| !bookmark.media.is_empty())
            .cloned()
            .expect("expected at least one bookmark with media");

        let date = target.tweeted_at.with_timezone(&Local).date_naive();
        let from = local_boundary(date, false).to_rfc3339();
        let to = local_boundary(date, true).to_rfc3339();

        let filtered = services
            .query_bookmarks(&BookmarkQuery {
                author: Some(target.author_handle.clone()),
                from_date: Some(from),
                to_date: Some(to),
                has_media: Some(true),
                limit: 200,
                ..BookmarkQuery::default()
            })
            .expect("compound query should succeed");

        assert!(
            filtered
                .items
                .iter()
                .any(|bookmark| bookmark.id == target.id),
            "expected the target bookmark to remain visible under combined filters"
        );
        assert!(
            filtered
                .items
                .iter()
                .all(|bookmark| bookmark.author_handle == target.author_handle),
            "expected author filter to be respected"
        );
        assert!(
            filtered
                .items
                .iter()
                .all(|bookmark| !bookmark.media.is_empty()),
            "expected media filter to be respected"
        );

        let deleted = services
            .delete_bookmark(&target.id)
            .expect("delete should succeed");
        assert!(deleted, "expected the bookmark to be deleted");

        let after_delete = services
            .list_bookmarks(0, 200)
            .expect("page should load after delete");
        assert!(
            after_delete
                .items
                .iter()
                .all(|bookmark| bookmark.id != target.id),
            "expected the deleted bookmark to disappear from the library"
        );
        assert_eq!(after_delete.total, seed_page.total - 1);
    }
}
