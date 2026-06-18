use super::actions::{enter_focus_scope, leave_focus_scope, set_remote_images_enabled};
use super::design_system::{AccentChoice, Density, FontChoice, PaperTone, WeightChoice};
use super::state::{FocusScope, LayoutMode, LibraryState};
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
                    div { class: "settings-options",
                        for candidate in LayoutMode::ALL {
                            button {
                                class: if layout == candidate { "subtle-chip active" } else { "subtle-chip" },
                                title: "{candidate.description()}",
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
                                onclick: move |_| state.write().layout = candidate.clone(),
                                "{candidate.as_str()}"
                            }
                        }
                    }
                    small { "session view" }
                }
                div { class: "settings-row",
                    span { "Theme" }
                    div { class: "settings-options",
                        for tone in PaperTone::ALL {
                            button {
                                class: if appearance.paper_tone == tone { "subtle-chip active" } else { "subtle-chip" },
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
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
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
                                onclick: move |_| state.write().appearance.density = density,
                                "{density.label()}"
                            }
                        }
                    }
                    small { "session only" }
                }
                div { class: "settings-row",
                    span { "Font" }
                    div { class: "settings-options",
                        for font in FontChoice::ALL {
                            button {
                                class: if appearance.font == font { "subtle-chip active" } else { "subtle-chip" },
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
                                onclick: move |_| state.write().appearance.font = font,
                                "{font.label()}"
                            }
                        }
                    }
                    small { "session only" }
                }
                div { class: "settings-row",
                    span { "Weight" }
                    div { class: "settings-options",
                        for weight in WeightChoice::ALL {
                            button {
                                class: if appearance.weight == weight { "subtle-chip active" } else { "subtle-chip" },
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
                                onclick: move |_| state.write().appearance.weight = weight,
                                "{weight.label()}"
                            }
                        }
                    }
                    small { "session only" }
                }
                div { class: "settings-row",
                    span { "Accent" }
                    div { class: "settings-options",
                        for accent in AccentChoice::ALL {
                            button {
                                class: if appearance.accent_choice == accent { "subtle-chip active" } else { "subtle-chip" },
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
                                onclick: move |_| {
                                    let mut next = state.write();
                                    next.appearance.accent_choice = accent;
                                    next.appearance.accent = accent.color().to_string();
                                },
                                "{accent.label()}"
                            }
                        }
                    }
                    small { "{appearance.accent}" }
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
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
                                onclick: move |_| set_remote_images_enabled(&mut state, true),
                                "Load"
                            }
                            button {
                                class: if remote_images_enabled { "subtle-chip" } else { "subtle-chip active" },
                                onfocus: move |_| enter_focus_scope(&mut state, FocusScope::SettingsControl),
                                onblur: move |_| leave_focus_scope(&mut state, FocusScope::SettingsControl),
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
