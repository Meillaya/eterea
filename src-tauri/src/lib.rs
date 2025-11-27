//! Tauri application library
//!
//! Provides IPC commands for the frontend to interact with the Rust backend.

use eterea_core::{Database, Ingester, Bookmark};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    db: Mutex<Database>,
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
    use reqwest::redirect::Policy;
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

        let stats = db.get_stats().map_err(|e| e.to_string())?;
        let total = stats.total_bookmarks;
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

        ingester
            .ingest_file(&path, &db)
            .map_err(|e| e.to_string())
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
    pub async fn open_url(url: String) -> Result<(), String> {
        open::that(&url).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn fetch_link_preview(url: String) -> Result<LinkPreview, String> {
        let client = reqwest::Client::builder()
            .user_agent("Eterea/0.1 (+https://github.com/eterea)")
            .redirect(Policy::limited(5))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let final_url = response.url().to_string();
        let body = response.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&body);

        let title = extract_meta(&document, &["og:title", "twitter:title"])
            .or_else(|| extract_title(&document));
        let description = extract_meta(&document, &["og:description", "twitter:description", "description"]);
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
            .and_then(|el| Some(el.text().collect::<String>().trim().to_string()))
            .filter(|s| !s.is_empty())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::open_default().expect("Failed to open database");

    let state = AppState {
        db: Mutex::new(db),
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
            commands::delete_bookmark,
            commands::toggle_favorite,
            commands::get_favorites,
            commands::search_with_filters,
            commands::open_url,
            commands::fetch_link_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

