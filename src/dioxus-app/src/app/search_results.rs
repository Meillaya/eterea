use super::actions::{load_more, set_remote_images_enabled};
use super::components::{MediaGallery, MediaGalleryMode};
use super::state::{LibraryState, Services};
use dioxus::prelude::*;
use eterea_core::Bookmark;

pub(crate) fn search_screen(
    mut state: Signal<LibraryState>,
    load_more_services: Services,
    bookmarks: Vec<Bookmark>,
    total: i64,
    has_more: bool,
    remote_images_enabled: bool,
) -> Element {
    rsx! {
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
                    for bookmark in bookmarks {
                        {
                            let Bookmark {
                                content,
                                author_handle,
                                author_name,
                                tags,
                                media,
                                ..
                            } = bookmark;
                            let context_label = format!("@{author_handle} tweet");
                            rsx! {
                                article { class: "search-result",
                                    div {
                                        span { class: "eyebrow", "@{author_handle}" }
                                        h4 { "{author_name}" }
                                    }
                                    div { class: "search-result-body",
                                        p { "{content}" }
                                        MediaGallery {
                                            media,
                                            remote_images_enabled,
                                            on_enable_remote_images: move |_| set_remote_images_enabled(&mut state, true),
                                            context_label,
                                            mode: MediaGalleryMode::Compact,
                                        }
                                    }
                                    div { class: "tag-list",
                                        for tag in &tags { span { class: "mini-tag", "#{tag}" } }
                                    }
                                }
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
    }
}
