//! Database schema definitions

pub const SCHEMA: &str = r#"
-- Main bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
    id TEXT PRIMARY KEY,
    tweet_url TEXT UNIQUE NOT NULL,
    content TEXT NOT NULL,
    note_text TEXT,
    tweeted_at INTEGER NOT NULL,  -- Unix timestamp for fast sorting
    imported_at INTEGER NOT NULL,
    author_handle TEXT NOT NULL,
    author_name TEXT NOT NULL,
    author_profile_url TEXT,
    author_profile_image TEXT,
    comments TEXT,
    is_favorite INTEGER DEFAULT 0,  -- Boolean as integer (0/1)
    has_media INTEGER DEFAULT 0     -- Denormalized: 1 if any media rows exist (avoids JOIN on filter)
);

-- Materialized author directory. The Authors screen must not GROUP BY the
-- full bookmarks table on every navigation; writes keep this table hot.
CREATE TABLE IF NOT EXISTS author_stats (
    author_handle TEXT PRIMARY KEY,
    author_name TEXT NOT NULL,
    author_profile_image TEXT,
    bookmark_count INTEGER NOT NULL DEFAULT 0,
    favorite_count INTEGER NOT NULL DEFAULT 0
);

-- Tags table (normalized for efficient filtering)
CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL COLLATE NOCASE
);

-- Bookmark-Tag junction table
CREATE TABLE IF NOT EXISTS bookmark_tags (
    bookmark_id TEXT NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (bookmark_id, tag_id),
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Media table
CREATE TABLE IF NOT EXISTS media (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bookmark_id TEXT NOT NULL,
    url TEXT NOT NULL,
    media_type TEXT NOT NULL,
    alt_text TEXT,
    width INTEGER,
    height INTEGER,
    source_media_key TEXT,
    source_type TEXT,
    preview_url TEXT,
    variant_url TEXT,
    variants_json TEXT,
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE
);

-- App metadata (sync status, feature state, lightweight preferences)
CREATE TABLE IF NOT EXISTS app_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
    content,
    note_text,
    author_handle,
    author_name,
    tags_text,
    content='bookmarks_fts_content',
    content_rowid='rowid',
    tokenize='porter unicode61'
);

-- Content table for FTS5
CREATE TABLE IF NOT EXISTS bookmarks_fts_content (
    rowid INTEGER PRIMARY KEY,
    bookmark_id TEXT NOT NULL,
    content TEXT,
    note_text TEXT,
    author_handle TEXT,
    author_name TEXT,
    tags_text TEXT,
    FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_bookmarks_tweeted_at ON bookmarks(tweeted_at DESC);
CREATE INDEX IF NOT EXISTS idx_bookmarks_tweeted_at_id ON bookmarks(tweeted_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_bookmarks_author_handle ON bookmarks(author_handle);
CREATE INDEX IF NOT EXISTS idx_author_stats_count_handle ON author_stats(bookmark_count DESC, author_handle ASC);
CREATE INDEX IF NOT EXISTS idx_bookmarks_imported_at ON bookmarks(imported_at DESC);
CREATE INDEX IF NOT EXISTS idx_bookmarks_favorite ON bookmarks(is_favorite) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_bookmarks_fav_date ON bookmarks(is_favorite, tweeted_at DESC) WHERE is_favorite = 1;
CREATE INDEX IF NOT EXISTS idx_bookmark_tags_bookmark ON bookmark_tags(bookmark_id);
CREATE INDEX IF NOT EXISTS idx_bookmark_tags_tag ON bookmark_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_media_bookmark ON media(bookmark_id);
CREATE INDEX IF NOT EXISTS idx_fts_content_bookmark ON bookmarks_fts_content(bookmark_id);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS bookmarks_fts_insert AFTER INSERT ON bookmarks_fts_content BEGIN
    INSERT INTO bookmarks_fts(rowid, content, note_text, author_handle, author_name, tags_text)
    VALUES (NEW.rowid, NEW.content, NEW.note_text, NEW.author_handle, NEW.author_name, NEW.tags_text);
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_delete AFTER DELETE ON bookmarks_fts_content BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, content, note_text, author_handle, author_name, tags_text)
    VALUES ('delete', OLD.rowid, OLD.content, OLD.note_text, OLD.author_handle, OLD.author_name, OLD.tags_text);
END;

CREATE TRIGGER IF NOT EXISTS bookmarks_fts_update AFTER UPDATE ON bookmarks_fts_content BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, content, note_text, author_handle, author_name, tags_text)
    VALUES ('delete', OLD.rowid, OLD.content, OLD.note_text, OLD.author_handle, OLD.author_name, OLD.tags_text);
    INSERT INTO bookmarks_fts(rowid, content, note_text, author_handle, author_name, tags_text)
    VALUES (NEW.rowid, NEW.content, NEW.note_text, NEW.author_handle, NEW.author_name, NEW.tags_text);
END;

-- Keep author directory summaries current without forcing read-time GROUP BYs.
CREATE TRIGGER IF NOT EXISTS author_stats_bookmark_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO author_stats (
        author_handle,
        author_name,
        author_profile_image,
        bookmark_count,
        favorite_count
    )
    VALUES (
        NEW.author_handle,
        COALESCE(NULLIF(NEW.author_name, ''), NEW.author_handle),
        NULLIF(NEW.author_profile_image, ''),
        1,
        CASE WHEN NEW.is_favorite = 1 THEN 1 ELSE 0 END
    )
    ON CONFLICT(author_handle) DO UPDATE SET
        author_name = CASE
            WHEN NEW.author_name IS NOT NULL AND NEW.author_name <> '' THEN NEW.author_name
            ELSE author_stats.author_name
        END,
        author_profile_image = COALESCE(NULLIF(NEW.author_profile_image, ''), author_stats.author_profile_image),
        bookmark_count = author_stats.bookmark_count + 1,
        favorite_count = author_stats.favorite_count + CASE WHEN NEW.is_favorite = 1 THEN 1 ELSE 0 END;
END;

CREATE TRIGGER IF NOT EXISTS author_stats_bookmark_delete AFTER DELETE ON bookmarks BEGIN
    UPDATE author_stats
    SET bookmark_count = bookmark_count - 1,
        favorite_count = favorite_count - CASE WHEN OLD.is_favorite = 1 THEN 1 ELSE 0 END
    WHERE author_handle = OLD.author_handle;

    DELETE FROM author_stats
    WHERE author_handle = OLD.author_handle
      AND bookmark_count <= 0;
END;

CREATE TRIGGER IF NOT EXISTS author_stats_bookmark_favorite_update
AFTER UPDATE OF is_favorite ON bookmarks
WHEN OLD.author_handle = NEW.author_handle AND OLD.is_favorite IS NOT NEW.is_favorite
BEGIN
    UPDATE author_stats
    SET favorite_count = favorite_count
        + CASE WHEN NEW.is_favorite = 1 THEN 1 ELSE 0 END
        - CASE WHEN OLD.is_favorite = 1 THEN 1 ELSE 0 END
    WHERE author_handle = NEW.author_handle;
END;

CREATE TRIGGER IF NOT EXISTS author_stats_bookmark_author_update
AFTER UPDATE OF author_handle ON bookmarks
WHEN OLD.author_handle IS NOT NEW.author_handle
BEGIN
    UPDATE author_stats
    SET bookmark_count = bookmark_count - 1,
        favorite_count = favorite_count - CASE WHEN OLD.is_favorite = 1 THEN 1 ELSE 0 END
    WHERE author_handle = OLD.author_handle;

    DELETE FROM author_stats
    WHERE author_handle = OLD.author_handle
      AND bookmark_count <= 0;

    INSERT INTO author_stats (
        author_handle,
        author_name,
        author_profile_image,
        bookmark_count,
        favorite_count
    )
    VALUES (
        NEW.author_handle,
        COALESCE(NULLIF(NEW.author_name, ''), NEW.author_handle),
        NULLIF(NEW.author_profile_image, ''),
        1,
        CASE WHEN NEW.is_favorite = 1 THEN 1 ELSE 0 END
    )
    ON CONFLICT(author_handle) DO UPDATE SET
        author_name = CASE
            WHEN NEW.author_name IS NOT NULL AND NEW.author_name <> '' THEN NEW.author_name
            ELSE author_stats.author_name
        END,
        author_profile_image = COALESCE(NULLIF(NEW.author_profile_image, ''), author_stats.author_profile_image),
        bookmark_count = author_stats.bookmark_count + 1,
        favorite_count = author_stats.favorite_count + CASE WHEN NEW.is_favorite = 1 THEN 1 ELSE 0 END;
END;

CREATE TRIGGER IF NOT EXISTS author_stats_bookmark_profile_update
AFTER UPDATE OF author_name, author_profile_image ON bookmarks
WHEN OLD.author_handle = NEW.author_handle
BEGIN
    UPDATE author_stats
    SET author_name = CASE
            WHEN NEW.author_name IS NOT NULL AND NEW.author_name <> '' THEN NEW.author_name
            ELSE author_name
        END,
        author_profile_image = COALESCE(NULLIF(NEW.author_profile_image, ''), author_profile_image)
    WHERE author_handle = NEW.author_handle;
END;

"#;

pub const PRAGMAS: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;
PRAGMA foreign_keys = ON;
"#;
