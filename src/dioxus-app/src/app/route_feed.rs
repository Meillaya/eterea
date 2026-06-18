use super::actions::{
    load_more, reload_library, set_remote_images_enabled, toggle_expanded_bookmark,
};
use super::components::BookmarkCard;
use super::route::ScreenRoute;
use super::state::{LayoutMode, LibraryState, Services};
use dioxus::prelude::*;
use eterea_core::Bookmark;

pub(crate) fn archive_feed_or_empty(
    mut state: Signal<LibraryState>,
    services: Services,
) -> Element {
    let payload = feed_payload(&state);
    if let Some(error) = payload.error_message {
        return rsx! {
            div { class: "error-card", strong { "Couldn’t load the archive." } p { "{error}" } }
        };
    }

    if payload.bookmarks.is_empty() {
        return rsx! {
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
        };
    }

    let load_more_services = services.clone();
    rsx! {
        div { class: "bookmark-feed {payload.layout.class_name()}",
            for bookmark in payload.bookmarks {
                {bookmark_card(
                    state,
                    services.clone(),
                    bookmark,
                    payload.remote_images_enabled,
                    payload.expanded_bookmark_id.as_deref(),
                )}
            }
        }
        if payload.has_more {
            button {
                class: "ghost-button wide load-more",
                onclick: move |_| load_more(&load_more_services, &mut state),
                "Load more"
            }
        }
    }
}

struct FeedPayload {
    bookmarks: Vec<Bookmark>,
    layout: LayoutMode,
    has_more: bool,
    error_message: Option<String>,
    remote_images_enabled: bool,
    expanded_bookmark_id: Option<String>,
}

fn feed_payload(state: &Signal<LibraryState>) -> FeedPayload {
    let snapshot = state.read();
    FeedPayload {
        bookmarks: snapshot.bookmarks.clone(),
        layout: snapshot.layout.clone(),
        has_more: snapshot.has_more,
        error_message: snapshot.error.clone(),
        remote_images_enabled: snapshot.remote_images_enabled,
        expanded_bookmark_id: snapshot.expanded_bookmark_id.clone(),
    }
}

fn bookmark_card(
    mut state: Signal<LibraryState>,
    services: Services,
    bookmark: Bookmark,
    remote_images_enabled: bool,
    expanded_bookmark_id: Option<&str>,
) -> Element {
    let toggle_services = services.clone();
    let delete_services = services.clone();
    let author_services = services;
    rsx! {
        BookmarkCard {
            key: "{bookmark.id}",
            expanded: expanded_bookmark_id == Some(bookmark.id.as_str()),
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
                        state.write().error = Some("Bookmark could not be deleted.".to_string());
                    }
                    Err(error) => state.write().error = Some(error.to_string()),
                }
            }
        }
    }
}
