use crate::app::actions::{format_timestamp, open_external_url};
use crate::app::components::{MediaGallery, MediaGalleryMode};
use dioxus::prelude::*;
use eterea_core::Bookmark;

#[component]
pub(crate) fn BookmarkCard(
    bookmark: Bookmark,
    expanded: bool,
    on_toggle_expand: EventHandler<String>,
    on_open_detail: EventHandler<String>,
    on_filter_author: EventHandler<String>,
    on_toggle_favorite: EventHandler<String>,
    on_delete: EventHandler<String>,
    remote_images_enabled: bool,
    on_enable_remote_images: EventHandler<()>,
) -> Element {
    let Bookmark {
        id,
        tweet_url,
        content,
        note_text,
        tweeted_at,
        imported_at,
        author_handle,
        author_name,
        tags,
        media,
        is_favorite,
        ..
    } = bookmark;
    let expand_id = id.clone();
    let detail_id = id.clone();
    let favorite_id = id.clone();
    let delete_id = id.clone();
    let filter_author = author_handle.clone();
    let context_label = format!("@{author_handle} tweet");
    let tweeted_at = format_timestamp(&tweeted_at);
    let imported_at = format_timestamp(&imported_at);
    let media_count = media.len();
    let favorite_label = if is_favorite {
        "★ Favorited"
    } else {
        "☆ Favorite"
    };

    rsx! {
        article {
            class: if expanded { "bookmark-card panel expanded" } else { "bookmark-card panel" },
            onclick: move |_| on_toggle_expand.call(expand_id.clone()),
            div {
                class: "bookmark-meta",
                div {
                    button {
                        class: "author-button",
                        onclick: move |event| { event.stop_propagation(); on_filter_author.call(filter_author.clone()); },
                        "@{author_handle}"
                    }
                    span { "{author_name}" }
                }
                span { "{tweeted_at}" }
            }
            p { class: "bookmark-content", "{content}" }
            if let Some(note) = &note_text {
                p { class: "bookmark-note", "{note}" }
            }
            if !tags.is_empty() {
                div {
                    class: "tag-list",
                    for tag in &tags {
                        span { class: "mini-tag", "#{tag}" }
                    }
                }
            }

            MediaGallery {
                media,
                remote_images_enabled,
                on_enable_remote_images,
                context_label,
                mode: if expanded { MediaGalleryMode::Detail } else { MediaGalleryMode::Compact },
            }

            if expanded {
                div {
                    class: "inline-detail",
                    div {
                        div { class: "detail-metric", span { "Tweeted" } strong { "{tweeted_at}" } }
                        div { class: "detail-metric", span { "Imported" } strong { "{imported_at}" } }
                        div { class: "detail-metric", span { "Media" } strong { "{media_count}" } }
                    }
                    p { "Click again to collapse · use j/k or arrow keys to move between entries · Esc clears the open entry." }
                }
            }
            div {
                class: "bookmark-footer",
                div {
                    class: "bookmark-stats",
                    span { "{media_count} media" }
                    span { "Imported {imported_at}" }
                }
                button {
                    class: "ghost-button small",
                    onclick: move |event| {
                        event.stop_propagation();
                        on_open_detail.call(detail_id.clone());
                    },
                    "Details"
                }
                button {
                    class: "ghost-button small",
                    onclick: move |event| {
                        event.stop_propagation();
                        if let Err(error) = open_external_url(&tweet_url) {
                            eprintln!("failed to open external URL: {error}");
                        }
                    },
                    "Open"
                }
                button {
                    class: "ghost-button small",
                    onclick: move |event| { event.stop_propagation(); on_toggle_favorite.call(favorite_id.clone()); },
                    "{favorite_label}"
                }
                button {
                    class: "ghost-button small danger-button",
                    onclick: move |event| { event.stop_propagation(); on_delete.call(delete_id.clone()); },
                    "Delete"
                }
            }
        }
    }
}
