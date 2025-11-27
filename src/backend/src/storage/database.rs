//! SQLite database implementation

use super::queries::BookmarkStats;
use super::schema::{PRAGMAS, SCHEMA};
use crate::models::{Bookmark, Media, MediaType};
use crate::{Error, Result};
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use tracing::{debug, info};

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
                    if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                    // Skip duplicates (same tweet_url)
                    debug!("Skipping duplicate bookmark: {}", bookmark.tweet_url);
                }
                Err(e) => {
                    conn.execute("ROLLBACK", [])?;
                    return Err(e);
                }
            }
        }
        
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
        
        let mut bookmarks: Vec<Bookmark> = stmt.query_map(params![query, limit as i64], |row| {
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
        
        let mut bookmarks: Vec<Bookmark> = stmt.query_map(params![limit as i64, offset as i64], |row| {
            self.row_to_bookmark(row)
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        self.hydrate_bookmarks(&mut bookmarks)?;
        
        Ok(bookmarks)
    }
    
    /// Get bookmarks by tag
    pub fn get_bookmarks_by_tag(&self, tag: &str, offset: usize, limit: usize) -> Result<Vec<Bookmark>> {
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
        
        let mut bookmarks: Vec<Bookmark> = stmt.query_map(params![tag, limit as i64, offset as i64], |row| {
            self.row_to_bookmark(row)
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        self.hydrate_bookmarks(&mut bookmarks)?;
        
        Ok(bookmarks)
    }
    
    /// Get bookmarks by author
    pub fn get_bookmarks_by_author(&self, handle: &str, offset: usize, limit: usize) -> Result<Vec<Bookmark>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, tweet_url, content, note_text, tweeted_at, imported_at,
                      author_handle, author_name, author_profile_url, author_profile_image,
                      comments, is_favorite
               FROM bookmarks
               WHERE author_handle = ?1
               ORDER BY tweeted_at DESC
               LIMIT ?2 OFFSET ?3"#,
        )?;
        
        let mut bookmarks: Vec<Bookmark> = stmt.query_map(params![handle, limit as i64, offset as i64], |row| {
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
            },
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    /// Delete a bookmark
    pub fn delete_bookmark(&self, id: &str) -> Result<bool> {
        let count = self.conn.execute(
            "DELETE FROM bookmarks WHERE id = ?1",
            params![id],
        )?;
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
        
        Ok(is_favorite)
    }
    
    /// Set favorite status for a bookmark
    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE bookmarks SET is_favorite = ?2 WHERE id = ?1",
            params![id, favorite as i32],
        )?;
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
        
        let mut bookmarks: Vec<Bookmark> = stmt.query_map(params![limit as i64, offset as i64], |row| {
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
        
        let mut bookmarks: Vec<Bookmark> = stmt.query_map(
            params![from_ts, to_ts, limit as i64, offset as i64],
            |row| self.row_to_bookmark(row),
        )?
        .filter_map(|r| r.ok())
        .collect();
        
        self.hydrate_bookmarks(&mut bookmarks)?;
        
        Ok(bookmarks)
    }
    
    /// Advanced search with filters
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
        // Build dynamic query
        let mut sql = String::from(
            r#"SELECT DISTINCT b.id, b.tweet_url, b.content, b.note_text, b.tweeted_at, b.imported_at,
                      b.author_handle, b.author_name, b.author_profile_url, b.author_profile_image,
                      b.comments, b.is_favorite
               FROM bookmarks b"#
        );
        
        let mut conditions = Vec::new();
        let mut joins = Vec::new();
        
        // FTS search
        if let Some(q) = query {
            if !q.trim().is_empty() {
                joins.push("JOIN bookmarks_fts_content fc ON fc.bookmark_id = b.id");
                joins.push("JOIN bookmarks_fts fts ON fts.rowid = fc.rowid");
                let fts_query = Self::prepare_fts_query(q);
                conditions.push(format!("bookmarks_fts MATCH '{}'", fts_query.replace("'", "''")));
            }
        }
        
        // Tag filter
        if let Some(t) = tag {
            joins.push("JOIN bookmark_tags bt ON bt.bookmark_id = b.id");
            joins.push("JOIN tags t ON t.id = bt.tag_id");
            conditions.push(format!("t.name = '{}'", t.replace("'", "''")));
        }
        
        // Author filter
        if let Some(a) = author {
            conditions.push(format!("b.author_handle = '{}'", a.replace("'", "''")));
        }
        
        // Date range
        if let Some(from) = from_date {
            conditions.push(format!("b.tweeted_at >= {}", from.timestamp()));
        }
        if let Some(to) = to_date {
            conditions.push(format!("b.tweeted_at <= {}", to.timestamp()));
        }
        
        // Favorites only
        if favorites_only {
            conditions.push("b.is_favorite = 1".to_string());
        }
        
        // Has media filter
        if let Some(has) = has_media {
            if has {
                joins.push("JOIN media m ON m.bookmark_id = b.id");
            } else {
                conditions.push("NOT EXISTS (SELECT 1 FROM media m WHERE m.bookmark_id = b.id)".to_string());
            }
        }
        
        // Build final query
        for join in joins {
            sql.push(' ');
            sql.push_str(join);
        }
        
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        
        sql.push_str(" ORDER BY b.tweeted_at DESC");
        sql.push_str(&format!(" LIMIT {}", limit));
        
        let mut stmt = self.conn.prepare(&sql)?;
        let mut bookmarks: Vec<Bookmark> = stmt.query_map([], |row| self.row_to_bookmark(row))?
            .filter_map(|r| r.ok())
            .collect();
        
        self.hydrate_bookmarks(&mut bookmarks)?;
        
        Ok(bookmarks)
    }

    fn hydrate_bookmarks(&self, bookmarks: &mut [Bookmark]) -> Result<()> {
        for bookmark in bookmarks {
            bookmark.tags = self.load_bookmark_tags(&bookmark.id)?;
            bookmark.media = self.load_bookmark_media(&bookmark.id)?;
        }
        Ok(())
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
        
        let tags = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(tags)
    }
    
    /// Get database statistics
    pub fn get_stats(&self) -> Result<BookmarkStats> {
        let total_bookmarks: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM bookmarks",
            [],
            |row| row.get(0),
        )?;
        
        let unique_authors: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT author_handle) FROM bookmarks",
            [],
            |row| row.get(0),
        )?;
        
        let unique_tags: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tags",
            [],
            |row| row.get(0),
        )?;

        let favorite_bookmarks: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM bookmarks WHERE is_favorite = 1",
            [],
            |row| row.get(0),
        )?;
        
        let earliest_date = self.conn.query_row(
            "SELECT MIN(tweeted_at) FROM bookmarks",
            [],
            |row| row.get::<_, Option<i64>>(0),
        )?;
        
        let latest_date = self.conn.query_row(
            "SELECT MAX(tweeted_at) FROM bookmarks",
            [],
            |row| row.get::<_, Option<i64>>(0),
        )?;
        
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
        
        let tags = stmt.query_map(params![bookmark_id], |row| {
            row.get::<_, String>(0)
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(tags)
    }
    
    /// Load media for a bookmark
    pub fn load_bookmark_media(&self, bookmark_id: &str) -> Result<Vec<Media>> {
        let mut stmt = self.conn.prepare(
            "SELECT url, media_type FROM media WHERE bookmark_id = ?1",
        )?;
        
        let media = stmt.query_map(params![bookmark_id], |row| {
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
}

