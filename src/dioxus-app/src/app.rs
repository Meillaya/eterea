mod actions;
mod author_directory;
mod components;
mod design_system;
mod import_modal;
mod onboarding;
mod route;
mod route_content;
mod route_directory;
mod route_feed;
mod screens;
mod search_results;
mod settings;
mod shell;
mod state;

use actions::{handle_global_keyboard, load_initial_state};
use design_system::shell_classes;
use dioxus::prelude::*;
use eterea_app::AppServices;
use state::Services;
use std::{cell::RefCell, rc::Rc};

const APP_CSS: &str = include_str!("../assets/app.css");

#[component]
pub fn App() -> Element {
    let services_result: Result<Services, String> = use_hook(|| {
        AppServices::open_default()
            .map(|services| Rc::new(RefCell::new(services)))
            .map_err(|error| error.to_string())
    });
    let services = match services_result {
        Ok(services) => services,
        Err(error) => {
            return rsx! {
                document::Title { "Eterea — Startup Error" }
                style { "{APP_CSS}" }
                div {
                    class: "app-shell",
                    main {
                        class: "main-column",
                        style: "grid-column: 1 / -1; padding: var(--density-page-pad); justify-content: center;",
                        section {
                            class: "error-card",
                            h1 { "Eterea could not open its archive" }
                            p { "{error}" }
                        }
                    }
                }
            };
        }
    };
    let mut state = use_signal(|| load_initial_state(&services));

    let snapshot = state.read();
    let palette_open = snapshot.shell.palette_open;
    let keybindings_open = snapshot.shell.keybindings_open;
    let shell_class = shell_classes(
        snapshot.appearance.paper_tone,
        snapshot.appearance.density,
        snapshot.appearance.font,
        snapshot.appearance.weight,
        snapshot.appearance.accent_choice,
    );
    let import_open = snapshot.import.open;
    let route = snapshot.route.clone();
    drop(snapshot);

    rsx! {
        document::Title { "Eterea — Dioxus Library" }
        style { "{APP_CSS}" }
        div {
            class: "{shell_class}",
            tabindex: "0",
            autofocus: true,
            onmounted: move |event| async move {
                let _ = event.set_focus(true).await;
            },
            onkeydown: move |event| {
                let key = event.key();
                let modifiers = event.modifiers();
                if handle_global_keyboard(
                    &services,
                    &mut state,
                    key,
                    modifiers.ctrl() || modifiers.meta(),
                ) {
                    event.prevent_default();
                }
            },
            {shell::top_bar(state, services.clone(), route)}
            main {
                class: "main-column terminal-main",
                {route_content::route_content(state, services.clone())}
            }
            {shell::status_line(state, services.clone())}
        }

        if import_open {
            {import_modal::import_modal(state, services.clone())}
        }

        if palette_open {
            {shell::command_palette_overlay(state, services.clone())}
        }

        if keybindings_open {
            {shell::keybindings_overlay(state)}
        }
    }
}
