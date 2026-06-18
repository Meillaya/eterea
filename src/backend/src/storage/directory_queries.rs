//! Directory-style aggregate read queries.

use super::database::Database;
use super::queries::AuthorStats;
use crate::Result;

impl Database {
    pub fn get_all_authors(&self) -> Result<Vec<AuthorStats>> {
        let mut stmt = self.conn.prepare_cached(
            r#"SELECT author_handle,
                      author_name,
                      author_profile_image,
                      bookmark_count,
                      favorite_count
               FROM author_stats
               ORDER BY bookmark_count DESC, author_handle ASC"#,
        )?;

        let authors = stmt
            .query_map([], |row| {
                Ok(AuthorStats {
                    handle: row.get(0)?,
                    name: row.get(1)?,
                    profile_image: row.get(2)?,
                    bookmark_count: row.get(3)?,
                    favorite_count: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(authors)
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
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(tags)
    }
}
