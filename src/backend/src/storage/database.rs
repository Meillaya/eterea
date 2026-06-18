//! SQLite database handle and storage module tests.

use rusqlite::Connection;
use std::time::Duration;

pub(super) const STATS_SNAPSHOT_METADATA_KEY: &str = "stats_snapshot_v1";
pub(super) const AUTHOR_STATS_METADATA_KEY: &str = "author_stats_v1";
pub(super) const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Main database handle.
pub struct Database {
    pub(super) conn: Connection,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Bookmark, BookmarkBuilder};
    use chrono::{TimeZone, Utc};

    fn sample_bookmark(
        tweet_id: &str,
        handle: &str,
        date: chrono::DateTime<Utc>,
        tag: &str,
        with_media: bool,
    ) -> Bookmark {
        let mut builder = BookmarkBuilder::new()
            .tweet_url(format!("https://x.com/{handle}/status/{tweet_id}"))
            .content(format!("Bookmark {tweet_id} #{tag}"))
            .tweeted_at(date)
            .author_handle(handle)
            .author_name(handle)
            .add_tag(tag);

        if with_media {
            builder = builder.add_media("https://pbs.twimg.com/media/example.jpg");
        }

        builder.build().unwrap()
    }

    fn insert_bookmark_with_invalid_tweeted_at(db: &Database, id: &str) {
        db.conn
            .execute(
                r#"INSERT INTO bookmarks
                   (id, tweet_url, content, note_text, tweeted_at, imported_at,
                    author_handle, author_name, author_profile_url, author_profile_image,
                    comments, is_favorite, has_media)
                   VALUES (?1, ?2, 'bad persisted timestamp', NULL, ?3, 0,
                           'corrupt_author', 'Corrupt Author', NULL, NULL, NULL, 0, 0)"#,
                rusqlite::params![
                    id,
                    format!("https://x.com/corrupt_author/status/{id}"),
                    i64::MAX
                ],
            )
            .unwrap();
    }

    fn assert_invalid_timestamp_error<T>(result: crate::Result<T>) {
        let error = match result {
            Ok(_) => panic!("expected invalid timestamp error"),
            Err(error) => error,
        };
        let message = error.to_string();
        assert!(
            message.contains("timestamp") || message.contains("out of range"),
            "expected invalid timestamp error, got: {message}"
        );
    }

    #[test]
    fn search_with_filters_is_parameterized_and_filters_correctly() {
        let db = Database::open_memory().unwrap();
        let first = sample_bookmark(
            "1",
            "alice",
            Utc.with_ymd_and_hms(2024, 5, 1, 12, 0, 0).unwrap(),
            "rust",
            true,
        );
        let second = sample_bookmark(
            "2",
            "bob",
            Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap(),
            "svelte",
            false,
        );

        db.insert_bookmarks(&[first.clone(), second.clone()])
            .unwrap();
        db.set_favorite(&first.id, true).unwrap();

        let results = db
            .search_with_filters(
                Some("Bookmark"),
                Some("rust"),
                Some("alice"),
                Some(Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap()),
                Some(Utc.with_ymd_and_hms(2024, 5, 31, 23, 59, 59).unwrap()),
                true,
                Some(true),
                20,
            )
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].author_handle, "alice");

        let injection_attempt = db
            .search_with_filters(Some("' OR 1=1 --"), None, None, None, None, false, None, 20)
            .unwrap();
        assert!(injection_attempt.is_empty());
    }

    #[test]
    fn get_bookmarks_batches_related_tags_and_media() {
        let db = Database::open_memory().unwrap();
        let first = sample_bookmark(
            "1",
            "alice",
            Utc.with_ymd_and_hms(2024, 5, 1, 12, 0, 0).unwrap(),
            "rust",
            true,
        );
        let mut second = sample_bookmark(
            "2",
            "bob",
            Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap(),
            "svelte",
            false,
        );
        second.tags.push("frontend".to_string());

        db.insert_bookmarks(&[first.clone(), second.clone()])
            .unwrap();

        let bookmarks = db.get_bookmarks(0, 10).unwrap();

        assert_eq!(db.count_bookmarks().unwrap(), 2);
        assert_eq!(bookmarks.len(), 2);
        assert_eq!(bookmarks[0].author_handle, "bob");
        assert_eq!(
            bookmarks[0].tags,
            vec!["frontend".to_string(), "svelte".to_string()]
        );
        assert!(bookmarks[0].media.is_empty());
        assert_eq!(bookmarks[1].author_handle, "alice");
        assert_eq!(bookmarks[1].tags, vec!["rust".to_string()]);
        assert_eq!(bookmarks[1].media.len(), 1);
    }

    #[test]
    fn get_bookmarks_returns_error_when_persisted_timestamp_is_invalid() {
        // Given: a persisted bookmark row whose timestamp cannot be represented as UTC.
        let db = Database::open_memory().unwrap();
        insert_bookmark_with_invalid_tweeted_at(&db, "invalid-list");

        // When: the page/list read maps persisted rows into bookmark models.
        let result = db.get_bookmarks(0, 10);

        // Then: the row-mapping error is returned instead of silently dropping the row.
        assert_invalid_timestamp_error(result);
    }

    #[test]
    fn get_bookmark_returns_error_when_persisted_timestamp_is_invalid() {
        // Given: a persisted bookmark row whose timestamp cannot be represented as UTC.
        let db = Database::open_memory().unwrap();
        insert_bookmark_with_invalid_tweeted_at(&db, "invalid-direct");

        // When: direct lookup maps the persisted row into a bookmark model.
        let result = db.get_bookmark("invalid-direct");

        // Then: the row-mapping error is returned without panicking.
        assert_invalid_timestamp_error(result);
    }

    #[test]
    fn get_stats_returns_error_when_persisted_timestamp_is_invalid() {
        // Given: a persisted bookmark row whose timestamp cannot be represented as UTC.
        let db = Database::open_memory().unwrap();
        insert_bookmark_with_invalid_tweeted_at(&db, "invalid-stats");

        // When: stats compute the persisted bookmark date range.
        let result = db.get_stats();

        // Then: the invalid timestamp is returned as an error without panicking.
        assert_invalid_timestamp_error(result);
    }

    #[test]
    fn stats_snapshot_stays_fresh_after_writes() {
        let db = Database::open_memory().unwrap();
        let first = sample_bookmark(
            "1",
            "alice",
            Utc.with_ymd_and_hms(2024, 5, 1, 12, 0, 0).unwrap(),
            "rust",
            false,
        );
        let second = sample_bookmark(
            "2",
            "bob",
            Utc.with_ymd_and_hms(2024, 6, 1, 12, 0, 0).unwrap(),
            "svelte",
            true,
        );

        db.insert_bookmarks(&[first.clone(), second.clone()])
            .unwrap();

        let initial = db.get_stats().unwrap();
        assert_eq!(initial.total_bookmarks, 2);
        assert_eq!(initial.favorite_bookmarks, 0);

        db.toggle_favorite(&second.id).unwrap();
        let after_favorite = db.get_stats().unwrap();
        assert_eq!(after_favorite.favorite_bookmarks, 1);

        db.delete_bookmark(&first.id).unwrap();
        let after_delete = db.get_stats().unwrap();
        assert_eq!(after_delete.total_bookmarks, 1);
        assert_eq!(after_delete.unique_authors, 1);
        assert_eq!(after_delete.favorite_bookmarks, 1);
    }

    #[test]
    fn author_stats_snapshot_stays_hot_after_writes() {
        let db = Database::open_memory().unwrap();
        let first = sample_bookmark(
            "1",
            "alice",
            Utc.with_ymd_and_hms(2024, 5, 1, 12, 0, 0).unwrap(),
            "rust",
            false,
        );
        let second = sample_bookmark(
            "2",
            "alice",
            Utc.with_ymd_and_hms(2024, 5, 2, 12, 0, 0).unwrap(),
            "rust",
            false,
        );
        let third = sample_bookmark(
            "3",
            "bob",
            Utc.with_ymd_and_hms(2024, 5, 3, 12, 0, 0).unwrap(),
            "systems",
            false,
        );

        db.insert_bookmarks(&[first.clone(), second.clone(), third.clone()])
            .unwrap();

        let initial = db.get_all_authors().unwrap();
        assert_eq!(initial.len(), 2);
        assert_eq!(initial[0].handle, "alice");
        assert_eq!(initial[0].bookmark_count, 2);
        assert_eq!(initial[0].favorite_count, 0);

        db.toggle_favorite(&first.id).unwrap();
        let after_favorite = db.get_all_authors().unwrap();
        let alice = after_favorite
            .iter()
            .find(|author| author.handle == "alice")
            .unwrap();
        assert_eq!(alice.favorite_count, 1);

        db.delete_bookmark(&second.id).unwrap();
        let after_delete = db.get_all_authors().unwrap();
        let alice = after_delete
            .iter()
            .find(|author| author.handle == "alice")
            .unwrap();
        assert_eq!(alice.bookmark_count, 1);
        assert_eq!(alice.favorite_count, 1);
    }

    #[test]
    fn database_sets_explicit_busy_timeout() {
        let db = Database::open_memory().unwrap();
        let busy_timeout_ms: i64 = db
            .conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();

        assert_eq!(busy_timeout_ms, SQLITE_BUSY_TIMEOUT.as_millis() as i64);
    }
}
