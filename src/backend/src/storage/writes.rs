//! Bookmark write and mutation persistence.

use super::database::Database;
use crate::models::Bookmark;
use crate::{Error, Result};
use rusqlite::{params, CachedStatement, Connection};
use tracing::debug;

struct BookmarkInsertStatements<'conn> {
    bookmark: CachedStatement<'conn>,
    tag: CachedStatement<'conn>,
    tag_id: CachedStatement<'conn>,
    bookmark_tag: CachedStatement<'conn>,
    media: CachedStatement<'conn>,
    fts_content: CachedStatement<'conn>,
}

impl<'conn> BookmarkInsertStatements<'conn> {
    fn prepare(conn: &'conn Connection) -> Result<Self> {
        Ok(Self {
            bookmark: conn.prepare_cached(
                r#"INSERT INTO bookmarks
                   (id, tweet_url, content, note_text, tweeted_at, imported_at,
                    author_handle, author_name, author_profile_url, author_profile_image,
                    comments, is_favorite, has_media)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"#,
            )?,
            tag: conn.prepare_cached("INSERT OR IGNORE INTO tags (name) VALUES (?1)")?,
            tag_id: conn.prepare_cached("SELECT id FROM tags WHERE name = ?1")?,
            bookmark_tag: conn.prepare_cached(
                "INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES (?1, ?2)",
            )?,
            media: conn.prepare_cached(
                r#"INSERT INTO media
                   (bookmark_id, url, media_type, alt_text, width, height,
                    source_media_key, source_type, preview_url, variant_url, variants_json)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            )?,
            fts_content: conn.prepare_cached(
                r#"INSERT INTO bookmarks_fts_content
                   (bookmark_id, content, note_text, author_handle, author_name, tags_text)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            )?,
        })
    }
}

impl Database {
    pub fn insert_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<usize> {
        let conn = &self.conn;
        conn.execute("BEGIN IMMEDIATE", [])?;

        let insert_result = (|| -> Result<usize> {
            let mut statements = BookmarkInsertStatements::prepare(conn)?;
            let mut count = 0;

            for bookmark in bookmarks {
                match Self::insert_bookmark_internal(bookmark, &mut statements) {
                    Ok(_) => count += 1,
                    Err(Error::Database(rusqlite::Error::SqliteFailure(err, _)))
                        if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                    {
                        // Skip duplicates (same tweet_url). Do not log the URL:
                        // imported bookmark URLs are user archive data.
                        debug!(target: "eterea::db", "skipping duplicate bookmark");
                    }
                    Err(error) => return Err(error),
                }
            }

            Ok(count)
        })();

        let count = match insert_result {
            Ok(count) => count,
            Err(error) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(error);
            }
        };

        if let Err(error) = self.refresh_stats_snapshot() {
            let _ = conn.execute("ROLLBACK", []);
            return Err(error);
        }
        conn.execute("COMMIT", [])?;
        Ok(count)
    }

    fn insert_bookmark_internal(
        bookmark: &Bookmark,
        statements: &mut BookmarkInsertStatements<'_>,
    ) -> Result<()> {
        let has_media_flag = if bookmark.media.is_empty() {
            0i32
        } else {
            1i32
        };
        statements.bookmark.execute(params![
            bookmark.id,
            bookmark.tweet_url,
            bookmark.content,
            bookmark.note_text,
            bookmark.tweeted_at.timestamp(),
            bookmark.imported_at.timestamp(),
            bookmark.author_handle,
            bookmark.author_name,
            bookmark.author_profile_url,
            bookmark.author_profile_image,
            bookmark.comments,
            bookmark.is_favorite as i32,
            has_media_flag,
        ])?;

        for tag in &bookmark.tags {
            statements.tag.execute(params![tag])?;
            let tag_id: i64 = statements
                .tag_id
                .query_row(params![tag], |row| row.get(0))?;
            statements
                .bookmark_tag
                .execute(params![bookmark.id, tag_id])?;
        }

        for media in &bookmark.media {
            statements.media.execute(params![
                bookmark.id,
                media.url,
                media.media_type.as_storage_str(),
                media.alt_text.as_deref(),
                media.width,
                media.height,
                media.source_media_key.as_deref(),
                media.source_type.as_deref(),
                media.preview_url.as_deref(),
                media.variant_url.as_deref(),
                media.variants_json.as_deref(),
            ])?;
        }

        let tags_text = bookmark.tags.join(" ");
        statements.fts_content.execute(params![
            bookmark.id,
            bookmark.content,
            bookmark.note_text,
            bookmark.author_handle,
            bookmark.author_name,
            tags_text,
        ])?;

        Ok(())
    }

    pub fn delete_bookmark(&self, id: &str) -> Result<bool> {
        let count = self
            .conn
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        if count > 0 {
            self.refresh_stats_snapshot()?;
        }
        Ok(count > 0)
    }

    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        self.conn.execute(
            "UPDATE bookmarks SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )?;

        let is_favorite: bool = self.conn.query_row(
            "SELECT is_favorite FROM bookmarks WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        self.refresh_stats_snapshot()?;
        Ok(is_favorite)
    }

    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET is_favorite = ?2 WHERE id = ?1",
            params![id, favorite as i32],
        )?;
        self.refresh_stats_snapshot()?;
        Ok(())
    }
}
