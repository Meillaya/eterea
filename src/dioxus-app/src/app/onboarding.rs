use super::actions::{enter_focus_scope, leave_focus_scope};
use super::route::ScreenRoute;
use super::state::{FocusScope, LibraryState};
use dioxus::prelude::*;

pub(crate) fn onboarding_screen(mut state: Signal<LibraryState>) -> Element {
    let ascii = r#"
  ┌─ eterea ───────────────────────────────┐
  │ local archive · terminal library        │
  │ import once, search in milliseconds     │
  └─────────────────────────────────────────┘
"#;

    rsx! {
        div { class: "onboarding-screen terminal-onboarding",
            pre { class: "onboarding-ascii", "{ascii}" }
            p { class: "eyebrow", "first run" }
            h3 { "Bring an export in, and the room becomes searchable." }
            p { class: "muted-copy onboarding-copy", "Eterea is local-first: export from X, preview the archive here, then browse with table/tree/dashboard/graph/calendar views. No telemetry, no remote sync, no production credentials." }
            div { class: "onboarding-steps",
                article {
                    strong { "1." }
                    h4 { "export from x" }
                    p { "Settings → Your account → download an archive." }
                }
                article {
                    strong { "2." }
                    h4 { "preview locally" }
                    p { "CSV, JSON, and bookmarks.js are parsed before anything is written." }
                }
                article {
                    strong { "3." }
                    h4 { "drive by keyboard" }
                    p { "Use 1..6 for tabs, / for search, : for commands, ? for help." }
                }
            }
            div { class: "onboarding-actions terminal-actions",
                button {
                    class: "accent-button",
                    onfocus: move |_| enter_focus_scope(&mut state, FocusScope::OnboardingAction),
                    onblur: move |_| leave_focus_scope(&mut state, FocusScope::OnboardingAction),
                    onclick: move |_| {
                        let mut next = state.write();
                        next.route = ScreenRoute::Import;
                        next.import.open = true;
                    },
                    "begin import"
                }
                button {
                    class: "ghost-button",
                    onfocus: move |_| enter_focus_scope(&mut state, FocusScope::OnboardingAction),
                    onblur: move |_| leave_focus_scope(&mut state, FocusScope::OnboardingAction),
                    onclick: move |_| state.write().route = ScreenRoute::Library,
                    "browse empty library"
                }
            }
            p { class: "muted-copy tiny", "local-first · no telemetry · MIT-licensed · Rust · Dioxus · SQLite" }
        }
    }
}
