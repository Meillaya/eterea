use super::actions::{count_active_filters, reload_library};
use super::route::ScreenRoute;
use super::state::{LayoutMode, LibraryState, Services};
use dioxus::prelude::*;

pub(crate) fn hero_filters(mut state: Signal<LibraryState>, services: Services) -> Element {
    let snapshot = state.read();
    let filters = snapshot.filters.clone();
    let layout = snapshot.layout.clone();
    let total = snapshot.total;
    drop(snapshot);
    let active_filter_count = count_active_filters(&filters);
    let search_submit_services = services.clone();
    let search_query_key_services = services.clone();
    let search_author_key_services = services.clone();
    let filter_toggle_services = services.clone();
    let media_toggle_services = services;

    rsx! {
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
    }
}
