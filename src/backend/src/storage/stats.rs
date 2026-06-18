//! Cached statistics snapshots.

use super::database::{Database, STATS_SNAPSHOT_METADATA_KEY};
use super::queries::BookmarkStats;
use crate::{Error, Result};
use chrono::{DateTime, TimeZone, Utc};
use tracing::debug;

impl Database {
    pub fn get_stats(&self) -> Result<BookmarkStats> {
        let overall_started = std::time::Instant::now();
        if let Some(snapshot) = self.get_metadata(STATS_SNAPSHOT_METADATA_KEY)? {
            if let Ok(stats) = serde_json::from_str::<BookmarkStats>(&snapshot) {
                debug!(
                    target: "eterea::db",
                    total_ms = overall_started.elapsed().as_millis(),
                    "loaded stats snapshot"
                );
                return Ok(stats);
            }
        }

        let compute_started = std::time::Instant::now();
        let stats = self.compute_stats()?;
        self.persist_stats_snapshot(&stats)?;
        debug!(
            target: "eterea::db",
            compute_ms = compute_started.elapsed().as_millis(),
            total_ms = overall_started.elapsed().as_millis(),
            "recomputed stats snapshot"
        );
        Ok(stats)
    }

    fn compute_stats(&self) -> Result<BookmarkStats> {
        let total_bookmarks: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))?;

        let unique_authors: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM author_stats", [], |row| row.get(0))?;

        let unique_tags: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;

        let favorite_bookmarks: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(favorite_count), 0) FROM author_stats",
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

        Ok(BookmarkStats {
            total_bookmarks,
            unique_authors,
            unique_tags,
            favorite_bookmarks,
            earliest_date: earliest_date
                .map(|timestamp| utc_timestamp_from_db(timestamp, "earliest bookmark"))
                .transpose()?,
            latest_date: latest_date
                .map(|timestamp| utc_timestamp_from_db(timestamp, "latest bookmark"))
                .transpose()?,
            top_tags,
        })
    }

    fn persist_stats_snapshot(&self, stats: &BookmarkStats) -> Result<()> {
        let payload = serde_json::to_string(stats).map_err(|error| {
            Error::Other(format!("Failed to serialize stats snapshot: {error}"))
        })?;
        self.set_metadata(STATS_SNAPSHOT_METADATA_KEY, &payload)
    }

    pub(crate) fn refresh_stats_snapshot(&self) -> Result<()> {
        let stats = self.compute_stats()?;
        self.persist_stats_snapshot(&stats)
    }
}

fn utc_timestamp_from_db(timestamp: i64, label: &str) -> Result<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single().ok_or_else(|| {
        Error::Other(format!(
            "Invalid {label} timestamp stored in bookmarks: {timestamp}"
        ))
    })
}
