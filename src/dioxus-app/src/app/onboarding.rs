use super::route::ScreenRoute;
use super::state::LibraryState;
use dioxus::prelude::*;

pub(crate) fn onboarding_screen(mut state: Signal<LibraryState>) -> Element {
    rsx! {
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
    }
}
