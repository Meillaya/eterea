//! Basic bookmark read queries.

use super::database::Database;
use crate::models::Bookmark;
use crate::Result;
use rusqlite::params;
use tracing::debug;

impl Database {
    pub fn get_bookmarks(&self, offset: usize, limit: usize) -> Result<Vec<Bookmark>> {
        let overall_started = std::time::Instant::now();
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               ORDER BY tweeted_at DESC, id DESC
               LIMIT ?1 OFFSET ?2"#,
        )?;

        let query_started = std::time::Instant::now();
        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let query_elapsed = query_started.elapsed();

        let hydrate_started = std::time::Instant::now();
        self.hydrate_bookmarks(&mut bookmarks)?;
        debug!(
            target: "eterea::db",
            offset,
            limit,
            rows = bookmarks.len(),
            query_ms = query_elapsed.as_millis(),
            hydrate_ms = hydrate_started.elapsed().as_millis(),
            total_ms = overall_started.elapsed().as_millis(),
            "loaded bookmark page"
        );

        Ok(bookmarks)
    }

    pub fn count_bookmarks(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))
            .map_err(Into::into)
    }

    /// Get bookmarks by tag
    pub fn get_bookmarks_by_tag(
        &self,
        tag: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT b.id, b.tweet_url, b.content, b.note_text, b.tweeted_at, b.imported_at,
                      b.author_handle, b.author_name, b.author_profile_url, b.author_profile_image,
                      b.comments, b.is_favorite
               FROM bookmarks b
               JOIN bookmark_tags bt ON bt.bookmark_id = b.id
               JOIN tags t ON t.id = bt.tag_id
               WHERE t.name = ?1
               ORDER BY b.tweeted_at DESC, b.id DESC
               LIMIT ?2 OFFSET ?3"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![tag, limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }

    /// Get bookmarks by author
    pub fn get_bookmarks_by_author(
        &self,
        handle: &str,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               WHERE author_handle = ?1
               ORDER BY tweeted_at DESC, id DESC
               LIMIT ?2 OFFSET ?3"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![handle, limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }

    /// Get a single bookmark by ID
    pub fn get_bookmark(&self, id: &str) -> Result<Option<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks WHERE id = ?1"#,
        )?;

        let result = stmt.query_row(params![id], |row| self.row_to_bookmark(row));

        match result {
            Ok(mut bookmark) => {
                bookmark.tags = self.load_bookmark_tags(&bookmark.id)?;
                bookmark.media = self.load_bookmark_media(&bookmark.id)?;
                Ok(Some(bookmark))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all favorite bookmarks
    pub fn get_favorites(&self, offset: usize, limit: usize) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               WHERE is_favorite = 1
               ORDER BY tweeted_at DESC, id DESC
               LIMIT ?1 OFFSET ?2"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }

    /// Get bookmarks within a date range
    pub fn get_bookmarks_by_date_range(
        &self,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<Bookmark>> {
        let from_ts = from.map(|d| d.timestamp()).unwrap_or(0);
        let to_ts = to.map(|d| d.timestamp()).unwrap_or(i64::MAX);

        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               WHERE tweeted_at >= ?1 AND tweeted_at <= ?2
               ORDER BY tweeted_at DESC, id DESC
               LIMIT ?3 OFFSET ?4"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(
                params![from_ts, to_ts, limit as i64, offset as i64],
                |row| self.row_to_bookmark(row),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }
}
