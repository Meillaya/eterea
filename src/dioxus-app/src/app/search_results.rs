use super::actions::{enter_focus_scope, leave_focus_scope, load_more, reload_library};
use super::route::ScreenRoute;
use super::state::{Filters, FocusScope, LibraryState, Services};
use dioxus::prelude::*;
use eterea_core::Bookmark;

pub(crate) fn search_screen(
    mut state: Signal<LibraryState>,
    load_more_services: Services,
    bookmarks: Vec<Bookmark>,
    total: i64,
    has_more: bool,
    _remote_images_enabled: bool,
) -> Element {
    let filters = state.read().filters.clone();
    let apply_query_services = load_more_services.clone();
    let apply_author_services = load_more_services.clone();
    let apply_from_services = load_more_services.clone();
    let apply_to_services = load_more_services.clone();
    let favorites_services = load_more_services.clone();
    let media_services = load_more_services.clone();
    let reset_services = load_more_services.clone();

    rsx! {
        div { class: "search-screen terminal-search",
            div { class: "search-summary",
                p { class: "eyebrow", "search" }
                h4 { "{bookmarks.len()} visible results · {total} total matches" }
                p { class: "muted-copy", "Use / for statusline search or combine author, tag, date, favorites, and media filters from commands/settings. Results stay local and paginated." }
            }
            div { class: "search-scope-row",
                label { class: "terminal-filter-field",
                    span { "query" }
                    input {
                        value: "{filters.query}",
                        placeholder: "content / tags / notes",
                        onfocus: move |_| enter_focus_scope(&mut state, FocusScope::TextInput),
                        onblur: move |_| leave_focus_scope(&mut state, FocusScope::TextInput),
                        oninput: move |event| state.write().filters.query = event.value(),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                state.write().route = ScreenRoute::Search;
                                reload_library(&apply_query_services, &mut state);
                            }
                        },
                    }
                }
                label { class: "terminal-filter-field",
                    span { "author" }
                    input {
                        value: "{filters.author_query}",
                        placeholder: "@handle",
                        onfocus: move |_| enter_focus_scope(&mut state, FocusScope::TextInput),
                        onblur: move |_| leave_focus_scope(&mut state, FocusScope::TextInput),
                        oninput: move |event| state.write().filters.author_query = event.value(),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                state.write().route = ScreenRoute::Search;
                                reload_library(&apply_author_services, &mut state);
                            }
                        },
                    }
                }
                label { class: "terminal-filter-field compact",
                    span { "from" }
                    input {
                        value: "{filters.from_date}",
                        placeholder: "YYYY-MM-DD",
                        onfocus: move |_| enter_focus_scope(&mut state, FocusScope::TextInput),
                        onblur: move |_| leave_focus_scope(&mut state, FocusScope::TextInput),
                        oninput: move |event| state.write().filters.from_date = event.value(),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                state.write().route = ScreenRoute::Search;
                                reload_library(&apply_from_services, &mut state);
                            }
                        },
                    }
                }
                label { class: "terminal-filter-field compact",
                    span { "to" }
                    input {
                        value: "{filters.to_date}",
                        placeholder: "YYYY-MM-DD",
                        onfocus: move |_| enter_focus_scope(&mut state, FocusScope::TextInput),
                        onblur: move |_| leave_focus_scope(&mut state, FocusScope::TextInput),
                        oninput: move |event| state.write().filters.to_date = event.value(),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                state.write().route = ScreenRoute::Search;
                                reload_library(&apply_to_services, &mut state);
                            }
                        },
                    }
                }
                button {
                    class: if filters.favorites_only { "subtle-chip active" } else { "subtle-chip" },
                    onclick: move |_| {
                        {
                            let mut next = state.write();
                            next.filters.favorites_only = !next.filters.favorites_only;
                            next.route = ScreenRoute::Search;
                        }
                        reload_library(&favorites_services, &mut state);
                    },
                    "★ favorites"
                }
                button {
                    class: if filters.has_media_only { "subtle-chip active" } else { "subtle-chip" },
                    onclick: move |_| {
                        {
                            let mut next = state.write();
                            next.filters.has_media_only = !next.filters.has_media_only;
                            next.route = ScreenRoute::Search;
                        }
                        reload_library(&media_services, &mut state);
                    },
                    "media"
                }
                button {
                    class: "subtle-chip",
                    onclick: move |_| {
                        {
                            let mut next = state.write();
                            next.filters = Filters::default();
                            next.route = ScreenRoute::Search;
                        }
                        reload_library(&reset_services, &mut state);
                    },
                    "clear"
                }
            }
            if bookmarks.is_empty() {
                div { class: "empty-card",
                    p { class: "eyebrow", "no results" }
                    h4 { "Nothing matched the current filters." }
                    p { class: "muted-copy", "Try a broader phrase, clear author/date filters, or return to the library table." }
                }
            } else {
                div { class: "search-result-list terminal-result-list",
                    for (index, bookmark) in bookmarks.into_iter().enumerate() {
                        {
                            let id = bookmark.id.clone();
                            let row_index = format!("{:03}", index + 1);
                            let tags = bookmark.tags.iter().map(|tag| format!("#{tag}")).collect::<Vec<_>>().join(" ");
                            rsx! {
                                button {
                                    class: "search-result terminal-result-row",
                                    onclick: move |_| state.write().route = ScreenRoute::Entry(id.clone()),
                                    span { class: "row-index", "{row_index}" }
                                    span { class: "row-author", "@{bookmark.author_handle}" }
                                    span { class: "row-content", "{bookmark.content}" }
                                    span { class: "row-tags", "{tags}" }
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
