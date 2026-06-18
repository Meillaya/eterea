use super::actions::reload_library;
use super::route::ScreenRoute;
use super::state::{Filters, LibraryState, Services};
use dioxus::prelude::*;

pub(crate) fn left_rail(
    mut state: Signal<LibraryState>,
    services: Services,
    route: ScreenRoute,
    has_active_filters: bool,
    total: i64,
    top_tags: Vec<(String, i64)>,
    filters: Filters,
) -> Element {
    let home_services = services.clone();
    let favorites_services = services.clone();
    let has_top_tags = !top_tags.is_empty();
    let tag_buttons = top_tags.into_iter().take(6).map(|(tag, count)| {
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

    rsx! {
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
                if !has_top_tags {
                    p { class: "muted-copy", "Tags appear here once the archive metadata finishes loading." }
                } else {
                    {tag_buttons}
                }
            }
        }
    }
}
