use super::*;
use rusqlite::{params, Connection};
use std::path::Path;
use tempfile::TempDir;

#[test]
fn initializes_empty_database_with_current_schema() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("empty.db");
    let db = Database::open(&db_path)?;

    assert_table_exists(&db.conn, "bookmarks")?;
    assert_table_exists(&db.conn, "media")?;
    assert_column_exists(&db.conn, "is_favorite")?;
    assert_column_exists(&db.conn, "has_media")?;
    assert_eq!(
        db.get_metadata(AUTHOR_STATS_METADATA_KEY)?.as_deref(),
        Some("ready")
    );
    Ok(())
}

#[test]
fn reopens_existing_current_schema_idempotently() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("current.db");
    let first = Database::open(&db_path)?;
    first.conn.execute(
        r#"INSERT INTO bookmarks
           (id, tweet_url, content, tweeted_at, imported_at, author_handle, author_name)
           VALUES ('current-1', 'https://x.com/current/status/1', 'current row',
                   1717243200, 1717243200, 'current', 'Current Author')"#,
        [],
    )?;
    drop(first);

    let reopened = Database::open(&db_path)?;

    let row_count: i64 = reopened
        .conn
        .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))?;
    assert_eq!(row_count, 1);
    assert_column_exists(&reopened.conn, "is_favorite")?;
    assert_column_exists(&reopened.conn, "has_media")?;
    Ok(())
}

#[test]
fn migrates_legacy_bookmarks_shape_idempotently() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("legacy.db");
    create_legacy_database(&db_path)?;

    let db = Database::open(&db_path)?;

    assert_column_exists(&db.conn, "is_favorite")?;
    assert_column_exists(&db.conn, "has_media")?;
    let has_media: i64 = db.conn.query_row(
        "SELECT has_media FROM bookmarks WHERE id = 'legacy-1'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(has_media, 1);
    let author_stats: (i64, i64) = db.conn.query_row(
        "SELECT bookmark_count, favorite_count FROM author_stats WHERE author_handle = 'old_author'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    assert_eq!(author_stats, (1, 0));
    assert_eq!(
        db.get_metadata(AUTHOR_STATS_METADATA_KEY)?.as_deref(),
        Some("ready")
    );
    drop(db);

    let reopened = Database::open(&db_path)?;
    let row_count: i64 = reopened
        .conn
        .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))?;
    let author_count: i64 =
        reopened
            .conn
            .query_row("SELECT COUNT(*) FROM author_stats", [], |row| row.get(0))?;
    assert_eq!(row_count, 1);
    assert_eq!(author_count, 1);
    Ok(())
}

#[test]
fn rejects_future_user_version_without_schema_mutation() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("future-version.db");
    create_future_version_database(&db_path)?;

    let result = Database::open(&db_path);

    assert!(result.is_err());
    let conn = Connection::open(&db_path)?;
    assert_column_absent(&conn, "is_favorite")?;
    assert_column_absent(&conn, "has_media")?;
    Ok(())
}

#[test]
fn rejects_incompatible_media_shape_without_has_media_partial_state() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("bad-media-shape.db");
    create_bad_media_shape_database(&db_path)?;

    let result = Database::open(&db_path);

    assert!(result.is_err());
    let conn = Connection::open(&db_path)?;
    assert_column_absent(&conn, "has_media")?;
    Ok(())
}

#[test]
fn repairs_stale_has_media_values_on_retry() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("stale-has-media.db");
    create_stale_has_media_database(&db_path)?;

    let db = Database::open(&db_path)?;

    let has_media: i64 = db.conn.query_row(
        "SELECT has_media FROM bookmarks WHERE id = 'legacy-1'",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(has_media, 1);
    Ok(())
}

#[test]
fn file_backed_database_uses_wal_journal_mode() -> crate::Result<()> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("wal-proof.db");
    let db = Database::open(&db_path)?;
    let journal_mode: String = db
        .conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    assert_eq!(journal_mode, "wal");
    assert!(std::path::PathBuf::from(format!("{}-wal", db_path.display())).exists());
    Ok(())
}

fn create_legacy_database(path: &Path) -> crate::Result<()> {
    let conn = Connection::open(path)?;
    create_legacy_schema(&conn)?;
    insert_legacy_bookmark(&conn)?;
    conn.execute(
        "INSERT INTO media (bookmark_id, url, media_type) VALUES (?1, ?2, 'image')",
        params!["legacy-1", "https://pbs.twimg.com/media/legacy.jpg"],
    )?;
    Ok(())
}

fn create_future_version_database(path: &Path) -> crate::Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA user_version = 1;")?;
    create_legacy_schema(&conn)?;
    insert_legacy_bookmark(&conn)
}

fn create_bad_media_shape_database(path: &Path) -> crate::Result<()> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE bookmarks (id TEXT PRIMARY KEY, tweet_url TEXT UNIQUE NOT NULL, content TEXT NOT NULL, tweeted_at INTEGER NOT NULL, imported_at INTEGER NOT NULL, author_handle TEXT NOT NULL, author_name TEXT NOT NULL);
         CREATE TABLE media (id INTEGER PRIMARY KEY AUTOINCREMENT, url TEXT NOT NULL, media_type TEXT NOT NULL);",
    )?;
    conn.execute(
        r#"INSERT INTO bookmarks
           (id, tweet_url, content, tweeted_at, imported_at, author_handle, author_name)
           VALUES ('legacy-1', 'https://x.com/old_author/status/1', 'legacy row',
                   1717243200, 1717243200, 'old_author', 'Old Author')"#,
        [],
    )?;
    Ok(())
}

fn create_stale_has_media_database(path: &Path) -> crate::Result<()> {
    let conn = Connection::open(path)?;
    create_legacy_schema(&conn)?;
    conn.execute(
        "ALTER TABLE bookmarks ADD COLUMN has_media INTEGER DEFAULT 0",
        [],
    )?;
    insert_legacy_bookmark(&conn)?;
    conn.execute(
        "INSERT INTO media (bookmark_id, url, media_type) VALUES (?1, ?2, 'image')",
        params!["legacy-1", "https://pbs.twimg.com/media/legacy.jpg"],
    )?;
    Ok(())
}

fn create_legacy_schema(conn: &Connection) -> crate::Result<()> {
    conn.execute_batch(
        "CREATE TABLE bookmarks (id TEXT PRIMARY KEY, tweet_url TEXT UNIQUE NOT NULL, content TEXT NOT NULL, note_text TEXT, tweeted_at INTEGER NOT NULL, imported_at INTEGER NOT NULL, author_handle TEXT NOT NULL, author_name TEXT NOT NULL, author_profile_url TEXT, author_profile_image TEXT, comments TEXT);
         CREATE TABLE media (id INTEGER PRIMARY KEY AUTOINCREMENT, bookmark_id TEXT NOT NULL, url TEXT NOT NULL, media_type TEXT NOT NULL);",
    )
    .map_err(crate::Error::from)
}

fn insert_legacy_bookmark(conn: &Connection) -> crate::Result<()> {
    conn.execute(
        r#"INSERT INTO bookmarks
           (id, tweet_url, content, note_text, tweeted_at, imported_at,
            author_handle, author_name, author_profile_url, author_profile_image, comments)
           VALUES ('legacy-1', 'https://x.com/old_author/status/1', 'legacy row',
                   NULL, 1717243200, 1717243200, 'old_author', 'Old Author',
                   NULL, NULL, NULL)"#,
        [],
    )?;
    Ok(())
}

fn assert_column_exists(conn: &Connection, column_name: &str) -> crate::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(bookmarks)")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let current_name: String = row.get(1)?;
        if current_name.eq_ignore_ascii_case(column_name) {
            return Ok(());
        }
    }
    Err(crate::Error::Other(format!(
        "missing migrated column {column_name}"
    )))
}

fn assert_column_absent(conn: &Connection, column_name: &str) -> crate::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(bookmarks)")?;
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let current_name: String = row.get(1)?;
        if current_name.eq_ignore_ascii_case(column_name) {
            return Err(crate::Error::Other(format!(
                "unexpected migrated column {column_name}"
            )));
        }
    }
    Ok(())
}

fn assert_table_exists(conn: &Connection, table_name: &str) -> crate::Result<()> {
    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
        [table_name],
        |row| row.get(0),
    )?;
    assert_eq!(exists, 1);
    Ok(())
}
