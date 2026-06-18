//! Idempotent SQLite schema initialization and data migrations.

use super::database::{Database, AUTHOR_STATS_METADATA_KEY};
use super::schema::{PRAGMAS, SCHEMA};
use crate::{Error, Result};
use rusqlite::types::ValueRef;
use tracing::debug;

const CURRENT_SCHEMA_USER_VERSION: i64 = 0;

const MEDIA_METADATA_COLUMNS: &[(&str, &str)] = &[
    ("alt_text", "TEXT"),
    ("width", "INTEGER"),
    ("height", "INTEGER"),
    ("source_media_key", "TEXT"),
    ("source_type", "TEXT"),
    ("preview_url", "TEXT"),
    ("variant_url", "TEXT"),
    ("variants_json", "TEXT"),
];

impl Database {
    pub(crate) fn initialize(&self) -> Result<()> {
        // Set performance pragmas
        self.conn.execute_batch(PRAGMAS)?;

        self.ensure_supported_user_version()?;
        self.ensure_supported_legacy_shape()?;
        self.ensure_legacy_bookmark_columns()?;

        // Create schema
        self.conn.execute_batch(SCHEMA)?;

        self.ensure_is_favorite_column()?;
        self.ensure_media_metadata_columns()?;
        self.ensure_has_media_column()?;
        self.ensure_author_stats_snapshot()?;
        self.conn.execute_batch("PRAGMA optimize;")?;

        debug!("Database initialized");
        Ok(())
    }

    pub fn observed_pragma_settings(&self) -> Result<Vec<(String, String)>> {
        [
            "journal_mode",
            "synchronous",
            "cache_size",
            "temp_store",
            "mmap_size",
            "foreign_keys",
        ]
        .into_iter()
        .map(|name| Ok((name.to_string(), self.pragma_value(name)?)))
        .collect()
    }

    fn ensure_supported_user_version(&self) -> Result<()> {
        let user_version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if user_version > CURRENT_SCHEMA_USER_VERSION {
            return Err(Error::Other(format!(
                "database user_version {user_version} is newer than supported {CURRENT_SCHEMA_USER_VERSION}"
            )));
        }
        Ok(())
    }

    fn ensure_supported_legacy_shape(&self) -> Result<()> {
        if self.table_exists("media")? && !self.table_column_exists("media", "bookmark_id")? {
            return Err(Error::Other(
                "unsupported legacy media table shape: missing bookmark_id".to_string(),
            ));
        }
        Ok(())
    }

    fn ensure_legacy_bookmark_columns(&self) -> Result<()> {
        if !self.table_exists("bookmarks")? {
            return Ok(());
        }

        if !self.bookmarks_column_exists("is_favorite")? {
            self.conn.execute(
                "ALTER TABLE bookmarks ADD COLUMN is_favorite INTEGER DEFAULT 0",
                [],
            )?;
        }

        Ok(())
    }

    fn table_exists(&self, table_name: &str) -> Result<bool> {
        let exists: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get(0),
        )?;
        Ok(exists > 0)
    }

    fn bookmarks_column_exists(&self, column_name: &str) -> Result<bool> {
        self.table_column_exists("bookmarks", column_name)
    }

    fn table_column_exists(&self, table_name: &str, column_name: &str) -> Result<bool> {
        let pragma = format!("PRAGMA table_info({table_name})");
        let mut stmt = self.conn.prepare(&pragma)?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name.eq_ignore_ascii_case(column_name) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn pragma_value(&self, name: &str) -> Result<String> {
        let pragma = format!("PRAGMA {name}");
        self.conn
            .query_row(&pragma, [], |row| {
                let value = row.get_ref(0)?;
                let rendered = match value {
                    ValueRef::Null => "null".to_string(),
                    ValueRef::Integer(value) => value.to_string(),
                    ValueRef::Real(value) => value.to_string(),
                    ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
                    ValueRef::Blob(value) => format!("<{} byte blob>", value.len()),
                };
                Ok(rendered)
            })
            .map_err(Error::from)
    }

    fn ensure_author_stats_snapshot(&self) -> Result<()> {
        if self.get_metadata(AUTHOR_STATS_METADATA_KEY)?.as_deref() == Some("ready") {
            return Ok(());
        }

        self.rebuild_author_stats_snapshot()?;
        self.set_metadata(AUTHOR_STATS_METADATA_KEY, "ready")?;
        Ok(())
    }

    fn rebuild_author_stats_snapshot(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
DELETE FROM author_stats;

INSERT INTO author_stats (
    author_handle,
    author_name,
    author_profile_image,
    bookmark_count,
    favorite_count
)
SELECT author_handle,
       COALESCE(NULLIF(MAX(author_name), ''), author_handle) AS author_name,
       MAX(NULLIF(author_profile_image, '')) AS author_profile_image,
       COUNT(*) AS bookmark_count,
       SUM(CASE WHEN is_favorite = 1 THEN 1 ELSE 0 END) AS favorite_count
FROM bookmarks
GROUP BY author_handle;
"#,
        )?;

        Ok(())
    }

    fn ensure_media_metadata_columns(&self) -> Result<()> {
        if !self.table_exists("media")? {
            return Ok(());
        }

        for (column_name, column_type) in MEDIA_METADATA_COLUMNS {
            if !self.table_column_exists("media", column_name)? {
                let statement = format!("ALTER TABLE media ADD COLUMN {column_name} {column_type}");
                self.conn.execute(&statement, [])?;
            }
        }

        Ok(())
    }

    fn ensure_has_media_column(&self) -> Result<()> {
        let has_column = self.bookmarks_column_exists("has_media")?;

        self.conn.execute("BEGIN IMMEDIATE", [])?;
        let result = (|| -> Result<()> {
            if !has_column {
                self.conn.execute(
                    "ALTER TABLE bookmarks ADD COLUMN has_media INTEGER DEFAULT 0",
                    [],
                )?;
            }
            self.conn.execute(
                "UPDATE bookmarks SET has_media = (SELECT CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END FROM media WHERE bookmark_id = bookmarks.id)",
                [],
            )?;
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", [])?;
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                return Err(e);
            }
        }

        // These DDL statements depend on has_media existing — run after the migration above.
        self.conn.execute_batch(
            r#"
CREATE INDEX IF NOT EXISTS idx_bookmarks_has_media ON bookmarks(has_media) WHERE has_media = 1;

CREATE TRIGGER IF NOT EXISTS media_insert_set_has_media AFTER INSERT ON media BEGIN
    UPDATE bookmarks SET has_media = 1 WHERE id = NEW.bookmark_id;
END;

CREATE TRIGGER IF NOT EXISTS media_delete_update_has_media AFTER DELETE ON media BEGIN
    UPDATE bookmarks SET has_media = (
        SELECT CASE WHEN COUNT(*) > 0 THEN 1 ELSE 0 END FROM media WHERE bookmark_id = OLD.bookmark_id
    ) WHERE id = OLD.bookmark_id;
END;
"#,
        )?;

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
}

#[cfg(test)]
#[path = "migrations_tests.rs"]
mod tests;
