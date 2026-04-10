//! Tauri application library
//!
//! Provides IPC commands for the frontend to interact with the Rust backend.

mod x_sync;

use eterea_core::{Bookmark, Database, Ingester};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
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

    #[tauri::command]
    pub fn get_bookmarks(
        state: State<AppState>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<PaginatedResponse<Bookmark>, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(50);

        let bookmarks = db.get_bookmarks(offset, limit).map_err(|e| e.to_string())?;
        let total = db.count_bookmarks().map_err(|e| e.to_string())?;
        let has_more = (offset + bookmarks.len()) < total as usize;

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
    pub fn get_stats(state: State<AppState>) -> Result<StatsResponse, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let stats = db.get_stats().map_err(|e| e.to_string())?;

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
        favorites_only: Option<bool>,
        has_media: Option<bool>,
        limit: Option<usize>,
    ) -> Result<Vec<Bookmark>, String> {
        let db = state.db.lock().map_err(|e| e.to_string())?;

        let from = from_date
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc));
        let to = to_date
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.with_timezone(&chrono::Utc));

        db.search_with_filters(
            query.as_deref(),
            tag.as_deref(),
            author.as_deref(),
            from,
            to,
            favorites_only.unwrap_or(false),
            has_media,
            limit.unwrap_or(100),
        )
        .map_err(|e| e.to_string())
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
        open::that(&url).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn fetch_link_preview(url: String) -> Result<LinkPreview, String> {
        const MAX_LINK_PREVIEW_BYTES: u64 = 512 * 1024;

        let client = reqwest::Client::builder()
            .user_agent("Eterea/0.1 (+https://github.com/eterea)")
            .redirect(Policy::limited(5))
            .connect_timeout(Duration::from_millis(800))
            .timeout(Duration::from_millis(1_500))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        let final_url = response.url().to_string();
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
            url,
            final_url,
            title,
            description,
            image_url,
            site_name,
        })
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
