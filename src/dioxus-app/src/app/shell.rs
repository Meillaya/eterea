use super::actions::{enter_focus_scope, leave_focus_scope, navigate_terminal_tab, reload_library};
use super::route::{ScreenRoute, TERMINAL_TOP_TABS};
use super::state::{FocusScope, KeyboardMode, LibraryState, Services};
use dioxus::prelude::*;

pub(crate) fn top_bar(
    mut state: Signal<LibraryState>,
    services: Services,
    route: ScreenRoute,
) -> Element {
    let tabs = TERMINAL_TOP_TABS
        .iter()
        .enumerate()
        .map(|(index, (route_id, label))| {
            let tab_services = services.clone();
            let is_active = route.nav_id() == *route_id;
            let tab_number = index + 1;
            rsx! {
                button {
                    class: if is_active { "top-tab active" } else { "top-tab" },
                    onclick: move |_| {
                        navigate_terminal_tab(&tab_services, &mut state, tab_number);
                    },
                    span { class: "tab-index", "[{tab_number}]" }
                    "{label}"
                }
            }
        });

    rsx! {
        header { class: "top-bar",
            div { class: "brand-block",
                span { class: "brand-diamond", "◆" }
                span { class: "brand-name", "eterea" }
                span { class: "brand-version", "v0.1.0" }
            }
            nav { class: "top-tabs", {tabs} }
            div { class: "top-hints",
                span { class: "kbd-hint", "?" } " help"
                span { class: "kbd-hint", "Ctrl" } "+" span { class: "kbd-hint", "P" } " palette"
            }
        }
    }
}

pub(crate) fn status_line(mut state: Signal<LibraryState>, services: Services) -> Element {
    let snapshot = state.read();
    let mode = snapshot.shell.mode;
    let mode_label = mode.label();
    let status_message = snapshot.status.clone();
    let command_buffer = snapshot.shell.command_buffer.clone();
    let selected_count = snapshot.shell.selected_bookmark_ids.len();
    let total = snapshot.total;
    let current_index = snapshot
        .expanded_bookmark_id
        .as_ref()
        .and_then(|id| {
            snapshot
                .bookmarks
                .iter()
                .position(|bookmark| &bookmark.id == id)
        })
        .map_or(0, |index| index + 1);
    let theme = snapshot.appearance.paper_tone.label().to_lowercase();
    drop(snapshot);

    if matches!(mode, KeyboardMode::Command | KeyboardMode::Search) {
        let prompt = if mode == KeyboardMode::Command {
            ":"
        } else {
            "/"
        };
        let status_class = if mode == KeyboardMode::Command {
            "status-line command-status-line"
        } else {
            "status-line search-status-line"
        };
        let input_scope = FocusScope::TextInput;
        let submit_services = services.clone();
        return rsx! {
            footer { class: "{status_class}",
                span { class: "mode-block", "{mode_label}" }
                span { class: "status-prompt", "{prompt}" }
                input {
                    class: "status-command-input",
                    value: "{command_buffer}",
                    autofocus: true,
                    onfocus: move |_| enter_focus_scope(&mut state, input_scope),
                    onblur: move |_| leave_focus_scope(&mut state, input_scope),
                    oninput: move |event| state.write().shell.command_buffer = event.value(),
                    onkeydown: move |event| {
                        match event.key() {
                            Key::Enter => submit_statusline(&submit_services, &mut state),
                            Key::Escape => state.write().shell.close_transient_ui(),
                            _ => {}
                        }
                    },
                }
            }
        };
    }

    rsx! {
        footer { class: "status-line",
            span { class: "mode-block", "{mode_label}" }
            if selected_count > 0 {
                span { class: "selected-block", "{selected_count} selected" }
            }
            span { class: "status-left", "{status_message}" }
            span { class: "status-right", "{current_index}/{total} · {theme} · Ln 1 Col 1" }
        }
    }
}

pub(crate) fn command_palette_overlay(
    mut state: Signal<LibraryState>,
    services: Services,
) -> Element {
    let palette_query = state.read().shell.palette_query.clone();
    let normalized_query = palette_query.trim().to_lowercase();
    let commands = [
        ("library", "1", "Open library", "library table"),
        ("authors", "2", "Browse authors", "directory handles voices"),
        ("topics", "3", "Browse topics", "tags cloud graph"),
        ("search", "4", "Search archive", "query filter fts"),
        ("import", "5", "Import bookmarks", "csv json archive"),
        ("settings", "6", "Open settings", "theme density media"),
    ];
    let matches = commands
        .into_iter()
        .filter(|(label, key, description, keywords)| {
            normalized_query.is_empty()
                || [*label, *key, *description, *keywords]
                    .iter()
                    .any(|value| value.to_lowercase().contains(&normalized_query))
        })
        .collect::<Vec<_>>();
    let first_command = matches.first().map(|(label, _, _, _)| (*label).to_string());
    let match_count = matches.len();
    let enter_services = services.clone();

    rsx! {
        div {
            class: "overlay-backdrop command-palette-backdrop",
            onclick: move |_| state.write().shell.close_transient_ui(),
            div {
                class: "palette-panel",
                onclick: move |event| event.stop_propagation(),
                div { class: "palette-input-row",
                    span { class: "palette-caret", "❯" }
                    input {
                        class: "palette-input",
                        autofocus: true,
                        placeholder: "Type a command, file, or query...",
                        value: "{palette_query}",
                        onfocus: move |_| enter_focus_scope(&mut state, FocusScope::CommandPalette),
                        onblur: move |_| leave_focus_scope(&mut state, FocusScope::CommandPalette),
                        oninput: move |event| state.write().shell.palette_query = event.value(),
                        onkeydown: move |event| {
                            match event.key() {
                                Key::Enter => {
                                    if let Some(command) = first_command.as_deref() {
                                        execute_palette_command(&enter_services, &mut state, command);
                                    }
                                }
                                Key::Escape => state.write().shell.close_transient_ui(),
                                _ => {}
                            }
                        },
                    }
                    span { class: "palette-count", "{match_count} matches" }
                }
                div { class: "palette-results",
                    for (label, key, description, keywords) in matches {
                        {
                            let command = label.to_string();
                            let row_services = services.clone();
                            rsx! {
                        button {
                            class: "palette-row",
                            onclick: move |_| execute_palette_command(&row_services, &mut state, &command),
                            span { class: "palette-icon", "›" }
                            span { class: "palette-label", "{label}" }
                            span { class: "palette-sub", "{description}" }
                            span { class: "palette-keywords", "{keywords}" }
                            span { class: "kbd-hint", "{key}" }
                        }
                            }
                        }
                    }
                }
                div { class: "palette-footer",
                    span { class: "kbd-hint", "↵" } " open"
                    span { class: "kbd-hint", "type" } " filter"
                    span { class: "kbd-hint", "Esc" } " close"
                }
            }
        }
    }
}

pub(crate) fn keybindings_overlay(mut state: Signal<LibraryState>) -> Element {
    let groups = [
        (
            "Movement",
            [
                ("j / ↓", "next entry"),
                ("k / ↑", "prev entry"),
                ("Esc", "normal / close"),
            ],
        ),
        (
            "Modes",
            [
                (":", "command mode"),
                ("/", "search mode"),
                ("v", "visual multi-select"),
            ],
        ),
        (
            "Navigation",
            [
                ("1..6", "jump tabs"),
                ("Ctrl-P", "command palette"),
                ("?", "this overlay"),
            ],
        ),
        (
            "Selection",
            [
                ("Space", "toggle selected"),
                ("a", "select all"),
                ("A", "deselect all"),
            ],
        ),
    ];

    rsx! {
        div {
            class: "overlay-backdrop keybindings-backdrop",
            onclick: move |_| state.write().shell.close_transient_ui(),
            div {
                class: "keybindings-panel",
                onclick: move |event| event.stop_propagation(),
                div { class: "keybindings-header",
                    p { class: "eyebrow", "Keyboard" }
                    button { class: "ghost-button small", onclick: move |_| state.write().shell.close_transient_ui(), "Esc" }
                }
                div { class: "keybindings-grid",
                    for (group, rows) in groups {
                        section { class: "keybinding-group",
                            h4 { "{group}" }
                            for (key, description) in rows {
                                div { class: "keybinding-row",
                                    span { class: "kbd-hint", "{key}" }
                                    span { "{description}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn submit_statusline(services: &Services, state: &mut Signal<LibraryState>) {
    let snapshot = state.read();
    let mode = snapshot.shell.mode;
    let raw_input = snapshot.shell.command_buffer.trim().to_string();
    drop(snapshot);

    if mode == KeyboardMode::Search {
        apply_search_query(services, state, raw_input);
        return;
    }

    let command = raw_input.to_lowercase();
    let route = match command.as_str() {
        "1" | "library" | "open library" => Some(ScreenRoute::Library),
        "2" | "authors" | "open authors" => Some(ScreenRoute::Authors),
        "3" | "topics" | "tags" => Some(ScreenRoute::Topics),
        "4" | "search" => Some(ScreenRoute::Search),
        "5" | "import" => Some(ScreenRoute::Import),
        "6" | "settings" => Some(ScreenRoute::Settings),
        "fav" | "favs" | "favorites" | "only favorites" => {
            apply_filter_command(services, state, |next| {
                next.filters.favorites_only = true;
                next.route = ScreenRoute::Favorites;
                next.status = "Favorites filter applied.".to_string();
            });
            return;
        }
        "media" | "has media" | "only media" => {
            apply_filter_command(services, state, |next| {
                next.filters.has_media_only = true;
                next.route = ScreenRoute::Search;
                next.status = "Media-only filter applied.".to_string();
            });
            return;
        }
        "clear" | "reset" | "reset filters" => {
            apply_filter_command(services, state, |next| {
                next.filters = Default::default();
                next.route = ScreenRoute::Library;
                next.status = "Filters cleared.".to_string();
            });
            return;
        }
        _ => None,
    };

    if let Some(value) = command_argument(&raw_input, "author") {
        let author = value.trim().trim_start_matches('@').to_string();
        apply_filter_command(services, state, |next| {
            next.filters.author_query = author.clone();
            next.route = ScreenRoute::Author(author.clone());
            next.status = format!("Author filter applied: @{author}.");
        });
        return;
    }

    if let Some(value) = command_argument(&raw_input, "tag") {
        let tag = value.trim().trim_start_matches('#').to_string();
        apply_filter_command(services, state, |next| {
            next.filters.selected_tag = Some(tag.clone());
            next.route = ScreenRoute::Topic(tag.clone());
            next.status = format!("Topic filter applied: #{tag}.");
        });
        return;
    }

    if let Some(value) = command_argument(&raw_input, "from") {
        let from_date = value.trim().to_string();
        apply_filter_command(services, state, |next| {
            next.filters.from_date = from_date.clone();
            next.route = ScreenRoute::Search;
            next.status = format!("From-date filter applied: {from_date}.");
        });
        return;
    }

    if let Some(value) = command_argument(&raw_input, "to") {
        let to_date = value.trim().to_string();
        apply_filter_command(services, state, |next| {
            next.filters.to_date = to_date.clone();
            next.route = ScreenRoute::Search;
            next.status = format!("To-date filter applied: {to_date}.");
        });
        return;
    }

    if let Some(route) = route {
        let should_reload = route == ScreenRoute::Library;
        {
            let mut next = state.write();
            next.route = route.clone();
            next.shell.close_transient_ui();
            next.status = format!("Opened {}.", route.nav_id());
            if should_reload {
                next.filters = Default::default();
            }
        }
        if should_reload {
            reload_library(services, state);
        }
    } else {
        let mut next = state.write();
        next.status = if command.is_empty() {
            "No command entered.".to_string()
        } else {
            format!("Unknown command: {command}")
        };
        next.shell.close_transient_ui();
    }
}

fn execute_palette_command(services: &Services, state: &mut Signal<LibraryState>, command: &str) {
    {
        let mut next = state.write();
        next.shell.command_buffer = command.to_string();
        next.shell.mode = KeyboardMode::Command;
    }
    submit_statusline(services, state);
}

fn command_argument<'a>(raw_input: &'a str, verb: &str) -> Option<&'a str> {
    let trimmed = raw_input.trim();
    let separator = trimmed.find(char::is_whitespace)?;
    let (candidate, argument) = trimmed.split_at(separator);
    candidate
        .eq_ignore_ascii_case(verb)
        .then_some(argument.trim_start())
}

fn apply_search_query(services: &Services, state: &mut Signal<LibraryState>, query: String) {
    apply_filter_command(services, state, |next| {
        next.filters.query = query.clone();
        next.route = ScreenRoute::Search;
        next.status = if query.trim().is_empty() {
            "Search cleared.".to_string()
        } else {
            format!("Searching local archive for “{}”.", query.trim())
        };
    });
}

fn apply_filter_command(
    services: &Services,
    state: &mut Signal<LibraryState>,
    apply: impl FnOnce(&mut LibraryState),
) {
    {
        let mut next = state.write();
        apply(&mut next);
        next.error = None;
        next.shell.close_transient_ui();
    }
    reload_library(services, state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_argument_preserves_filter_value_case() {
        assert_eq!(
            command_argument("author @CamelCase", "author"),
            Some("@CamelCase")
        );
        assert_eq!(command_argument("TAG #RustLang", "tag"), Some("#RustLang"));
        assert_eq!(
            command_argument("from 2026-06-18", "from"),
            Some("2026-06-18")
        );
        assert_eq!(command_argument("topic #RustLang", "tag"), None);
    }
}
