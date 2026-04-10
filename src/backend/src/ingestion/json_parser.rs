//! JSON parsing for Twitter bookmark exports

use crate::models::{Bookmark, BookmarkBuilder};
use crate::{Error, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tracing::{debug, warn};

/// Parser for JSON bookmark exports
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, path: &Path) -> Result<Vec<Bookmark>> {
        let file = File::open(path)?;
        match serde_json::from_reader::<_, Vec<FlatJsonBookmark>>(BufReader::new(file)) {
            Ok(bookmarks) => match Self::try_map_flat_bookmarks(bookmarks) {
                Some(bookmarks) => {
                    debug!("Parsed {} bookmarks from JSON fast path", bookmarks.len());
                    Ok(bookmarks)
                }
                None => {
                    let raw = std::fs::read_to_string(path)?;
                    self.parse_str(&raw)
                }
            },
            Err(_) => {
                let raw = std::fs::read_to_string(path)?;
                self.parse_str(&raw)
            }
        }
    }

    pub fn parse_str(&self, raw: &str) -> Result<Vec<Bookmark>> {
        let payload = self.extract_payload(raw)?;

        if let Ok(bookmarks) = serde_json::from_str::<Vec<FlatJsonBookmark>>(payload) {
            if let Some(bookmarks) = Self::try_map_flat_bookmarks(bookmarks) {
                debug!("Parsed {} bookmarks from JSON fast path", bookmarks.len());
                return Ok(bookmarks);
            }
        }

        let root: Value = serde_json::from_str(payload)?;
        let raw_bookmarks = self.collect_entries(&root);

        let mut bookmarks = Vec::new();
        for (idx, raw) in raw_bookmarks.into_iter().enumerate() {
            match self.convert_raw(raw) {
                Ok(bookmark) => bookmarks.push(bookmark),
                Err(e) => warn!("Skipping JSON entry {}: {}", idx, e),
            }
        }

        debug!("Parsed {} bookmarks from JSON fallback path", bookmarks.len());
        Ok(bookmarks)
    }

    fn from_flat_bookmark(raw: FlatJsonBookmark) -> Result<Bookmark> {
        let tweet_url = raw
            .tweet_url
            .or_else(|| {
                raw.screen_name
                    .as_ref()
                    .zip(raw.id_str.as_ref())
                    .map(|(screen_name, id)| format!("https://x.com/{screen_name}/status/{id}"))
            })
            .ok_or_else(|| Error::Other("Missing tweet URL".into()))?;
        let author_handle = raw.screen_name.unwrap_or_default();
        let author_name = raw.name.unwrap_or_else(|| author_handle.clone());
        let tweeted_at = parse_date_candidates([
            raw.tweeted_at.as_deref(),
            raw.bookmark_date.as_deref(),
            raw.created_at.as_deref(),
        ])?;

        let mut builder = BookmarkBuilder::new()
            .tweet_url(tweet_url)
            .content(raw.full_text.unwrap_or_default())
            .note_text(raw.note_tweet_text.unwrap_or_default())
            .tweeted_at(tweeted_at)
            .author_handle(author_handle)
            .author_name(author_name)
            .author_profile_image(raw.profile_image_url_https.unwrap_or_default());

        if let Some(media) = raw.extended_media.or(raw.media) {
            for item in media {
                if let Some(url) = item.media_url_https.or(item.url) {
                    builder = builder.add_media(url);
                }
            }
        }

        let mut bookmark = builder.build().map_err(|e| Error::Other(e.to_string()))?;
        if bookmark.tags.is_empty() {
            for tag in bookmark.extract_hashtags() {
                if !bookmark.tags.contains(&tag) {
                    bookmark.tags.push(tag);
                }
            }
            bookmark.compute_search_text();
        }

        Ok(bookmark)
    }

    fn try_map_flat_bookmarks(bookmarks: Vec<FlatJsonBookmark>) -> Option<Vec<Bookmark>> {
        let mut mapped = Vec::with_capacity(bookmarks.len());
        for bookmark in bookmarks {
            match Self::from_flat_bookmark(bookmark) {
                Ok(bookmark) => mapped.push(bookmark),
                Err(_) => return None,
            }
        }
        Some(mapped)
    }

    fn extract_payload<'a>(&self, raw: &'a str) -> Result<&'a str> {
        let trimmed = raw.trim_start_matches('\u{feff}').trim();

        if trimmed.starts_with("window.YTD") {
            let start = trimmed.find(['[', '{']).ok_or_else(|| {
                Error::InvalidFormat("Could not locate JSON payload in archive JS".into())
            })?;
            let end = trimmed.rfind([']', '}']).ok_or_else(|| {
                Error::InvalidFormat("Could not locate end of JSON payload in archive JS".into())
            })?;

            return Ok(trimmed[start..=end].trim());
        }

        Ok(trimmed)
    }

    fn collect_entries<'a>(&self, root: &'a Value) -> Vec<&'a Value> {
        match root {
            Value::Array(items) => items.iter().collect(),
            Value::Object(map) => {
                for key in ["bookmarks", "items", "data"] {
                    if let Some(Value::Array(items)) = map.get(key) {
                        return items.iter().collect();
                    }
                }
                vec![root]
            }
            _ => Vec::new(),
        }
    }

    fn convert_raw(&self, raw: &Value) -> Result<Bookmark> {
        let raw = self.unwrap_entry(raw);

        let author_handle = self
            .extract_string(
                raw,
                &[
                    &["screen_name"],
                    &["author_handle"],
                    &["username"],
                    &["user", "screen_name"],
                    &["core", "user_results", "result", "legacy", "screen_name"],
                ],
            )
            .or_else(|| {
                self.extract_string(
                    raw,
                    &[&["tweet_url"], &["url"], &["expanded_url"], &["expandedUrl"]],
                )
                .and_then(|url| self.extract_handle_from_url(&url))
            })
            .unwrap_or_default();

        let author_name = self
            .extract_string(
                raw,
                &[
                    &["name"],
                    &["author_name"],
                    &["display_name"],
                    &["user", "name"],
                    &["core", "user_results", "result", "legacy", "name"],
                ],
            )
            .unwrap_or_else(|| author_handle.clone());

        let tweet_id =
            self.extract_string(raw, &[&["tweet_id"], &["tweetId"], &["id_str"], &["id"]]);
        let tweet_url = self
            .extract_string(
                raw,
                &[
                    &["tweet_url"],
                    &["url"],
                    &["expanded_url"],
                    &["expandedUrl"],
                    &["tweet", "url"],
                    &["bookmark", "url"],
                ],
            )
            .or_else(|| {
                tweet_id.as_ref().map(|id| {
                    if author_handle.is_empty() {
                        format!("https://x.com/i/web/status/{id}")
                    } else {
                        format!("https://x.com/{author_handle}/status/{id}")
                    }
                })
            })
            .ok_or_else(|| Error::Other("Missing tweet URL".into()))?;

        let content = self
            .extract_string(
                raw,
                &[
                    &["full_text"],
                    &["text"],
                    &["content"],
                    &["legacy", "full_text"],
                    &["tweet", "full_text"],
                ],
            )
            .unwrap_or_default();

        let note_text = self.extract_string(raw, &[&["note_tweet_text"], &["noteTweetText"]]);
        let tweeted_at = self.parse_date(raw)?;

        let mut builder = BookmarkBuilder::new()
            .tweet_url(tweet_url)
            .content(content)
            .note_text(note_text.unwrap_or_default())
            .tweeted_at(tweeted_at)
            .author_handle(author_handle)
            .author_name(author_name);

        if let Some(img) = self.extract_string(
            raw,
            &[
                &["profile_image_url_https"],
                &["profile_image"],
                &["user", "profile_image_url_https"],
                &[
                    "core",
                    "user_results",
                    "result",
                    "legacy",
                    "profile_image_url_https",
                ],
            ],
        ) {
            builder = builder.author_profile_image(img);
        }

        if let Some(profile_url) =
            self.extract_string(raw, &[&["author_profile_url"], &["profile_url"]])
        {
            builder = builder.author_profile_url(profile_url);
        }

        if let Some(tags) = self.extract_tags(raw) {
            for tag in tags {
                builder = builder.add_tag(tag);
            }
        }

        for url in self.extract_media_urls(raw) {
            builder = builder.add_media(url);
        }

        let mut bookmark = builder.build().map_err(|e| Error::Other(e.to_string()))?;

        if bookmark.tags.is_empty() {
            for tag in bookmark.extract_hashtags() {
                if !bookmark.tags.contains(&tag) {
                    bookmark.tags.push(tag);
                }
            }
            bookmark.compute_search_text();
        }

        Ok(bookmark)
    }

    fn unwrap_entry<'a>(&self, raw: &'a Value) -> &'a Value {
        let mut current = raw;

        loop {
            let Some(object) = current.as_object() else {
                return current;
            };

            let mut next = None;
            for key in ["bookmark", "tweet", "item", "entry", "data", "value"] {
                if let Some(candidate) = object.get(key) {
                    next = Some(candidate);
                    break;
                }
            }

            match next {
                Some(candidate) => current = candidate,
                None => return current,
            }
        }
    }

    fn extract_string(&self, raw: &Value, paths: &[&[&str]]) -> Option<String> {
        paths
            .iter()
            .find_map(|path| Self::value_at_path(raw, path))
            .and_then(Self::value_to_string)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn extract_tags(&self, raw: &Value) -> Option<Vec<String>> {
        let tags = Self::value_at_path(raw, &["tags"])
            .or_else(|| Self::value_at_path(raw, &["entities", "hashtags"]))
            .or_else(|| Self::value_at_path(raw, &["legacy", "entities", "hashtags"]))?;

        let Value::Array(items) = tags else {
            return None;
        };

        let mut result = Vec::new();
        for item in items {
            if let Some(tag) = Self::value_to_string(item)
                .or_else(|| Self::value_at_path(item, &["text"]).and_then(Self::value_to_string))
            {
                let normalized = tag.trim().trim_start_matches('#').to_lowercase();
                if !normalized.is_empty() && !result.contains(&normalized) {
                    result.push(normalized);
                }
            }
        }

        Some(result)
    }

    fn extract_media_urls(&self, raw: &Value) -> Vec<String> {
        let mut urls = Vec::new();

        for path in [
            &["media"][..],
            &["extended_media"][..],
            &["entities", "media"][..],
            &["legacy", "entities", "media"][..],
        ] {
            let Some(Value::Array(items)) = Self::value_at_path(raw, path) else {
                continue;
            };

            for item in items {
                let url = Self::value_at_path(item, &["url"])
                    .or_else(|| Self::value_at_path(item, &["media_url"]))
                    .or_else(|| Self::value_at_path(item, &["media_url_https"]))
                    .and_then(Self::value_to_string);

                if let Some(url) = url {
                    if !urls.contains(&url) {
                        urls.push(url);
                    }
                }
            }
        }

        urls
    }

    fn extract_handle_from_url(&self, url: &str) -> Option<String> {
        let trimmed = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
        let path = trimmed.split_once('/').map(|(_, rest)| rest)?;
        let handle = path.split('/').next()?.trim();
        if handle.is_empty() || handle == "i" {
            return None;
        }
        Some(handle.trim_start_matches('@').to_string())
    }

    fn parse_date(&self, raw: &Value) -> Result<DateTime<Utc>> {
        let date_str = self.extract_string(
            raw,
            &[
                &["tweeted_at"],
                &["created_at"],
                &["createdAt"],
                &["bookmark_date"],
                &["date"],
                &["timestamp"],
                &["tweet", "created_at"],
                &["bookmark", "createdAt"],
            ],
        );

        let Some(s) = date_str else {
            return Err(Error::Other("Missing date".into()));
        };

        parse_date_candidates([Some(s.as_str()), None, None])
    }

    fn value_at_path<'a>(raw: &'a Value, path: &[&str]) -> Option<&'a Value> {
        let mut current = raw;
        for segment in path {
            current = current.get(*segment)?;
        }
        Some(current)
    }

    fn value_to_string(value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Number(number) => Some(number.to_string()),
            _ => None,
        }
    }
}

fn parse_date_candidates(candidates: [Option<&str>; 3]) -> Result<DateTime<Utc>> {
    for candidate in candidates.into_iter().flatten() {
        let s = candidate.trim().trim_matches('"');
        if let Ok(epoch) = s.parse::<i64>() {
            if let Some(dt) = DateTime::<Utc>::from_timestamp(epoch, 0) {
                return Ok(dt);
            }
        }
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Ok(dt) = DateTime::parse_from_str(s, "%a %b %d %H:%M:%S %z %Y") {
            return Ok(dt.with_timezone(&Utc));
        }
    }
    Err(Error::Other("Could not parse date".into()))
}

#[derive(Debug, Deserialize)]
struct FlatJsonBookmark {
    #[serde(default)]
    id_str: Option<String>,
    #[serde(default)]
    bookmark_date: Option<String>,
    #[serde(default)]
    profile_image_url_https: Option<String>,
    #[serde(default)]
    screen_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    full_text: Option<String>,
    #[serde(default)]
    note_tweet_text: Option<String>,
    #[serde(default)]
    tweeted_at: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    tweet_url: Option<String>,
    #[serde(default)]
    extended_media: Option<Vec<FlatMedia>>,
    #[serde(default)]
    media: Option<Vec<FlatMedia>>,
}

#[derive(Debug, Deserialize)]
struct FlatMedia {
    #[serde(default)]
    media_url_https: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parses_flat_json_fast_path() {
        let bookmarks = JsonParser::new()
            .parse_str(
                r#"[{"bookmark_date":"2025-08-25T17:25:01.047Z","profile_image_url_https":"https://pbs.twimg.com/profile.jpg","screen_name":"cinesansar","name":"persona","full_text":"hello #rust","tweeted_at":"2025-08-25T10:52:35.000Z","extended_media":[{"media_url_https":"https://pbs.twimg.com/media/test.jpg"}],"tweet_url":"https://x.com/cinesansar/status/123"}]"#,
            )
            .unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].author_handle, "cinesansar");
        assert_eq!(bookmarks[0].tags, vec!["rust"]);
        assert_eq!(bookmarks[0].media.len(), 1);
    }

    #[test]
    fn parses_array_json_export() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bookmarks.json");
        fs::write(
            &path,
            r#"[{"tweet_url":"https://x.com/rustlang/status/123","full_text":"Rust release day #rust","tweeted_at":"2024-05-01T14:30:00Z","screen_name":"rustlang","name":"Rust Language","media":[{"url":"https://pbs.twimg.com/media/test.jpg"}]}]"#,
        )
        .unwrap();

        let bookmarks = JsonParser::new().parse(&path).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].author_handle, "rustlang");
        assert_eq!(bookmarks[0].tags, vec!["rust"]);
        assert_eq!(bookmarks[0].media.len(), 1);
    }

    #[test]
    fn parses_archive_js_wrapper() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bookmarks.js");
        fs::write(
            &path,
            r#"window.YTD.bookmark.part0 = [{"bookmark": {"tweetId":"999","createdAt":"2024-08-25T10:52:35.000Z","screen_name":"sveltejs","name":"Svelte","full_text":"Runes are here","expandedUrl":"https://x.com/sveltejs/status/999"}}];"#,
        )
        .unwrap();

        let bookmarks = JsonParser::new().parse(&path).unwrap();
        assert_eq!(bookmarks.len(), 1);
        assert_eq!(bookmarks[0].tweet_url, "https://x.com/sveltejs/status/999");
        assert_eq!(bookmarks[0].author_name, "Svelte");
        assert_eq!(bookmarks[0].author_handle, "sveltejs");
    }
}
