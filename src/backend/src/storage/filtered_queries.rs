//! Full-text and filtered bookmark queries.

use super::database::Database;
use crate::models::Bookmark;
use crate::Result;
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter};
use tracing::debug;

impl Database {
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
               ORDER BY bm25(bookmarks_fts), b.tweeted_at DESC, b.id DESC
               LIMIT ?2"#,
        )?;

        let mut bookmarks: Vec<Bookmark> = stmt
            .query_map(params![query, limit as i64], |row| {
                self.row_to_bookmark(row)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

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
        let (bookmarks, _) = self.search_with_filters_page(
            query,
            tag,
            author,
            from_date,
            to_date,
            favorites_only,
            has_media,
            0,
            limit,
        )?;

        Ok(bookmarks)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn search_with_filters_page(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        author: Option<&str>,
        from_date: Option<chrono::DateTime<chrono::Utc>>,
        to_date: Option<chrono::DateTime<chrono::Utc>>,
        favorites_only: bool,
        has_media: Option<bool>,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Bookmark>, i64)> {
        let overall_started = std::time::Instant::now();
        let (where_clause, mut params) = self.build_filtered_where_clause(
            query,
            tag,
            author,
            from_date,
            to_date,
            favorites_only,
            has_media,
        );

        // Single query: data + total count via window function (no second COUNT query)
        let mut sql = String::from(
            r#"SELECT b.id, b.tweet_url, b.content, b.note_text, b.tweeted_at, b.imported_at,
                      b.author_handle, b.author_name, b.author_profile_url, b.author_profile_image,
                      b.comments, b.is_favorite,
                      COUNT(*) OVER() AS total_count
               FROM bookmarks b"#,
        );
        if !where_clause.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&where_clause);
        }
        sql.push_str(" ORDER BY b.tweeted_at DESC, b.id DESC LIMIT ? OFFSET ?");

        params.push(Value::Integer(limit as i64));
        params.push(Value::Integer(offset as i64));

        let mut stmt = self.conn.prepare(&sql)?;
        let query_started = std::time::Instant::now();

        let mut total: i64 = 0;
        let mut bookmarks = Vec::new();
        let mut rows = stmt.query(params_from_iter(params.iter()))?;
        while let Some(row) = rows.next()? {
            // Named column access — safe against projection reordering
            total = row.get::<_, i64>("total_count")?;
            bookmarks.push(self.row_to_bookmark(row)?);
        }
        let query_elapsed = query_started.elapsed();

        let hydrate_started = std::time::Instant::now();
        self.hydrate_bookmarks(&mut bookmarks)?;
        debug!(
            target: "eterea::db",
            offset,
            limit,
            total,
            rows = bookmarks.len(),
            query_ms = query_elapsed.as_millis(),
            hydrate_ms = hydrate_started.elapsed().as_millis(),
            total_ms = overall_started.elapsed().as_millis(),
            "loaded filtered bookmark page"
        );

        Ok((bookmarks, total))
    }

    /// Build a WHERE clause using subqueries — no outer JOINs means no DISTINCT needed.
    /// FTS and tag filters use IN-subqueries; has_media uses the denormalized column.
    #[allow(clippy::too_many_arguments)]
    fn build_filtered_where_clause(
        &self,
        query: Option<&str>,
        tag: Option<&str>,
        author: Option<&str>,
        from_date: Option<chrono::DateTime<chrono::Utc>>,
        to_date: Option<chrono::DateTime<chrono::Utc>>,
        favorites_only: bool,
        has_media: Option<bool>,
    ) -> (String, Vec<Value>) {
        let mut conditions = Vec::<String>::new();
        let mut params = Vec::<Value>::new();

        if let Some(q) = query {
            if !q.trim().is_empty() {
                // FTS via subquery: query the virtual table first (its optimized MATCH path),
                // then look up bookmark_id via the rowid link to our content table.
                // No outer JOIN → no row multiplication, no DISTINCT needed.
                conditions.push(
                    "b.id IN (SELECT fc.bookmark_id FROM bookmarks_fts fts \
                     JOIN bookmarks_fts_content fc ON fc.rowid = fts.rowid \
                     WHERE bookmarks_fts MATCH ?)"
                        .to_string(),
                );
                params.push(Value::Text(Self::prepare_fts_query(q)));
            }
        }

        if let Some(t) = tag {
            // Tag via subquery: no outer JOIN, no row multiplication
            conditions.push(
                "b.id IN (SELECT bt.bookmark_id FROM bookmark_tags bt \
                 JOIN tags t ON t.id = bt.tag_id WHERE t.name = ?)"
                    .to_string(),
            );
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
            // Use denormalized column — no JOIN needed
            conditions.push(format!("b.has_media = {}", if has { 1 } else { 0 }));
        }

        (conditions.join(" AND "), params)
    }
}
