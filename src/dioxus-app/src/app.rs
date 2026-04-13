use chrono::{DateTime, Local};
use dioxus::prelude::*;
use eterea_app::{AppServices, BookmarkQuery, BookmarkStats};
use eterea_core::Bookmark;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

const APP_CSS: &str = include_str!("../assets/app.css");
const PAGE_SIZE: usize = 48;

#[derive(Clone, Default, PartialEq)]
struct Filters {
    query: String,
    selected_tag: Option<String>,
    favorites_only: bool,
}

#[derive(Clone, PartialEq)]
enum LayoutMode {
    Focus,
    Grid,
    List,
}

impl LayoutMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Focus => "Focus",
            Self::Grid => "Grid",
            Self::List => "List",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            Self::Focus => "The calmest, most spacious reading mode.",
            Self::Grid => "Balanced density when you want more on screen.",
            Self::List => "A faster skim for larger archives.",
        }
    }

    fn class_name(&self) -> &'static str {
        match self {
            Self::Focus => "feed-focus",
            Self::Grid => "feed-grid",
            Self::List => "feed-list",
        }
    }
}

#[derive(Clone, Default, PartialEq)]
struct ImportState {
    open: bool,
    path: String,
    message: Option<String>,
    error: Option<String>,
}

#[derive(Clone, PartialEq)]
struct LibraryState {
    bookmarks: Vec<Bookmark>,
    stats: Option<BookmarkStats>,
    top_tags: Vec<(String, i64)>,
    filters: Filters,
    layout: LayoutMode,
    total: i64,
    has_more: bool,
    page_size: usize,
    status: String,
    error: Option<String>,
    import: ImportState,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            bookmarks: Vec::new(),
            stats: None,
            top_tags: Vec::new(),
            filters: Filters::default(),
            layout: LayoutMode::Focus,
            total: 0,
            has_more: false,
            page_size: PAGE_SIZE,
            status: "Archive ready.".to_string(),
            error: None,
            import: ImportState::default(),
        }
    }
}

type Services = Rc<RefCell<AppServices>>;

#[component]
pub fn App() -> Element {
    let services = use_hook(|| {
        Rc::new(RefCell::new(
            AppServices::open_default().expect("failed to open Eterea services"),
        ))
    });
    let mut state = use_signal(|| load_initial_state(&services));
    let home_services = services.clone();
    let favorites_services = services.clone();
    let search_submit_services = services.clone();
    let search_key_services = services.clone();
    let filter_toggle_services = services.clone();
    let reset_view_services = services.clone();
    let load_more_services = services.clone();
    let import_services = services.clone();

    let status_message = state.read().status.clone();
    let error_message = state.read().error.clone();
    let top_tags = state.read().top_tags.clone();
    let total = state.read().total;
    let bookmarks = state.read().bookmarks.clone();
    let filters = state.read().filters.clone();
    let layout = state.read().layout.clone();
    let has_more = state.read().has_more;
    let import_state = state.read().import.clone();
    let unique_authors = state
        .read()
        .stats
        .as_ref()
        .map(|stats| stats.unique_authors)
        .unwrap_or(0);
    let tag_buttons = top_tags.clone().into_iter().take(6).map(|(tag, count)| {
        let tag_services = services.clone();
        let is_active = filters.selected_tag.as_deref() == Some(tag.as_str());
        rsx! {
            button {
                class: if is_active { "tag-pill active" } else { "tag-pill" },
                onclick: move |_| {
                    {
                        let mut next = state.write();
                        next.filters.selected_tag = Some(tag.clone());
                        next.filters.favorites_only = false;
                        next.error = None;
                    }
                    reload_library(&tag_services, &mut state);
                },
                span { "#{tag}" }
                small { "{count}" }
            }
        }
    });
    let bookmark_cards = bookmarks.clone().into_iter().map(|bookmark| {
        let toggle_services = services.clone();
        rsx! {
            BookmarkCard {
                key: "{bookmark.id}",
                bookmark,
                on_toggle_favorite: move |id: String| {
                    match toggle_services.borrow().toggle_favorite(&id) {
                        Ok(_) => {
                            reload_library(&toggle_services, &mut state);
                            state.write().status = "Favorite updated.".to_string();
                        }
                        Err(error) => state.write().error = Some(error.to_string()),
                    }
                }
            }
        }
    });

    rsx! {
        document::Title { "Eterea — Dioxus Library" }
        style { "{APP_CSS}" }
        div {
            class: "app-shell",
            aside {
                class: "left-rail",
                div {
                    class: "brand-card panel",
                    p { class: "eyebrow", "Local-first archive" }
                    h1 { class: "brand-mark", "Eterea" }
                    p {
                        class: "muted-copy",
                        "A calm reading room for saved tweets — fast to open, quiet to browse, easy to keep useful."
                    }
                    button {
                        class: "accent-button wide",
                        onclick: move |_| state.write().import.open = true,
                        "Import bookmarks"
                    }
                    p { class: "chip-row" }
                    div { class: "subtle-chip", "desktop-first MVP" }
                    div { class: "subtle-chip", "rust backend preserved" }
                }

                div {
                    class: "panel nav-panel",
                    p { class: "eyebrow", "Navigate" }
                    button {
                        class: if filters.favorites_only || filters.selected_tag.is_some() || !filters.query.is_empty() { "nav-link" } else { "nav-link active" },
                        onclick: move |_| {
                            let mut next = state.write();
                            next.filters = Filters::default();
                            next.error = None;
                            drop(next);
                            reload_library(&home_services, &mut state);
                        },
                        span { "Library" }
                        small { "Everything you saved" }
                    }
                    button {
                        class: if filters.favorites_only { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            {
                                let mut next = state.write();
                                next.filters.favorites_only = true;
                                next.filters.selected_tag = None;
                                next.error = None;
                            }
                            reload_library(&favorites_services, &mut state);
                        },
                        span { "Favorites" }
                        small { "Pinned to revisit" }
                    }
                }

                div {
                    class: "panel tag-panel",
                    p { class: "eyebrow", "Top tags" }
                    if top_tags.is_empty() {
                        p { class: "muted-copy", "Tags appear here once the archive metadata finishes loading." }
                    } else {
                        {tag_buttons}
                    }
                }
            }

            main {
                class: "main-column",
                section {
                    class: "panel hero-panel",
                    div {
                        class: "hero-copy",
                        div {
                            class: "pill-row",
                            span { class: "pill", "Library" }
                            span { class: "pill", "local-first" }
                            span { class: "pill", "{total} saved" }
                        }
                        h2 { class: "hero-title", "Read what you saved without the rest of the internet shouting over it." }
                        p {
                            class: "muted-copy hero-subtitle",
                            "Saved tweets, kept quiet and easy to read. Everything stays tuned for fast open, quick filtering, and a calmer archive that keeps tweet content first."
                        }
                    }
                    div {
                        class: "hero-actions",
                        button {
                            class: "ghost-button",
                            onclick: move |_| state.write().layout = LayoutMode::Focus,
                            "Tune layout"
                        }
                        button {
                            class: "accent-button",
                            onclick: move |_| state.write().import.open = true,
                            "Import bookmarks"
                        }
                    }
                    div {
                        class: "search-row",
                        input {
                            class: "search-input",
                            r#type: "text",
                            value: "{filters.query}",
                            placeholder: "Search your archive by text or author",
                            oninput: move |event| state.write().filters.query = event.value(),
                            onkeydown: move |event| {
                                if event.key() == Key::Enter {
                                    reload_library(&search_key_services, &mut state);
                                }
                            }
                        }
                        button {
                            class: "ghost-button",
                            onclick: move |_| reload_library(&search_submit_services, &mut state),
                            "Search"
                        }
                    }
                    div {
                        class: "layout-row",
                        button {
                            class: if filters.favorites_only { "subtle-chip active" } else { "subtle-chip" },
                            onclick: move |_| {
                                {
                                    let mut next = state.write();
                                    next.filters.favorites_only = !next.filters.favorites_only;
                                    next.error = None;
                                }
                                reload_library(&filter_toggle_services, &mut state);
                            },
                            "Favorites only"
                        }
                        for candidate in [LayoutMode::Focus, LayoutMode::Grid, LayoutMode::List] {
                            button {
                                class: if layout == candidate { "layout-pill active" } else { "layout-pill" },
                                title: "{candidate.description()}",
                                onclick: move |_| state.write().layout = candidate.clone(),
                                "{candidate.as_str()}"
                            }
                        }
                    }
                }

                section {
                    class: "panel library-panel",
                    div {
                        class: "library-header",
                        div {
                            p { class: "eyebrow", "Reading feed" }
                            h3 { class: "section-title", "Library" }
                            p {
                                class: "muted-copy",
                                "Showing {bookmarks.len()} of {total} bookmarks • {unique_authors} authors in the archive."
                            }
                        }
                        div {
                            class: "library-actions",
                            button {
                                class: "ghost-button",
                                onclick: move |_| {
                                    state.write().filters = Filters::default();
                                    reload_library(&reset_view_services, &mut state);
                                },
                                "Reset view"
                            }
                            button {
                                class: "ghost-button",
                                onclick: move |_| state.write().import.open = true,
                                "Import"
                            }
                        }
                    }

                    if let Some(error) = error_message {
                        div { class: "error-card", strong { "Couldn’t load the archive." } p { "{error}" } }
                    } else if bookmarks.is_empty() {
                        div {
                            class: "empty-card",
                            p { class: "eyebrow", "Nothing here yet" }
                            h4 { "The archive is quiet." }
                            p { class: "muted-copy", "Import a bookmark export to fill the library back in." }
                            button {
                                class: "accent-button",
                                onclick: move |_| state.write().import.open = true,
                                "Import bookmarks"
                            }
                        }
                    } else {
                        div { class: "bookmark-feed {layout.class_name()}",
                            {bookmark_cards}
                        }
                        if has_more {
                            button {
                                class: "ghost-button wide load-more",
                                onclick: move |_| load_more(&load_more_services, &mut state),
                                "Load more"
                            }
                        }
                    }
                }
            }
        }

        if import_state.open {
            div {
                class: "modal-backdrop",
                onclick: move |_| state.write().import.open = false,
                div {
                    class: "modal panel",
                    onclick: move |event| event.stop_propagation(),
                    p { class: "eyebrow", "Bring more into the room" }
                    h3 { class: "section-title", "Import bookmarks" }
                    p { class: "muted-copy", "Paste a local path to a CSV, JSON, or X archive JS export. The importer keeps using the existing Rust backend parser and local SQLite storage." }
                    input {
                        class: "path-input",
                        r#type: "text",
                        value: "{import_state.path}",
                        placeholder: "/home/you/Downloads/bookmarks.json",
                        oninput: move |event| state.write().import.path = event.value(),
                    }
                    if let Some(message) = import_state.message {
                        p { class: "success-copy", "{message}" }
                    }
                    if let Some(error) = import_state.error {
                        p { class: "error-copy", "{error}" }
                    }
                    div {
                        class: "modal-actions",
                        button {
                            class: "ghost-button",
                            onclick: move |_| state.write().import.open = false,
                            "Close"
                        }
                        button {
                            class: "accent-button",
                            onclick: move |_| {
                                let path = PathBuf::from(state.read().import.path.trim());
                                if path.as_os_str().is_empty() {
                                    state.write().import.error = Some("Enter a file path before importing.".to_string());
                                    return;
                                }
                                match import_services.borrow().import_file(&path) {
                                    Ok(imported) => {
                                        {
                                            let mut next = state.write();
                                            next.import.error = None;
                                            next.import.message = Some(format!("Imported {imported} bookmarks from {}.", path.display()));
                                            next.status = format!("Imported {imported} bookmarks.");
                                        }
                                        reload_library(&import_services, &mut state);
                                    }
                                    Err(error) => {
                                        state.write().import.error = Some(error.to_string());
                                    }
                                }
                            },
                            "Import file"
                        }
                    }
                    p { class: "muted-copy tiny", "Direct X sync and richer settings remain deferred in this first Dioxus pass." }
                }
            }
        }

        footer { class: "status-bar", "{status_message}" }
    }
}

#[component]
fn BookmarkCard(bookmark: Bookmark, on_toggle_favorite: EventHandler<String>) -> Element {
    let tweeted_at = format_timestamp(&bookmark.tweeted_at.to_rfc3339());
    let media_count = bookmark.media.len();
    let favorite_label = if bookmark.is_favorite {
        "★ Favorited"
    } else {
        "☆ Favorite"
    };

    rsx! {
        article {
            class: "bookmark-card panel",
            div {
                class: "bookmark-meta",
                div {
                    strong { "@{bookmark.author_handle}" }
                    span { "{bookmark.author_name}" }
                }
                span { "{tweeted_at}" }
            }
            p { class: "bookmark-content", "{bookmark.content}" }
            if let Some(note) = &bookmark.note_text {
                p { class: "bookmark-note", "{note}" }
            }
            if !bookmark.tags.is_empty() {
                div {
                    class: "tag-list",
                    for tag in &bookmark.tags {
                        span { class: "mini-tag", "#{tag}" }
                    }
                }
            }
            div {
                class: "bookmark-footer",
                div {
                    class: "bookmark-stats",
                    span { "{media_count} media" }
                    span { "Imported {format_timestamp(&bookmark.imported_at.to_rfc3339())}" }
                }
                button {
                    class: "ghost-button small",
                    onclick: move |_| on_toggle_favorite.call(bookmark.id.clone()),
                    "{favorite_label}"
                }
            }
        }
    }
}

fn load_initial_state(services: &Services) -> LibraryState {
    let mut state = LibraryState::default();
    refresh_from_services(services, &mut state, false);
    state
}

fn reload_library(services: &Services, state: &mut Signal<LibraryState>) {
    let mut next = state.write();
    refresh_from_services(services, &mut next, false);
}

fn load_more(services: &Services, state: &mut Signal<LibraryState>) {
    let query = {
        let current = state.read();
        BookmarkQuery {
            query: query_or_none(&current.filters.query),
            tag: current.filters.selected_tag.clone(),
            favorites_only: current.filters.favorites_only,
            offset: current.bookmarks.len(),
            limit: current.page_size,
            ..BookmarkQuery::default()
        }
    };

    match services.borrow().query_bookmarks(&query) {
        Ok(page) => {
            let mut next = state.write();
            next.bookmarks.extend(page.items);
            next.total = page.total;
            next.has_more = page.has_more;
            next.status = "Loaded more bookmarks.".to_string();
            next.error = None;
        }
        Err(error) => state.write().error = Some(error.to_string()),
    }
}

fn refresh_from_services(services: &Services, state: &mut LibraryState, preserve_status: bool) {
    let query = BookmarkQuery {
        query: query_or_none(&state.filters.query),
        tag: state.filters.selected_tag.clone(),
        favorites_only: state.filters.favorites_only,
        offset: 0,
        limit: state.page_size,
        ..BookmarkQuery::default()
    };

    match services.borrow().query_bookmarks(&query) {
        Ok(page) => {
            state.bookmarks = page.items;
            state.total = page.total;
            state.has_more = page.has_more;
            state.error = None;
            if !preserve_status {
                state.status = if state.total == 0 {
                    "Archive is empty — import a bookmark export to begin.".to_string()
                } else {
                    format!("Archive ready — {} bookmarks loaded.", state.total)
                };
            }
        }
        Err(error) => {
            state.error = Some(error.to_string());
            state.bookmarks.clear();
            state.total = 0;
            state.has_more = false;
        }
    }

    match services.borrow().stats() {
        Ok(stats) => {
            state.top_tags = stats.top_tags.clone();
            state.stats = Some(stats);
        }
        Err(error) => {
            state.error = Some(error.to_string());
            state.stats = None;
            state.top_tags.clear();
        }
    }
}

fn format_timestamp(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| {
            parsed
                .with_timezone(&Local)
                .format("%b %-d, %Y")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_string())
}

fn query_or_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
