//! Bookmark row mapping and related tag/media hydration helpers.

use super::database::Database;
use crate::models::{Bookmark, Media, MediaType};
use crate::Result;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::types::Type;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter};
use std::collections::HashMap;
use tracing::debug;

impl Database {
    pub(crate) fn hydrate_bookmarks(&self, bookmarks: &mut [Bookmark]) -> Result<()> {
        if bookmarks.is_empty() {
            return Ok(());
        }

        let overall_started = std::time::Instant::now();
        let bookmark_ids = bookmarks
            .iter()
            .map(|bookmark| bookmark.id.clone())
            .collect::<Vec<_>>();
        let tags_started = std::time::Instant::now();
        let tags_by_bookmark = self.load_tags_for_bookmarks(&bookmark_ids)?;
        let tags_elapsed = tags_started.elapsed();
        let media_started = std::time::Instant::now();
        let media_by_bookmark = self.load_media_for_bookmarks(&bookmark_ids)?;
        let media_elapsed = media_started.elapsed();

        for bookmark in bookmarks {
            bookmark.tags = tags_by_bookmark
                .get(&bookmark.id)
                .cloned()
                .unwrap_or_default();
            bookmark.media = media_by_bookmark
                .get(&bookmark.id)
                .cloned()
                .unwrap_or_default();
        }
        debug!(
            target: "eterea::db",
            bookmarks = bookmark_ids.len(),
            tags_ms = tags_elapsed.as_millis(),
            media_ms = media_elapsed.as_millis(),
            total_ms = overall_started.elapsed().as_millis(),
            "hydrated bookmark related data"
        );
        Ok(())
    }

    fn load_tags_for_bookmarks(
        &self,
        bookmark_ids: &[String],
    ) -> Result<HashMap<String, Vec<String>>> {
        if bookmark_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = vec!["?"; bookmark_ids.len()].join(", ");
        let sql = format!(
            r#"SELECT bt.bookmark_id, t.name
               FROM bookmark_tags bt
               JOIN tags t ON t.id = bt.tag_id
               WHERE bt.bookmark_id IN ({placeholders})
               ORDER BY bt.bookmark_id, t.name"#
        );
        let params = bookmark_ids
            .iter()
            .cloned()
            .map(Value::Text)
            .collect::<Vec<_>>();
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params.iter()))?;
        let mut tags_by_bookmark = HashMap::<String, Vec<String>>::new();

        while let Some(row) = rows.next()? {
            let bookmark_id: String = row.get(0)?;
            let tag: String = row.get(1)?;
            tags_by_bookmark.entry(bookmark_id).or_default().push(tag);
        }

        Ok(tags_by_bookmark)
    }

    fn load_media_for_bookmarks(
        &self,
        bookmark_ids: &[String],
    ) -> Result<HashMap<String, Vec<Media>>> {
        if bookmark_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = vec!["?"; bookmark_ids.len()].join(", ");
        let sql = format!(
            r#"SELECT bookmark_id, url, media_type, alt_text, width, height,
                      source_media_key, source_type, preview_url, variant_url, variants_json
               FROM media
               WHERE bookmark_id IN ({placeholders})
               ORDER BY bookmark_id, id"#
        );
        let params = bookmark_ids
            .iter()
            .cloned()
            .map(Value::Text)
            .collect::<Vec<_>>();
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params_from_iter(params.iter()))?;
        let mut media_by_bookmark = HashMap::<String, Vec<Media>>::new();

        while let Some(row) = rows.next()? {
            let bookmark_id: String = row.get(0)?;
            let media = media_from_row(row, 1)?;

            media_by_bookmark
                .entry(bookmark_id)
                .or_default()
                .push(media);
        }

        Ok(media_by_bookmark)
    }

    pub(crate) fn row_to_bookmark(&self, row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
        let id: String = row.get(0)?;
        let tweeted_at_ts: i64 = row.get(4)?;
        let imported_at_ts: i64 = row.get(5)?;
        let is_favorite: i32 = row.get(11)?;

        let bookmark = Bookmark {
            id: id.clone(),
            tweet_url: row.get(1)?,
            content: row.get(2)?,
            note_text: row.get(3)?,
            tweeted_at: utc_timestamp_from_row(tweeted_at_ts, 4)?,
            imported_at: utc_timestamp_from_row(imported_at_ts, 5)?,
            author_handle: row.get(6)?,
            author_name: row.get(7)?,
            author_profile_url: row.get(8)?,
            author_profile_image: row.get(9)?,
            comments: row.get(10)?,
            tags: Vec::new(),
            media: Vec::new(),
            is_favorite: is_favorite != 0,
            search_text: String::new(),
        };

        // Note: tags and media are loaded separately for performance
        // Use load_bookmark_tags() and load_bookmark_media() when needed

        Ok(bookmark)
    }

    /// Load tags for a bookmark
    pub fn load_bookmark_tags(&self, bookmark_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT t.name FROM tags t
               JOIN bookmark_tags bt ON bt.tag_id = t.id
               WHERE bt.bookmark_id = ?1"#,
        )?;

        let tags = stmt
            .query_map(params![bookmark_id], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }

    /// Load media for a bookmark
    pub fn load_bookmark_media(&self, bookmark_id: &str) -> Result<Vec<Media>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT url, media_type, alt_text, width, height,
                          source_media_key, source_type, preview_url, variant_url, variants_json
                   FROM media
                   WHERE bookmark_id = ?1
                   ORDER BY id"#,
        )?;

        let media = stmt
            .query_map(params![bookmark_id], |row| media_from_row(row, 0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(media)
    }
}

fn media_from_row(row: &rusqlite::Row<'_>, start: usize) -> rusqlite::Result<Media> {
    let media_type: String = row.get(start + 1)?;
    Ok(Media {
        url: row.get(start)?,
        media_type: MediaType::from_storage_str(&media_type),
        alt_text: row.get(start + 2)?,
        width: row.get(start + 3)?,
        height: row.get(start + 4)?,
        source_media_key: row.get(start + 5)?,
        source_type: row.get(start + 6)?,
        preview_url: row.get(start + 7)?,
        variant_url: row.get(start + 8)?,
        variants_json: row.get(start + 9)?,
    })
}

fn utc_timestamp_from_row(timestamp: i64, column: usize) -> rusqlite::Result<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single().ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            Type::Integer,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid UTC timestamp stored in bookmarks: {timestamp}"),
            )),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn insert_bookmark_with_malformed_favorite(db: &Database, id: &str) {
        db.conn
            .execute(
                r#"INSERT INTO bookmarks
                   (id, tweet_url, content, note_text, tweeted_at, imported_at,
                    author_handle, author_name, author_profile_url, author_profile_image,
                    comments, is_favorite, has_media)
                   VALUES (?1, ?2, 'bad favorite flag', NULL, 1714564800, 1714564800,
                           'corrupt_author', 'Corrupt Author', NULL, NULL, NULL,
                           'malformed-favorite', 0)"#,
                params![id, format!("https://x.com/corrupt_author/status/{id}")],
            )
            .unwrap();
    }

    #[test]
    fn get_bookmark_returns_error_when_persisted_is_favorite_is_malformed() {
        // Given: a persisted bookmark row whose favorite flag is non-integer data.
        let db = Database::open_memory().unwrap();
        insert_bookmark_with_malformed_favorite(&db, "malformed-favorite");

        // When: direct lookup maps the persisted row into a bookmark model.
        let result = db.get_bookmark("malformed-favorite");

        // Then: the row conversion error is returned instead of defaulting false.
        let error = result.expect_err("expected malformed is_favorite error");
        let message = error.to_string();
        assert!(
            message.contains("is_favorite") || message.contains("Invalid column type"),
            "expected malformed is_favorite error, got: {message}"
        );
    }
}
