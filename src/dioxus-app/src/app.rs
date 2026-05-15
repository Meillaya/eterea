mod actions;
mod components;
mod design_system;
mod route;
mod screens;
mod state;

use actions::{
    apply_import_error, apply_import_preview, apply_import_success, clear_expanded_bookmark,
    count_active_filters, load_initial_state, load_more, mark_importing, move_expanded_bookmark,
    reload_library, set_import_source, set_remote_images_enabled, toggle_expanded_bookmark,
};
use components::{BookmarkCard, MediaGallery, MediaGalleryMode};
use design_system::{shell_classes, Density, PaperTone};
use dioxus::prelude::*;
use eterea_app::AppServices;
use route::ScreenRoute;
use state::{Filters, ImportStage, LayoutMode};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

const APP_CSS: &str = include_str!("../assets/app.css");

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
    let search_query_key_services = services.clone();
    let search_author_key_services = services.clone();
    let filter_toggle_services = services.clone();
    let media_toggle_services = services.clone();
    let reset_view_services = services.clone();
    let load_more_services = services.clone();
    let import_preview_services = services.clone();
    let import_commit_services = services.clone();

    let status_message = state.read().status.clone();
    let error_message = state.read().error.clone();
    let top_tags = state.read().top_tags.clone();
    let total = state.read().total;
    let bookmarks = state.read().bookmarks.clone();
    let filters = state.read().filters.clone();
    let layout = state.read().layout.clone();
    let has_more = state.read().has_more;
    let import_state = state.read().import.clone();
    let appearance = state.read().appearance.clone();
    let remote_images_enabled = state.read().remote_images_enabled;
    let import_button_label = if import_state.preview.is_some() {
        "Import preview"
    } else {
        "Import without preview"
    };
    let expanded_bookmark_id = state.read().expanded_bookmark_id.clone();
    let route = state.read().route.clone();
    let authors = state.read().authors.clone();
    let topics = state.read().topics.clone();
    let detail_bookmark = match &route {
        ScreenRoute::Entry(id) => bookmarks
            .iter()
            .find(|bookmark| bookmark.id == *id)
            .cloned(),
        _ => None,
    };
    let section_title = match &route {
        ScreenRoute::Favorites => "Favorites".to_string(),
        ScreenRoute::Authors => "Authors".to_string(),
        ScreenRoute::Topics => "Topics".to_string(),
        ScreenRoute::Search => "Search".to_string(),
        ScreenRoute::Import => "Import".to_string(),
        ScreenRoute::Settings => "Settings".to_string(),
        ScreenRoute::Onboarding => "Onboarding".to_string(),
        ScreenRoute::Entry(_) => "Entry detail".to_string(),
        ScreenRoute::Author(handle) => format!("@{handle}"),
        ScreenRoute::Topic(tag) => format!("#{tag}"),
        ScreenRoute::Library => "Library".to_string(),
    };
    let active_filter_count = count_active_filters(&filters);
    let has_active_filters = active_filter_count > 0;
    let shell_class = shell_classes(appearance.paper_tone, appearance.density);
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
                    state.write().route = ScreenRoute::Topic(tag.clone());
                    reload_library(&tag_services, &mut state);
                },
                span { "#{tag}" }
                small { "{count}" }
            }
        }
    });
    let bookmark_cards = bookmarks.clone().into_iter().map(|bookmark| {
        let toggle_services = services.clone();
        let delete_services = services.clone();
        let author_services = services.clone();
        rsx! {
            BookmarkCard {
                key: "{bookmark.id}",
                expanded: expanded_bookmark_id.as_deref() == Some(bookmark.id.as_str()),
                on_toggle_expand: move |id: String| {
                    let mut next = state.write();
                    toggle_expanded_bookmark(&mut next, id);
                },
                on_open_detail: move |id: String| {
                    state.write().route = ScreenRoute::Entry(id);
                },
                bookmark,
                remote_images_enabled,
                on_enable_remote_images: move |_| set_remote_images_enabled(&mut state, true),
                on_filter_author: move |author: String| {
                    {
                        let mut next = state.write();
                        next.filters.author_query = author;
                        next.error = None;
                    }
                    reload_library(&author_services, &mut state);
                },
                on_toggle_favorite: move |id: String| {
                    match toggle_services.borrow().toggle_favorite(&id) {
                        Ok(_) => {
                            reload_library(&toggle_services, &mut state);
                            state.write().status = "Favorite updated.".to_string();
                        }
                        Err(error) => state.write().error = Some(error.to_string()),
                    }
                },
                on_delete: move |id: String| {
                    match delete_services.borrow().delete_bookmark(&id) {
                        Ok(true) => {
                            reload_library(&delete_services, &mut state);
                            state.write().status = "Bookmark deleted.".to_string();
                        }
                        Ok(false) => {
                            state.write().error =
                                Some("Bookmark could not be deleted.".to_string());
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
            class: "{shell_class}",
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
                        class: if route == ScreenRoute::Library && !has_active_filters { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            let mut next = state.write();
                            next.route = ScreenRoute::Library;
                            next.filters = Filters::default();
                            next.error = None;
                            drop(next);
                            reload_library(&home_services, &mut state);
                        },
                        span { "Library" }
                        small { "Everything you saved" }
                    }
                    button {
                        class: if route == ScreenRoute::Favorites { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            {
                                let mut next = state.write();
                                next.filters.favorites_only = true;
                                next.filters.selected_tag = None;
                                next.route = ScreenRoute::Favorites;
                                next.error = None;
                            }
                            reload_library(&favorites_services, &mut state);
                        },
                        span { "Favorites" }
                        small { "Pinned to revisit" }
                    }
                    button {
                        class: if route == ScreenRoute::Authors { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            state.write().route = ScreenRoute::Authors;
                        },
                        span { "Authors" }
                        small { "Voices" }
                    }
                    button {
                        class: if route == ScreenRoute::Topics { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            state.write().route = ScreenRoute::Topics;
                        },
                        span { "Topics" }
                        small { "Tags" }
                    }
                    button {
                        class: if route == ScreenRoute::Search { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            state.write().route = ScreenRoute::Search;
                        },
                        span { "Search" }
                        small { "/" }
                    }
                    button {
                        class: if route == ScreenRoute::Import { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            let mut next = state.write();
                            next.route = ScreenRoute::Import;
                            next.import.open = true;
                        },
                        span { "Import" }
                        small { "Local file" }
                    }
                    button {
                        class: if route == ScreenRoute::Settings { "nav-link active" } else { "nav-link" },
                        onclick: move |_| {
                            state.write().route = ScreenRoute::Settings;
                        },
                        span { "Settings" }
                        small { "Local" }
                    }
                    if total == 0 {
                        button {
                            class: if route == ScreenRoute::Onboarding { "nav-link active" } else { "nav-link" },
                            onclick: move |_| {
                                state.write().route = ScreenRoute::Onboarding;
                            },
                            span { "Onboarding" }
                            small { "First run" }
                        }
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
                            onclick: move |_| state.write().layout = LayoutMode::Issue,
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
                                    state.write().route = ScreenRoute::Search;
                                    reload_library(&search_query_key_services, &mut state);
                                }
                            }
                        }
                        input {
                            class: "search-input secondary",
                            r#type: "text",
                            value: "{filters.author_query}",
                            placeholder: "Filter by author handle",
                            oninput: move |event| state.write().filters.author_query = event.value(),
                            onkeydown: move |event| {
                                if event.key() == Key::Enter {
                                    state.write().route = ScreenRoute::Search;
                                    reload_library(&search_author_key_services, &mut state);
                                }
                            }
                        }
                        button {
                            class: "ghost-button",
                            onclick: move |_| {
                                state.write().route = ScreenRoute::Search;
                                reload_library(&search_submit_services, &mut state);
                            },
                            "Search"
                        }
                    }
                    div {
                        class: "filter-row",
                        input {
                            class: "date-input",
                            r#type: "date",
                            value: "{filters.from_date}",
                            oninput: move |event| state.write().filters.from_date = event.value(),
                        }
                        input {
                            class: "date-input",
                            r#type: "date",
                            value: "{filters.to_date}",
                            oninput: move |event| state.write().filters.to_date = event.value(),
                        }
                        button {
                            class: if filters.has_media_only { "subtle-chip active" } else { "subtle-chip" },
                            onclick: move |_| {
                                {
                                    let mut next = state.write();
                                    next.filters.has_media_only = !next.filters.has_media_only;
                                    next.error = None;
                                }
                                reload_library(&media_toggle_services, &mut state);
                            },
                            "Has media"
                        }
                        if active_filter_count > 0 {
                            span { class: "filter-summary", "{active_filter_count} active filters" }
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
                        for candidate in [LayoutMode::Issue, LayoutMode::FrontPage, LayoutMode::LongRead, LayoutMode::Spread] {
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
                    tabindex: "0",
                    onkeydown: move |event| {
                        let key = event.key();
                        let mut next = state.write();
                        match key {
                            Key::ArrowDown => move_expanded_bookmark(&mut next, 1),
                            Key::ArrowUp => move_expanded_bookmark(&mut next, -1),
                            Key::Character(value) if value.eq_ignore_ascii_case("j") => {
                                move_expanded_bookmark(&mut next, 1);
                            }
                            Key::Character(value) if value.eq_ignore_ascii_case("k") => {
                                move_expanded_bookmark(&mut next, -1);
                            }
                            Key::Escape => clear_expanded_bookmark(&mut next),
                            Key::Character(value) if value == "/" => {
                                next.status = "Use the search field above, then press Enter to search.".to_string();
                            }
                            _ => {}
                        }
                    },
                    div {
                        class: "library-header",
                        div {
                            p { class: "eyebrow", "Reading feed" }
                            h3 { class: "section-title", "{section_title}" }
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

                    if route == ScreenRoute::Authors {
                        div { class: "directory-list",
                            for author in authors.clone() {
                                button {
                                    class: "directory-row",
                                    onclick: {
                                        let directory_services = services.clone();
                                        move |_| {
                                            {
                                                let mut next = state.write();
                                                next.route = ScreenRoute::Author(author.handle.clone());
                                                next.filters.author_query = author.handle.clone();
                                                next.filters.selected_tag = None;
                                                next.filters.favorites_only = false;
                                                next.error = None;
                                            }
                                            reload_library(&directory_services, &mut state);
                                        }
                                    },
                                    span { class: "directory-title", "{author.name}" }
                                    small { "@{author.handle} · {author.bookmark_count} entries · {author.favorite_count} ★" }
                                }
                            }
                        }
                    } else if route == ScreenRoute::Topics {
                        div { class: "topic-cloud",
                            for topic in topics.clone() {
                                button {
                                    class: "topic-token",
                                    onclick: {
                                        let directory_services = services.clone();
                                        move |_| {
                                            {
                                                let mut next = state.write();
                                                next.route = ScreenRoute::Topic(topic.tag.clone());
                                                next.filters.selected_tag = Some(topic.tag.clone());
                                                next.filters.author_query.clear();
                                                next.filters.favorites_only = false;
                                                next.error = None;
                                            }
                                            reload_library(&directory_services, &mut state);
                                        }
                                    },
                                    "#{topic.tag}"
                                    sup { "{topic.bookmark_count}" }
                                }
                            }
                        }
                    } else if let Some(bookmark) = detail_bookmark {
                        article { class: "detail-screen",
                            p { class: "eyebrow", "From the archive" }
                            h3 { "@{bookmark.author_handle}" }
                            p { class: "detail-content", "{bookmark.content}" }
                            if let Some(note) = &bookmark.note_text {
                                blockquote { "{note}" }
                            }
                            div { class: "tag-list",
                                for tag in &bookmark.tags { span { class: "mini-tag", "#{tag}" } }
                            }
                            MediaGallery {
                                media: bookmark.media.clone(),
                                remote_images_enabled,
                                on_enable_remote_images: move |_| set_remote_images_enabled(&mut state, true),
                                context_label: format!("@{} tweet", bookmark.author_handle),
                                mode: MediaGalleryMode::Detail,
                            }
                            button {
                                class: "ghost-button",
                                onclick: move |_| state.write().route = ScreenRoute::Library,
                                "← Back to library"
                            }
                        }
                    } else if route == ScreenRoute::Onboarding {
                        div { class: "onboarding-screen",
                            p { class: "eyebrow", "The room is empty" }
                            h3 { "Welcome." }
                            p { class: "muted-copy onboarding-copy", "Eterea is a local-first reading room for bookmarks from X. Nothing leaves your machine: export your archive, preview it here, then read in Issue, Front Page, Long-Read, or Spread mode." }
                            div { class: "onboarding-steps",
                                article {
                                    strong { "I." }
                                    h4 { "Export from X" }
                                    p { "Settings → Your account → Download an archive. The bookmarks.js file is the primary target." }
                                }
                                article {
                                    strong { "II." }
                                    h4 { "Preview locally" }
                                    p { "CSV, JSON, and X archive JS are parsed before anything is written to SQLite." }
                                }
                                article {
                                    strong { "III." }
                                    h4 { "Read quietly" }
                                    p { "Search with /, navigate cards with j/k, and keep useful entries in Favorites." }
                                }
                            }
                            div { class: "onboarding-actions",
                                button {
                                    class: "accent-button",
                                    onclick: move |_| {
                                        let mut next = state.write();
                                        next.route = ScreenRoute::Import;
                                        next.import.open = true;
                                    },
                                    "Begin import"
                                }
                                button {
                                    class: "ghost-button",
                                    onclick: move |_| state.write().route = ScreenRoute::Library,
                                    "Browse empty library"
                                }
                            }
                            p { class: "muted-copy tiny", "Local-first · no telemetry · MIT-licensed · built with Rust, Dioxus, and SQLite." }
                        }
                    } else if route == ScreenRoute::Settings {
                        div { class: "settings-screen",
                            p { class: "eyebrow", "Preferences · v0.1.0" }
                            h3 { "Set the room." }
                            p { class: "muted-copy", "These appearance controls update the current session only. Persistent config is intentionally not claimed until a safe config file workflow is added." }
                            section { class: "settings-section",
                                h4 { "Reading" }
                                div { class: "settings-row",
                                    span { "Default layout" }
                                    strong { "{layout.as_str()}" }
                                    small { "changed from layout chips above" }
                                }
                                div { class: "settings-row",
                                    span { "Paper tone" }
                                    div { class: "settings-options",
                                        for tone in PaperTone::ALL {
                                            button {
                                                class: if appearance.paper_tone == tone { "subtle-chip active" } else { "subtle-chip" },
                                                onclick: move |_| state.write().appearance.paper_tone = tone,
                                                "{tone.label()}"
                                            }
                                        }
                                    }
                                    small { "session only" }
                                }
                                div { class: "settings-row",
                                    span { "Density" }
                                    div { class: "settings-options",
                                        for density in Density::ALL {
                                            button {
                                                class: if appearance.density == density { "subtle-chip active" } else { "subtle-chip" },
                                                onclick: move |_| state.write().appearance.density = density,
                                                "{density.label()}"
                                            }
                                        }
                                    }
                                    small { "session only" }
                                }
                                div { class: "settings-row",
                                    span { "Accent" }
                                    strong { "{appearance.accent}" }
                                    small { "fixed in v0.1.0" }
                                }
                            }
                            section { class: "settings-section",
                                h4 { "Remote media" }
                                div { class: "settings-row media-setting-row",
                                    span { "Tweet images" }
                                    div { class: "settings-options stacked-copy",
                                        div { class: "settings-options",
                                            button {
                                                class: if remote_images_enabled { "subtle-chip active" } else { "subtle-chip" },
                                                onclick: move |_| set_remote_images_enabled(&mut state, true),
                                                "Load"
                                            }
                                            button {
                                                class: if remote_images_enabled { "subtle-chip" } else { "subtle-chip active" },
                                                onclick: move |_| set_remote_images_enabled(&mut state, false),
                                                "Hide"
                                            }
                                        }
                                        p { class: "muted-copy tiny", "Hidden by default. Loading thumbnails fetches stored HTTPS tweet image URLs from the network for this session only; media metadata remains local." }
                                    }
                                    small { if remote_images_enabled { "session load" } else { "default hidden" } }
                                }
                            }
                            section { class: "settings-section",
                                h4 { "Storage and import" }
                                div { class: "settings-row",
                                    span { "Database" }
                                    strong { "Default local SQLite path" }
                                    small { "opened by backend" }
                                }
                                div { class: "settings-row",
                                    span { "Import format" }
                                    strong { "Auto-detect CSV / JSON / JS" }
                                    small { "preview first" }
                                }
                                div { class: "settings-row",
                                    span { "Deduplicate" }
                                    strong { "On" }
                                    small { "tweet URL uniqueness" }
                                }
                            }
                            section { class: "settings-section",
                                h4 { "About" }
                                div { class: "settings-row",
                                    span { "Built with" }
                                    strong { "Rust · Dioxus · SQLite" }
                                    small { "local-first" }
                                }
                                div { class: "settings-row",
                                    span { "Telemetry" }
                                    strong { "None" }
                                    small { "no network sync in this build" }
                                }
                            }
                        }
                    } else if route == ScreenRoute::Search {
                        div { class: "search-screen",
                            div { class: "search-summary",
                                p { class: "eyebrow", "Search the archive" }
                                h4 { "{bookmarks.len()} visible results · {total} total matches" }
                                p { class: "muted-copy", "Search composes text, author, tag, date, favorites, and media filters. Results stay paginated for large local archives." }
                            }
                            div { class: "search-scope-row",
                                span { class: "subtle-chip active", "All" }
                                span { class: "subtle-chip", "Content" }
                                span { class: "subtle-chip", "Tags" }
                                span { class: "subtle-chip", "Authors" }
                                span { class: "subtle-chip", "Notes" }
                            }
                            if bookmarks.is_empty() {
                                div { class: "empty-card",
                                    p { class: "eyebrow", "No results" }
                                    h4 { "Nothing matched the current filters." }
                                    p { class: "muted-copy", "Try a broader phrase, clear the author/date filters, or reset the view." }
                                }
                            } else {
                                div { class: "search-result-list",
                                    for bookmark in bookmarks.clone() {
                                        article { class: "search-result",
                                            div {
                                                span { class: "eyebrow", "@{bookmark.author_handle}" }
                                                h4 { "{bookmark.author_name}" }
                                            }
                                            div { class: "search-result-body",
                                                p { "{bookmark.content}" }
                                                MediaGallery {
                                                    media: bookmark.media.clone(),
                                                    remote_images_enabled,
                                                    on_enable_remote_images: move |_| set_remote_images_enabled(&mut state, true),
                                                    context_label: format!("@{} tweet", bookmark.author_handle),
                                                    mode: MediaGalleryMode::Compact,
                                                }
                                            }
                                            div { class: "tag-list",
                                                for tag in &bookmark.tags { span { class: "mini-tag", "#{tag}" } }
                                            }
                                        }
                                    }
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
                    } else if route == ScreenRoute::Import {
                        div { class: "import-screen-hint",
                            p { class: "eyebrow", "Local import" }
                            h4 { "Preview an export before it enters the archive." }
                            p { class: "muted-copy", "The import dialog accepts CSV, JSON, and X archive JS files. Preview is a dry parse; the final write is transactional and skips duplicate tweet URLs." }
                            button {
                                class: "accent-button",
                                onclick: move |_| state.write().import.open = true,
                                "Open import dialog"
                            }
                        }
                    } else if let Some(error) = error_message {
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
                    p { class: "muted-copy", "Paste a local path to a CSV, JSON, or X archive JS export. Preview runs as a dry parse first; the final write uses the Rust parser and local SQLite transaction." }
                    div { class: "import-steps",
                        for step in [ImportStage::Source, ImportStage::Preview, ImportStage::Importing, ImportStage::Done] {
                            span {
                                class: if step == import_state.stage { "import-step active" } else { "import-step" },
                                "{step.as_str()}"
                            }
                        }
                    }
                    label {
                        class: "picker-button",
                        "Choose a file"
                        input {
                            key: "{import_state.picker_key}",
                            class: "hidden-file-input",
                            r#type: "file",
                            accept: ".csv,.json,.js",
                            onchange: move |event| {
                                if let Some(file) = event.files().into_iter().next() {
                                    let path = file.path().display().to_string();
                                    let mut next = state.write();
                                    set_import_source(
                                        &mut next.import,
                                        path,
                                        Some("Selected file from native picker.".to_string()),
                                    );
                                    next.import.picker_key = next.import.picker_key.wrapping_add(1);
                                }
                            }
                        }
                    }
                    input {
                        class: "path-input",
                        r#type: "text",
                        value: "{import_state.path}",
                        placeholder: "/home/you/Downloads/bookmarks.json",
                        oninput: move |event| {
                            let mut next = state.write();
                            set_import_source(&mut next.import, event.value(), None);
                        },
                    }
                    if let Some(preview) = &import_state.preview {
                        div { class: "preview-card",
                            div { class: "preview-metrics",
                                div { class: "detail-metric", span { "Format" } strong { "{preview.format}" } }
                                div { class: "detail-metric", span { "Detected" } strong { "{preview.bookmark_count}" } }
                                div { class: "detail-metric", span { "Source" } strong { "{preview.source_label}" } }
                            }
                            p { class: "muted-copy tiny", "{preview.duplicate_policy}" }
                            if !preview.sample.is_empty() {
                                div { class: "preview-list",
                                    for item in preview.sample.clone() {
                                        article { class: "preview-row",
                                            strong { "@{item.author_handle}" }
                                            p { "{item.content}" }
                                            small { "{item.tag_count} tags" }
                                            if item.has_media {
                                                small { "media" }
                                            } else {
                                                small { "text" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
                            class: "ghost-button",
                            onclick: move |_| {
                                let path = PathBuf::from(state.read().import.path.trim());
                                if path.as_os_str().is_empty() {
                                    state.write().import.error = Some("Enter a file path before importing.".to_string());
                                    return;
                                }
                                match import_preview_services.borrow().preview_import_file(&path) {
                                    Ok(preview) => {
                                        let mut next = state.write();
                                        apply_import_preview(&mut next.import, preview);
                                        next.status = "Import preview ready.".to_string();
                                    }
                                    Err(error) => {
                                        let mut next = state.write();
                                        apply_import_error(&mut next.import, error.to_string());
                                    }
                                }
                            },
                            "Preview"
                        }
                        button {
                            class: "accent-button",
                            onclick: move |_| {
                                let path = PathBuf::from(state.read().import.path.trim());
                                if path.as_os_str().is_empty() {
                                    state.write().import.error = Some("Enter a file path before importing.".to_string());
                                    return;
                                }
                                {
                                    let mut next = state.write();
                                    mark_importing(&mut next.import);
                                    next.status = "Importing bookmarks…".to_string();
                                }
                                match import_commit_services.borrow().import_file(&path) {
                                    Ok(imported) => {
                                        {
                                            let mut next = state.write();
                                            apply_import_success(&mut next.import, &path, imported);
                                            next.route = ScreenRoute::Library;
                                            next.status = format!("Imported {imported} bookmarks.");
                                        }
                                        reload_library(&import_commit_services, &mut state);
                                    }
                                    Err(error) => {
                                        let mut next = state.write();
                                        apply_import_error(&mut next.import, error.to_string());
                                    }
                                }
                            },
                            "{import_button_label}"
                        }
                    }
                    p { class: "muted-copy tiny", "Direct X sync remains deferred; import stays local-first and reversible by deleting imported rows." }
                }
            }
        }

        footer { class: "status-bar", "{status_message}" }
    }
}
