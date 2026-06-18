use super::actions::set_remote_images_enabled;
use super::design_system::{Density, PaperTone};
use super::state::LibraryState;
use dioxus::prelude::*;

pub(crate) fn settings_screen(mut state: Signal<LibraryState>) -> Element {
    let snapshot = state.read();
    let appearance = snapshot.appearance.clone();
    let layout = snapshot.layout.clone();
    let remote_images_enabled = snapshot.remote_images_enabled;
    drop(snapshot);

    rsx! {
        div { class: "settings-screen",
            p { class: "eyebrow", "Preferences · v0.1.0" }
            h3 { "Set the room." }
            p { class: "muted-copy", "These appearance controls update the current session only. Persistent config is intentionally not claimed until a safe config file workflow is added." }
            section { class: "settings-section",
                h4 { "Reading" }
                div { class: "settings-row",
                    span { "Default layout" }
                    strong { "{layout.as_str()}" }
                    small { "changed from layout chips above" }
                }
                div { class: "settings-row",
                    span { "Paper tone" }
                    div { class: "settings-options",
                        for tone in PaperTone::ALL {
                            button {
                                class: if appearance.paper_tone == tone { "subtle-chip active" } else { "subtle-chip" },
                                onclick: move |_| state.write().appearance.paper_tone = tone,
                                "{tone.label()}"
                            }
                        }
                    }
                    small { "session only" }
                }
                div { class: "settings-row",
                    span { "Density" }
                    div { class: "settings-options",
                        for density in Density::ALL {
                            button {
                                class: if appearance.density == density { "subtle-chip active" } else { "subtle-chip" },
                                onclick: move |_| state.write().appearance.density = density,
                                "{density.label()}"
                            }
                        }
                    }
                    small { "session only" }
                }
                div { class: "settings-row",
                    span { "Accent" }
                    strong { "{appearance.accent}" }
                    small { "fixed in v0.1.0" }
                }
            }
            section { class: "settings-section",
                h4 { "Remote media" }
                div { class: "settings-row media-setting-row",
                    span { "Tweet images" }
                    div { class: "settings-options stacked-copy",
                        div { class: "settings-options",
                            button {
                                class: if remote_images_enabled { "subtle-chip active" } else { "subtle-chip" },
                                onclick: move |_| set_remote_images_enabled(&mut state, true),
                                "Load"
                            }
                            button {
                                class: if remote_images_enabled { "subtle-chip" } else { "subtle-chip active" },
                                onclick: move |_| set_remote_images_enabled(&mut state, false),
                                "Hide"
                            }
                        }
                        p { class: "muted-copy tiny", "Hidden by default. Loading thumbnails fetches stored HTTPS tweet image URLs from the network for this session only; media metadata remains local." }
                    }
                    small { if remote_images_enabled { "session load" } else { "default hidden" } }
                }
            }
            section { class: "settings-section",
                h4 { "Storage and import" }
                div { class: "settings-row",
                    span { "Database" }
                    strong { "Default local SQLite path" }
                    small { "opened by backend" }
                }
                div { class: "settings-row",
                    span { "Import format" }
                    strong { "Auto-detect CSV / JSON / JS" }
                    small { "preview first" }
                }
                div { class: "settings-row",
                    span { "Deduplicate" }
                    strong { "On" }
                    small { "tweet URL uniqueness" }
                }
            }
            section { class: "settings-section",
                h4 { "About" }
                div { class: "settings-row",
                    span { "Built with" }
                    strong { "Rust · Dioxus · SQLite" }
                    small { "local-first" }
                }
                div { class: "settings-row",
                    span { "Telemetry" }
                    strong { "None" }
                    small { "no network sync in this build" }
                }
            }
        }
    }
}
