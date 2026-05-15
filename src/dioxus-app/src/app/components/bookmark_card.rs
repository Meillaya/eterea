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
    let expand_id = bookmark.id.clone();
    let detail_id = bookmark.id.clone();
    let favorite_id = bookmark.id.clone();
    let delete_id = bookmark.id.clone();
    let tweeted_at = format_timestamp(&bookmark.tweeted_at.to_rfc3339());
    let media_count = bookmark.media.len();
    let favorite_label = if bookmark.is_favorite {
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
                        onclick: move |event| { event.stop_propagation(); on_filter_author.call(bookmark.author_handle.clone()); },
                        "@{bookmark.author_handle}"
                    }
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

            MediaGallery {
                media: bookmark.media.clone(),
                remote_images_enabled,
                on_enable_remote_images,
                context_label: format!("@{} tweet", bookmark.author_handle),
                mode: if expanded { MediaGalleryMode::Detail } else { MediaGalleryMode::Compact },
            }

            if expanded {
                div {
                    class: "inline-detail",
                    div {
                        div { class: "detail-metric", span { "Tweeted" } strong { "{tweeted_at}" } }
                        div { class: "detail-metric", span { "Imported" } strong { "{format_timestamp(&bookmark.imported_at.to_rfc3339())}" } }
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
                    span { "Imported {format_timestamp(&bookmark.imported_at.to_rfc3339())}" }
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
                        if let Err(error) = open_external_url(&bookmark.tweet_url) {
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
