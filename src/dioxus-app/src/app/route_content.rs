use super::actions::{
    clear_expanded_bookmark, move_expanded_bookmark, reload_library, set_remote_images_enabled,
};
use super::author_directory::{author_directory_status, visible_author_directory};
use super::components::{MediaGallery, MediaGalleryMode};
use super::onboarding::onboarding_screen;
use super::route::ScreenRoute;
use super::route_directory::{authors_directory, topics_cloud, visible_topic_cloud};
use super::route_feed::archive_feed_or_empty;
use super::search_results::search_screen;
use super::settings::settings_screen;
use super::state::{Filters, LibraryState, Services};
use dioxus::prelude::*;

pub(crate) fn route_content(mut state: Signal<LibraryState>, services: Services) -> Element {
    let route = state.read().route.clone();
    let section_title = section_title(&route);
    let (visible_count, total, unique_authors) = route_summary(&state);
    let reset_view_services = services.clone();

    rsx! {
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
                        "Showing {visible_count} of {total} bookmarks • {unique_authors} authors in the archive."
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

            {route_body(state, services, route)}
        }
    }
}

fn route_summary(state: &Signal<LibraryState>) -> (usize, i64, i64) {
    let snapshot = state.read();
    let unique_authors = snapshot
        .stats
        .as_ref()
        .map_or(0, |stats| stats.unique_authors);
    (snapshot.bookmarks.len(), snapshot.total, unique_authors)
}

fn route_body(state: Signal<LibraryState>, services: Services, route: ScreenRoute) -> Element {
    match route {
        ScreenRoute::Authors => {
            let (visible_authors, author_status) = author_directory_payload(&state);
            authors_directory(state, services, visible_authors, author_status)
        }
        ScreenRoute::Topics => {
            let snapshot = state.read();
            let (visible_topics, topics_limited) = visible_topic_cloud(&snapshot.topics);
            let topic_total = snapshot.topics.len();
            let topics = visible_topics.to_vec();
            drop(snapshot);
            topics_cloud(state, services, topics, topic_total, topics_limited)
        }
        ScreenRoute::Entry(id) => detail_or_feed(state, services, &id),
        ScreenRoute::Onboarding => onboarding_screen(state),
        ScreenRoute::Settings => settings_screen(state),
        ScreenRoute::Search => search_payload(state, services),
        ScreenRoute::Import => import_hint(state),
        ScreenRoute::Favorites
        | ScreenRoute::Author(_)
        | ScreenRoute::Topic(_)
        | ScreenRoute::Library => archive_feed_or_empty(state, services),
    }
}

fn author_directory_payload(
    state: &Signal<LibraryState>,
) -> (Vec<eterea_app::AuthorSummary>, String) {
    let snapshot = state.read();
    let (visible_authors, author_has_more) =
        visible_author_directory(&snapshot.authors, &snapshot.filters.author_query);
    let author_status = author_directory_status(
        visible_authors.len(),
        snapshot.authors.len(),
        &snapshot.filters.author_query,
        author_has_more,
    );
    (visible_authors, author_status)
}

fn search_payload(state: Signal<LibraryState>, services: Services) -> Element {
    let snapshot = state.read();
    let bookmarks = snapshot.bookmarks.clone();
    let total = snapshot.total;
    let has_more = snapshot.has_more;
    let remote_images_enabled = snapshot.remote_images_enabled;
    drop(snapshot);
    search_screen(
        state,
        services,
        bookmarks,
        total,
        has_more,
        remote_images_enabled,
    )
}

fn detail_or_feed(mut state: Signal<LibraryState>, services: Services, id: &str) -> Element {
    let snapshot = state.read();
    let bookmark = snapshot
        .bookmarks
        .iter()
        .find(|bookmark| bookmark.id == id)
        .cloned();
    let remote_images_enabled = snapshot.remote_images_enabled;
    drop(snapshot);

    if let Some(bookmark) = bookmark {
        let eterea_core::Bookmark {
            content,
            note_text,
            author_handle,
            tags,
            media,
            ..
        } = bookmark;
        let context_label = format!("@{author_handle} tweet");
        return rsx! {
            article { class: "detail-screen",
                p { class: "eyebrow", "From the archive" }
                h3 { "@{author_handle}" }
                p { class: "detail-content", "{content}" }
                if let Some(note) = &note_text {
                    blockquote { "{note}" }
                }
                div { class: "tag-list",
                    for tag in &tags { span { class: "mini-tag", "#{tag}" } }
                }
                MediaGallery {
                    media,
                    remote_images_enabled,
                    on_enable_remote_images: move |_| set_remote_images_enabled(&mut state, true),
                    context_label,
                    mode: MediaGalleryMode::Detail,
                }
                button {
                    class: "ghost-button",
                    onclick: move |_| state.write().route = ScreenRoute::Library,
                    "← Back to library"
                }
            }
        };
    }

    archive_feed_or_empty(state, services)
}

fn import_hint(mut state: Signal<LibraryState>) -> Element {
    rsx! {
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
    }
}

fn section_title(route: &ScreenRoute) -> String {
    match route {
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
    }
}
