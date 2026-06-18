mod actions;
mod author_directory;
mod components;
mod design_system;
mod hero_filters;
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

use actions::{count_active_filters, load_initial_state};
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
    let state = use_signal(|| load_initial_state(&services));

    let snapshot = state.read();
    let status_message = snapshot.status.clone();
    let shell_class = shell_classes(snapshot.appearance.paper_tone, snapshot.appearance.density);
    let filters = snapshot.filters.clone();
    let has_active_filters = count_active_filters(&filters) > 0;
    let import_open = snapshot.import.open;
    let route = snapshot.route.clone();
    let total = snapshot.total;
    let top_tags = snapshot.top_tags.clone();
    drop(snapshot);

    rsx! {
        document::Title { "Eterea — Dioxus Library" }
        style { "{APP_CSS}" }
        div {
            class: "{shell_class}",
            {shell::left_rail(state, services.clone(), route, has_active_filters, total, top_tags, filters)}
            main {
                class: "main-column",
                {hero_filters::hero_filters(state, services.clone())}
                {route_content::route_content(state, services.clone())}
            }
        }

        if import_open {
            {import_modal::import_modal(state, services.clone())}
        }

        footer { class: "status-bar", "{status_message}" }
    }
}
