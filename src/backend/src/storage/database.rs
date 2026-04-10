//! SQLite database implementation

use super::queries::BookmarkStats;
use super::schema::{PRAGMAS, SCHEMA};
use crate::models::{Bookmark, Media, MediaType};
use crate::{Error, Result};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

const STATS_SNAPSHOT_METADATA_KEY: &str = "stats_snapshot_v1";

/// Main database handle
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open database at the default location
    pub fn open_default() -> Result<Self> {
        let path = Self::default_path();
        Self::open(&path)
    }

    /// Get the default database path
    pub fn default_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eterea")
            .join("bookmarks.db")
    }

    /// Open or create database at the specified path
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        info!("Opening database at: {}", path.display());
        let conn = Connection::open(path)?;

        let db = Self { conn };
        db.initialize()?;

        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn open_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        // Set performance pragmas
        self.conn.execute_batch(PRAGMAS)?;

        // Create schema
        self.conn.execute_batch(SCHEMA)?;

        self.ensure_is_favorite_column()?;

        debug!("Database initialized");
        Ok(())
    }

    fn ensure_is_favorite_column(&self) -> Result<()> {
        let mut stmt = self.conn.prepare("PRAGMA table_info(bookmarks)")?;
        let mut rows = stmt.query([])?;
        let mut has_column = false;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case("is_favorite") {
                has_column = true;
                break;
            }
        }

        if !has_column {
            self.conn.execute(
                "ALTER TABLE bookmarks ADD COLUMN is_favorite INTEGER DEFAULT 0",
                [],
            )?;
        }

        Ok(())
    }

    /// Insert multiple bookmarks in a transaction
    pub fn insert_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<usize> {
        let mut count = 0;

        // Use a transaction for batch insert
        let conn = &self.conn;
        conn.execute("BEGIN IMMEDIATE", [])?;

        for bookmark in bookmarks {
            match self.insert_bookmark_internal(bookmark) {
                Ok(_) => count += 1,
                Err(Error::Database(rusqlite::Error::SqliteFailure(err, _)))
                    if err.code == rusqlite::ErrorCode::ConstraintViolation =>
                {
                    // Skip duplicates (same tweet_url)
                    debug!("Skipping duplicate bookmark: {}", bookmark.tweet_url);
                }
                Err(e) => {
                    conn.execute("ROLLBACK", [])?;
                    return Err(e);
                }
            }
        }

        self.refresh_stats_snapshot()?;
        conn.execute("COMMIT", [])?;
        Ok(count)
    }

    fn insert_bookmark_internal(&self, bookmark: &Bookmark) -> Result<()> {
        // Insert main bookmark
        self.conn.execute(
            r#"INSERT INTO bookmarks 
               (id, tweet_url, content, note_text, tweeted_at, imported_at,
                author_handle, author_name, author_profile_url, author_profile_image, comments, is_favorite)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            params![
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
            ],
        )?;

        // Insert tags
        for tag in &bookmark.tags {
            // Insert tag if not exists
            self.conn.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![tag],
            )?;

            // Get tag ID
            let tag_id: i64 = self.conn.query_row(
                "SELECT id FROM tags WHERE name = ?1",
                params![tag],
                |row| row.get(0),
            )?;

            // Link bookmark to tag
            self.conn.execute(
                "INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES (?1, ?2)",
                params![bookmark.id, tag_id],
            )?;
        }

        // Insert media
        for media in &bookmark.media {
            let media_type = match media.media_type {
                MediaType::Image => "image",
                MediaType::Video => "video",
                MediaType::Gif => "gif",
                MediaType::Unknown => "unknown",
            };

            self.conn.execute(
                "INSERT INTO media (bookmark_id, url, media_type) VALUES (?1, ?2, ?3)",
                params![bookmark.id, media.url, media_type],
            )?;
        }

        // Insert FTS content
        let tags_text = bookmark.tags.join(" ");
        self.conn.execute(
            r#"INSERT INTO bookmarks_fts_content 
               (bookmark_id, content, note_text, author_handle, author_name, tags_text)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                bookmark.id,
                bookmark.content,
                bookmark.note_text,
                bookmark.author_handle,
                bookmark.author_name,
                tags_text,
            ],
        )?;

        Ok(())
    }

    /// Full-text search across bookmarks
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Bookmark>> {
        let query = Self::prepare_fts_query(query);

        let mut stmt = self.conn.prepare(
            r#"SELECT b.id, b.tweet_url, b.content, b.note_text, b.tweeted_at, b.imported_at,
                      b.author_handle, b.author_name, b.author_profile_url, b.author_profile_image,
                      b.comments, b.is_favorite
               FROM bookmarks b
               JOIN bookmarks_fts_content fc ON fc.bookmark_id = b.id
               JOIN bookmarks_fts fts ON fts.rowid = fc.rowid
               WHERE bookmarks_fts MATCH ?1
               ORDER BY bm25(bookmarks_fts), b.tweeted_at DESC
               LIMIT ?2"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![query, limit as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }

    /// Prepare FTS5 query (add prefix matching for better UX)
    fn prepare_fts_query(query: &str) -> String {
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|term| {
                // Escape special FTS5 characters
                let escaped = term.replace('"', "\"\"");
                format!("\"{}\"*", escaped)
            })
            .collect();

        terms.join(" ")
    }

    /// Get all bookmarks with pagination
    pub fn get_bookmarks(&self, offset: usize, limit: usize) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               ORDER BY tweeted_at DESC
               LIMIT ?1 OFFSET ?2"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        self.hydrate_bookmarks(&mut bookmarks)?;

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
               ORDER BY b.tweeted_at DESC
               LIMIT ?2 OFFSET ?3"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![tag, limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

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
               ORDER BY tweeted_at DESC
               LIMIT ?2 OFFSET ?3"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![handle, limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

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

    /// Delete a bookmark
    pub fn delete_bookmark(&self, id: &str) -> Result<bool> {
        let count = self
            .conn
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![id])?;
        if count > 0 {
            self.refresh_stats_snapshot()?;
        }
        Ok(count > 0)
    }

    /// Toggle favorite status for a bookmark
    pub fn toggle_favorite(&self, id: &str) -> Result<bool> {
        self.conn.execute(
            "UPDATE bookmarks SET is_favorite = NOT is_favorite WHERE id = ?1",
            params![id],
        )?;

        // Return the new favorite status
        let is_favorite: bool = self.conn.query_row(
            "SELECT is_favorite FROM bookmarks WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        self.refresh_stats_snapshot()?;
        Ok(is_favorite)
    }

    /// Set favorite status for a bookmark
    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET is_favorite = ?2 WHERE id = ?1",
            params![id, favorite as i32],
        )?;
        self.refresh_stats_snapshot()?;
        Ok(())
    }

    /// Get all favorite bookmarks
    pub fn get_favorites(&self, offset: usize, limit: usize) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               WHERE is_favorite = 1
               ORDER BY tweeted_at DESC
               LIMIT ?1 OFFSET ?2"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

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
               ORDER BY tweeted_at DESC
               LIMIT ?3 OFFSET ?4"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(
                params![from_ts, to_ts, limit as i64, offset as i64],
                |row| self.row_to_bookmark(row),
            )?
            .filter_map(|r| r.ok())
            .collect();

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }

    /// Advanced search with filters
    #[allow(clippy::too_many_arguments)]
    pub fn search_with_filters(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        author: Option<&str>,
        from_date: Option<chrono::DateTime<chrono::Utc>>,
        to_date: Option<chrono::DateTime<chrono::Utc>>,
        favorites_only: bool,
        has_media: Option<bool>,
        limit: usize,
    ) -> Result<Vec<Bookmark>> {
        let mut sql = String::from(
            r#"SELECT DISTINCT b.id, b.tweet_url, b.content, b.note_text, b.tweeted_at, b.imported_at,
                      b.author_handle, b.author_name, b.author_profile_url, b.author_profile_image,
                      b.comments, b.is_favorite
               FROM bookmarks b"#,
        );

        let mut conditions = Vec::new();
        let mut params = Vec::<Value>::new();
        let mut uses_fts = false;
        let mut uses_tags = false;
        let mut uses_media_join = false;

        if let Some(q) = query {
            if !q.trim().is_empty() {
                uses_fts = true;
                conditions.push("bookmarks_fts MATCH ?".to_string());
                params.push(Value::Text(Self::prepare_fts_query(q)));
            }
        }

        if let Some(t) = tag {
            uses_tags = true;
            conditions.push("t.name = ?".to_string());
            params.push(Value::Text(t.to_string()));
        }

        if let Some(a) = author {
            conditions.push("b.author_handle = ?".to_string());
            params.push(Value::Text(a.to_string()));
        }

        if let Some(from) = from_date {
            conditions.push("b.tweeted_at >= ?".to_string());
            params.push(Value::Integer(from.timestamp()));
        }
        if let Some(to) = to_date {
            conditions.push("b.tweeted_at <= ?".to_string());
            params.push(Value::Integer(to.timestamp()));
        }

        if favorites_only {
            conditions.push("b.is_favorite = 1".to_string());
        }

        if let Some(has) = has_media {
            if has {
                uses_media_join = true;
            } else {
                conditions.push(
                    "NOT EXISTS (SELECT 1 FROM media m WHERE m.bookmark_id = b.id)".to_string(),
                );
            }
        }

        if uses_fts {
            sql.push_str(" JOIN bookmarks_fts_content fc ON fc.bookmark_id = b.id");
            sql.push_str(" JOIN bookmarks_fts fts ON fts.rowid = fc.rowid");
        }
        if uses_tags {
            sql.push_str(" JOIN bookmark_tags bt ON bt.bookmark_id = b.id");
            sql.push_str(" JOIN tags t ON t.id = bt.tag_id");
        }
        if uses_media_join {
            sql.push_str(" JOIN media m ON m.bookmark_id = b.id");
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        sql.push_str(" ORDER BY b.tweeted_at DESC");
        sql.push_str(" LIMIT ?");
        params.push(Value::Integer(limit as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params_from_iter(params.iter()), |row| {
                self.row_to_bookmark(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        self.hydrate_bookmarks(&mut bookmarks)?;

        Ok(bookmarks)
    }

    fn hydrate_bookmarks(&self, bookmarks: &mut [Bookmark]) -> Result<()> {
        if bookmarks.is_empty() {
            return Ok(());
        }

        let bookmark_ids = bookmarks
            .iter()
            .map(|bookmark| bookmark.id.clone())
            .collect::<Vec<_>>();
        let tags_by_bookmark = self.load_tags_for_bookmarks(&bookmark_ids)?;
        let media_by_bookmark = self.load_media_for_bookmarks(&bookmark_ids)?;

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
            r#"SELECT bookmark_id, url, media_type
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
            let url: String = row.get(1)?;
            let media_type = match row.get::<_, String>(2)?.as_str() {
                "image" => MediaType::Image,
                "video" => MediaType::Video,
                "gif" => MediaType::Gif,
                _ => MediaType::Unknown,
            };

            media_by_bookmark
                .entry(bookmark_id)
                .or_default()
                .push(Media { url, media_type });
        }

        Ok(media_by_bookmark)
    }

    /// Get all unique tags with counts
    pub fn get_all_tags(&self) -> Result<Vec<(String, i64)>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT t.name, COUNT(bt.bookmark_id) as count
               FROM tags t
               LEFT JOIN bookmark_tags bt ON bt.tag_id = t.id
               GROUP BY t.id
               ORDER BY count DESC"#,
        )?;

        let tags = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(tags)
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<BookmarkStats> {
        if let Some(snapshot) = self.get_metadata(STATS_SNAPSHOT_METADATA_KEY)? {
            if let Ok(stats) = serde_json::from_str::<BookmarkStats>(&snapshot) {
                return Ok(stats);
            }
        }

        let stats = self.compute_stats()?;
        self.persist_stats_snapshot(&stats)?;
        Ok(stats)
    }

    fn compute_stats(&self) -> Result<BookmarkStats> {
        let total_bookmarks: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))?;

        let unique_authors: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT author_handle) FROM bookmarks",
            [],
            |row| row.get(0),
        )?;

        let unique_tags: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;

        let favorite_bookmarks: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )?;

        let earliest_date =
            self.conn
                .query_row("SELECT MIN(tweeted_at) FROM bookmarks", [], |row| {
                    row.get::<_, Option<i64>>(0)
                })?;

        let latest_date =
            self.conn
                .query_row("SELECT MAX(tweeted_at) FROM bookmarks", [], |row| {
                    row.get::<_, Option<i64>>(0)
                })?;

        let top_tags = self.get_all_tags()?;

        use chrono::TimeZone;

        Ok(BookmarkStats {
            total_bookmarks,
            unique_authors,
            unique_tags,
            favorite_bookmarks,
            earliest_date: earliest_date.map(|ts| chrono::Utc.timestamp_opt(ts, 0).unwrap()),
            latest_date: latest_date.map(|ts| chrono::Utc.timestamp_opt(ts, 0).unwrap()),
            top_tags,
        })
    }

    fn persist_stats_snapshot(&self, stats: &BookmarkStats) -> Result<()> {
        let payload = serde_json::to_string(stats)
            .map_err(|error| Error::Other(format!("Failed to serialize stats snapshot: {error}")))?;
        self.set_metadata(STATS_SNAPSHOT_METADATA_KEY, &payload)
    }

    fn refresh_stats_snapshot(&self) -> Result<()> {
        let stats = self.compute_stats()?;
        self.persist_stats_snapshot(&stats)
    }

    /// Convert a database row to a Bookmark
    fn row_to_bookmark(&self, row: &rusqlite::Row) -> rusqlite::Result<Bookmark> {
        use chrono::TimeZone;

        let id: String = row.get(0)?;
        let tweeted_at_ts: i64 = row.get(4)?;
        let imported_at_ts: i64 = row.get(5)?;
        let is_favorite: i32 = row.get(11).unwrap_or(0);

        let bookmark = Bookmark {
            id: id.clone(),
            tweet_url: row.get(1)?,
            content: row.get(2)?,
            note_text: row.get(3)?,
            tweeted_at: chrono::Utc.timestamp_opt(tweeted_at_ts, 0).unwrap(),
            imported_at: chrono::Utc.timestamp_opt(imported_at_ts, 0).unwrap(),
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
            .filter_map(|r| r.ok())
            .collect();

        Ok(tags)
    }

    /// Load media for a bookmark
    pub fn load_bookmark_media(&self, bookmark_id: &str) -> Result<Vec<Media>> {
        let mut stmt = self
            .conn
            .prepare("SELECT url, media_type FROM media WHERE bookmark_id = ?1")?;

        let media = stmt
            .query_map(params![bookmark_id], |row| {
                let url: String = row.get(0)?;
                let media_type_str: String = row.get(1)?;
                let media_type = match media_type_str.as_str() {
                    "image" => MediaType::Image,
                    "video" => MediaType::Video,
                    "gif" => MediaType::Gif,
                    _ => MediaType::Unknown,
                };
                Ok(Media { url, media_type })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(media)
    }

    /// Store lightweight app metadata as JSON/text.
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO app_metadata (key, value)
               VALUES (?1, ?2)
               ON CONFLICT(key) DO UPDATE SET value = excluded.value"#,
            params![key, value],
        )?;
        Ok(())
    }

    /// Read lightweight app metadata.
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>> {
        let result = self.conn.query_row(
            "SELECT value FROM app_metadata WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BookmarkBuilder;
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

        db.insert_bookmarks(&[first.clone(), second.clone()]).unwrap();

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
}
