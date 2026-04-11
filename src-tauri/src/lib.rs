//! Tauri application library
//!
//! Provides IPC commands for the frontend to interact with the Rust backend.

mod x_sync;

use eterea_core::{Bookmark, Database, Ingester};
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
use url::Url;
use x_sync::{
    build_failed_sync_status_from_previous, build_success_sync_status,
    import_bookmarks_from_x as sync_bookmarks_from_x, load_sync_status, metadata_key,
    serialize_sync_status, XImportSummary, XSessionToken, XSyncStatus,
};

pub struct AppState {
    db: Mutex<Database>,
    x_session: Mutex<Option<XSessionToken>>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    items: Vec<T>,
    total: i64,
    offset: usize,
    limit: usize,
    has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    total_bookmarks: i64,
    unique_authors: i64,
    unique_tags: i64,
    favorite_bookmarks: i64,
    earliest_date: Option<String>,
    latest_date: Option<String>,
    top_tags: Vec<(String, i64)>,
}

#[derive(Debug, Serialize)]
pub struct LinkPreview {
    url: String,
    final_url: String,
    title: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    site_name: Option<String>,
}

mod commands {
    use super::*;
    use reqwest::{header::CONTENT_TYPE, redirect::Policy};
    use scraper::{Html, Selector};

    fn trace_label(command: &str, trace_id: Option<&str>) -> String {
        match trace_id {
            Some(trace_id) if !trace_id.is_empty() => format!("[eterea][tauri][{command}][{trace_id}]"),
            _ => format!("[eterea][tauri][{command}]"),
        }
    }

    #[tauri::command]
    pub fn get_bookmarks(
        state: State<AppState>,
        offset: Option<usize>,
        limit: Option<usize>,
        trace_id: Option<String>,
    ) -> Result<PaginatedResponse<Bookmark>, String> {
        let label = trace_label("get_bookmarks", trace_id.as_deref());
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(50);
        eprintln!("{label} start offset={offset} limit={limit}");

        let lock_started = std::time::Instant::now();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let lock_elapsed = lock_started.elapsed();
        eprintln!("{label} db-lock={}ms", lock_elapsed.as_millis());

        let query_started = std::time::Instant::now();
        let bookmarks = db.get_bookmarks(offset, limit).map_err(|e| e.to_string())?;
        let total = db.count_bookmarks().map_err(|e| e.to_string())?;
        let has_more = (offset + bookmarks.len()) < total as usize;
        eprintln!(
            "{label} done items={} total={} query={}ms",
            bookmarks.len(),
            total,
            query_started.elapsed().as_millis()
        );

        Ok(PaginatedResponse {
            items: bookmarks,
            total,
            offset,
            limit,
            has_more,
        })
    }

    #[tauri::command]
    pub fn search_bookmarks(
        state: State<AppState>,
        query: Option<String>,
        tag: Option<String>,
        limit: Option<usize>,
    ) -> Result<Vec<Bookmark>, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let limit = limit.unwrap_or(100);

        if let Some(tag) = tag {
            return db
                .get_bookmarks_by_tag(&tag, 0, limit)
                .map_err(|e| e.to_string());
        }

        if let Some(query) = query {
            if !query.trim().is_empty() {
                return db.search(&query, limit).map_err(|e| e.to_string());
            }
        }

        db.get_bookmarks(0, limit).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn get_stats(
        state: State<AppState>,
        trace_id: Option<String>,
    ) -> Result<StatsResponse, String> {
        let label = trace_label("get_stats", trace_id.as_deref());
        eprintln!("{label} start");
        let lock_started = std::time::Instant::now();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        eprintln!("{label} db-lock={}ms", lock_started.elapsed().as_millis());
        let stats_started = std::time::Instant::now();
        let stats = db.get_stats().map_err(|e| e.to_string())?;
        eprintln!(
            "{label} done total={} tags={} stats={}ms",
            stats.total_bookmarks,
            stats.unique_tags,
            stats_started.elapsed().as_millis()
        );

        Ok(StatsResponse {
            total_bookmarks: stats.total_bookmarks,
            unique_authors: stats.unique_authors,
            unique_tags: stats.unique_tags,
            favorite_bookmarks: stats.favorite_bookmarks,
            earliest_date: stats.earliest_date.map(|d| d.to_rfc3339()),
            latest_date: stats.latest_date.map(|d| d.to_rfc3339()),
            top_tags: stats.top_tags,
        })
    }

    #[tauri::command]
    pub fn import_file(state: State<AppState>, path: String) -> Result<usize, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let ingester = Ingester::new();
        let path = PathBuf::from(path);

        ingester.ingest_file(&path, &db).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn import_bookmarks_content(
        state: State<AppState>,
        filename: String,
        content: String,
    ) -> Result<usize, String> {
        let extension = PathBuf::from(&filename)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_string();
        let ingester = Ingester::new();
        let bookmarks = ingester
            .parse_content(&extension, &content)
            .map_err(|e| e.to_string())?;
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.insert_bookmarks(&bookmarks).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn delete_bookmark(state: State<AppState>, id: String) -> Result<bool, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.delete_bookmark(&id).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn toggle_favorite(state: State<AppState>, id: String) -> Result<bool, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.toggle_favorite(&id).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn get_favorites(
        state: State<AppState>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<Bookmark>, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(50);

        db.get_favorites(offset, limit).map_err(|e| e.to_string())
    }

    #[allow(clippy::too_many_arguments)]
    #[tauri::command]
    pub fn search_with_filters(
        state: State<AppState>,
        query: Option<String>,
        tag: Option<String>,
        author: Option<String>,
        from_date: Option<String>,
        to_date: Option<String>,
        offset: Option<usize>,
        favorites_only: Option<bool>,
        has_media: Option<bool>,
        limit: Option<usize>,
        trace_id: Option<String>,
    ) -> Result<PaginatedResponse<Bookmark>, String> {
        let label = trace_label("search_with_filters", trace_id.as_deref());
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(100);
        eprintln!(
            "{label} start offset={} limit={} query={:?} tag={:?} author={:?} favorites_only={} has_media={:?}",
            offset,
            limit,
            query,
            tag,
            author,
            favorites_only.unwrap_or(false),
            has_media
        );

        let lock_started = std::time::Instant::now();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        eprintln!("{label} db-lock={}ms", lock_started.elapsed().as_millis());

        let from = from_date
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc));
        let to = to_date
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc));

        let query_started = std::time::Instant::now();
        let (items, total) = db
            .search_with_filters_page(
            query.as_deref(),
            tag.as_deref(),
            author.as_deref(),
            from,
            to,
            favorites_only.unwrap_or(false),
            has_media,
            offset,
            limit,
        )
        .map_err(|e| e.to_string())?;

        let has_more = (offset + items.len()) < total as usize;
        eprintln!(
            "{label} done items={} total={} query={}ms",
            items.len(),
            total,
            query_started.elapsed().as_millis()
        );

        Ok(PaginatedResponse {
            items,
            total,
            offset,
            limit,
            has_more,
        })
    }

    #[tauri::command]
    pub fn get_x_sync_status(state: State<AppState>) -> Result<XSyncStatus, String> {
        let metadata = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            db.get_metadata(metadata_key()).map_err(|e| e.to_string())?
        };
        let session = state.x_session.lock().map_err(|e| e.to_string())?;
        Ok(load_sync_status(metadata, session.as_ref()))
    }

    #[tauri::command]
    pub async fn import_bookmarks_from_x(
        state: State<'_, AppState>,
    ) -> Result<XImportSummary, String> {
        let existing_session = {
            let session = state.x_session.lock().map_err(|e| e.to_string())?;
            session.clone()
        };

        let outcome = match sync_bookmarks_from_x(existing_session).await {
            Ok(outcome) => outcome,
            Err(error) => {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let previous = db.get_metadata(metadata_key()).map_err(|e| e.to_string())?;
                let failed = build_failed_sync_status_from_previous(previous, error.clone());
                let metadata = serialize_sync_status(&failed)?;
                db.set_metadata(metadata_key(), &metadata)
                    .map_err(|e| e.to_string())?;
                return Err(error);
            }
        };

        let total_fetched = outcome.bookmarks.len();
        let imported_count = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let imported = db
                .insert_bookmarks(&outcome.bookmarks)
                .map_err(|e| e.to_string())?;
            let summary = XImportSummary {
                imported_count: imported,
                skipped_count: total_fetched.saturating_sub(imported),
                total_fetched,
                last_synced_at: chrono::Utc::now().to_rfc3339(),
                reauthenticated: outcome.reauthenticated,
                status_message: if imported == 0 {
                    "X import finished, but everything was already in your library.".to_string()
                } else {
                    format!("Imported {imported} bookmarks from X into your local library.")
                },
            };
            let persisted = build_success_sync_status(&summary);
            let metadata = serialize_sync_status(&persisted)?;
            db.set_metadata(metadata_key(), &metadata)
                .map_err(|e| e.to_string())?;
            summary
        };

        {
            let mut session = state.x_session.lock().map_err(|e| e.to_string())?;
            *session = Some(outcome.session);
        }

        Ok(imported_count)
    }

    #[tauri::command]
    pub async fn open_url(url: String) -> Result<(), String> {
        let parsed = parse_public_https_url(&url)?;
        open::that(parsed.as_str()).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn fetch_link_preview(url: String) -> Result<LinkPreview, String> {
        const MAX_LINK_PREVIEW_BYTES: u64 = 512 * 1024;
        let parsed_url = parse_public_https_url(&url)?;

        let client = reqwest::Client::builder()
            .user_agent("Eterea/0.1 (+https://github.com/eterea)")
            .redirect(Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    return attempt.error("Too many redirects.");
                }

                if parse_public_https_url(attempt.url().as_str()).is_err() {
                    return attempt.error("Redirect target is not allowed.");
                }

                attempt.follow()
            }))
            .connect_timeout(Duration::from_millis(800))
            .timeout(Duration::from_millis(1_500))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .get(parsed_url.clone())
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let final_url = response.url().to_string();
        parse_public_https_url(&final_url)?;
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if !content_type.is_empty()
            && !content_type.starts_with("text/html")
            && !content_type.starts_with("application/xhtml+xml")
        {
            return Err("Link preview only supports HTML pages.".to_string());
        }

        if response
            .content_length()
            .map(|length| length > MAX_LINK_PREVIEW_BYTES)
            .unwrap_or(false)
        {
            return Err("Link preview content too large.".to_string());
        }

        let body = response.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&body);

        let title = extract_meta(&document, &["og:title", "twitter:title"])
            .or_else(|| extract_title(&document));
        let description = extract_meta(
            &document,
            &["og:description", "twitter:description", "description"],
        );
        let image_url = extract_meta(&document, &["og:image", "twitter:image"]);
        let site_name = extract_meta(&document, &["og:site_name"]);

        Ok(LinkPreview {
            url: parsed_url.to_string(),
            final_url,
            title,
            description,
            image_url,
            site_name,
        })
    }

    fn parse_public_https_url(raw: &str) -> Result<Url, String> {
        let parsed = Url::parse(raw).map_err(|_| "Invalid URL.".to_string())?;
        if parsed.scheme() != "https" {
            return Err("Only HTTPS URLs are allowed.".to_string());
        }

        let host = parsed
            .host_str()
            .ok_or_else(|| "URL must include a hostname.".to_string())?
            .to_ascii_lowercase();

        if host == "localhost" || host.ends_with(".local") || host.ends_with(".internal") {
            return Err("Local network URLs are not allowed.".to_string());
        }

        if !host_resolves_to_public_ip(&host, parsed.port_or_known_default().unwrap_or(443))? {
            return Err("Private or local network URLs are not allowed.".to_string());
        }

        Ok(parsed)
    }

    fn host_resolves_to_public_ip(host: &str, port: u16) -> Result<bool, String> {
        let resolved = (host, port)
            .to_socket_addrs()
            .map_err(|_| "Unable to resolve URL host.".to_string())?;

        let mut found_public = false;
        for address in resolved {
            if !is_public_ip(address.ip()) {
                return Ok(false);
            }
            found_public = true;
        }

        Ok(found_public)
    }

    fn is_public_ip(ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => is_public_ipv4(ipv4),
            IpAddr::V6(ipv6) => is_public_ipv6(ipv6),
        }
    }

    fn is_public_ipv4(ip: Ipv4Addr) -> bool {
        let octets = ip.octets();
        if ip.is_private()
            || ip.is_loopback()
            || ip.is_link_local()
            || ip.is_multicast()
            || ip.is_unspecified()
        {
            return false;
        }

        if octets[0] == 0
            || (octets[0] == 100 && (64..=127).contains(&octets[1]))
            || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
            || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
            || (octets[0] == 198 && octets[1] == 18)
            || (octets[0] == 198 && octets[1] == 19)
            || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
            || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
        {
            return false;
        }

        true
    }

    fn is_public_ipv6(ip: Ipv6Addr) -> bool {
        if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
            return false;
        }

        let segments = ip.segments();
        let first = segments[0];
        if (first & 0xfe00) == 0xfc00 || (first & 0xffc0) == 0xfe80 {
            return false;
        }

        true
    }

    fn extract_meta(document: &Html, keys: &[&str]) -> Option<String> {
        let selector = Selector::parse("meta").ok()?;
        'outer: for element in document.select(&selector) {
            let value = element.value();
            for key in keys {
                if value
                    .attr("property")
                    .map(|p| p.eq_ignore_ascii_case(key))
                    .unwrap_or(false)
                    || value
                        .attr("name")
                        .map(|n| n.eq_ignore_ascii_case(key))
                        .unwrap_or(false)
                {
                    if let Some(content) = value.attr("content") {
                        if !content.is_empty() {
                            return Some(content.to_string());
                        }
                    }
                    continue 'outer;
                }
            }
        }
        None
    }

    fn extract_title(document: &Html) -> Option<String> {
        let selector = Selector::parse("title").ok()?;
        document
            .select(&selector)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::open_default().expect("Failed to open database");

    let state = AppState {
        db: Mutex::new(db),
        x_session: Mutex::new(None),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_bookmarks,
            commands::search_bookmarks,
            commands::get_stats,
            commands::import_file,
            commands::import_bookmarks_content,
            commands::delete_bookmark,
            commands::toggle_favorite,
            commands::get_favorites,
            commands::search_with_filters,
            commands::get_x_sync_status,
            commands::import_bookmarks_from_x,
            commands::open_url,
            commands::fetch_link_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
