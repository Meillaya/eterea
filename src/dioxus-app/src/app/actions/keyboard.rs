use super::{clear_expanded_bookmark, move_expanded_bookmark, reload_library};
use crate::app::route::{ScreenRoute, TERMINAL_TOP_TABS};
use crate::app::state::{FocusScope, KeyboardMode, LibraryState, Services};
use dioxus::prelude::{Key, ReadableExt, Signal, WritableExt};

pub(crate) fn enter_focus_scope(state: &mut Signal<LibraryState>, scope: FocusScope) {
    let mut next = state.write();
    next.shell.focus_scope = scope;
    next.shell.mode = match scope {
        FocusScope::Shell => next.shell.mode,
        FocusScope::TextInput
            if matches!(
                next.shell.mode,
                KeyboardMode::Command | KeyboardMode::Search
            ) =>
        {
            next.shell.mode
        }
        FocusScope::CommandPalette if next.shell.palette_open => KeyboardMode::Command,
        _ => KeyboardMode::Insert,
    };
}

pub(crate) fn leave_focus_scope(state: &mut Signal<LibraryState>, scope: FocusScope) {
    let mut next = state.write();
    if next.shell.focus_scope == scope {
        next.shell.focus_scope = FocusScope::Shell;
        if next.shell.mode == KeyboardMode::Insert {
            next.shell.mode = KeyboardMode::Normal;
        }
    }
}

pub(crate) fn navigate_terminal_tab(
    services: &Services,
    state: &mut Signal<LibraryState>,
    tab_number: usize,
) -> bool {
    let Some((route_id, _)) = TERMINAL_TOP_TABS.get(tab_number.saturating_sub(1)) else {
        return false;
    };
    let route = ScreenRoute::parse(route_id);
    apply_route_navigation(services, state, route);
    true
}

pub(crate) fn handle_global_keyboard(
    services: &Services,
    state: &mut Signal<LibraryState>,
    key: Key,
    ctrl_or_meta: bool,
) -> bool {
    if state.read().shell.suppresses_global_shortcuts() {
        if key == Key::Escape {
            close_keyboard_surface(state);
            return true;
        }
        return false;
    }

    if is_palette_shortcut(&key, ctrl_or_meta) {
        let mut next = state.write();
        next.shell.palette_open = true;
        next.shell.focus_scope = FocusScope::CommandPalette;
        next.shell.mode = KeyboardMode::Command;
        next.status = "Command palette opened.".to_string();
        return true;
    }

    match key {
        Key::Escape => {
            close_keyboard_surface(state);
            true
        }
        Key::ArrowDown => {
            move_current_bookmark(state, 1);
            true
        }
        Key::ArrowUp => {
            move_current_bookmark(state, -1);
            true
        }
        Key::Character(value) => handle_character_key(services, state, value.as_str()),
        _ => false,
    }
}

fn close_keyboard_surface(state: &mut Signal<LibraryState>) {
    let mut next = state.write();
    next.shell.close_transient_ui();
    clear_expanded_bookmark(&mut next);
    next.status = "Returned to NORMAL mode.".to_string();
}

fn handle_character_key(
    services: &Services,
    state: &mut Signal<LibraryState>,
    value: &str,
) -> bool {
    match value {
        "1" | "2" | "3" | "4" | "5" | "6" => {
            let tab_number = value.parse::<usize>().expect("matched numeric tab key");
            navigate_terminal_tab(services, state, tab_number)
        }
        "j" | "J" => {
            move_current_bookmark(state, 1);
            true
        }
        "k" | "K" => {
            move_current_bookmark(state, -1);
            true
        }
        ":" => {
            let mut next = state.write();
            next.shell.enter_mode(KeyboardMode::Command);
            next.status = "COMMAND mode — type a command in the statusline.".to_string();
            true
        }
        "/" => {
            let mut next = state.write();
            next.shell.enter_mode(KeyboardMode::Search);
            next.shell.command_buffer = next.filters.query.clone();
            next.status = "SEARCH mode — type a query in the statusline.".to_string();
            true
        }
        "?" => {
            let mut next = state.write();
            next.shell.keybindings_open = true;
            next.shell.focus_scope = FocusScope::CommandPalette;
            next.status = "Keybindings overlay opened.".to_string();
            true
        }
        "v" | "V" => {
            let mut next = state.write();
            next.shell.enter_mode(KeyboardMode::Visual);
            next.status = "VISUAL mode — use Space to toggle rows.".to_string();
            true
        }
        "i" | "I" => {
            let mut next = state.write();
            next.shell.enter_mode(KeyboardMode::Insert);
            next.shell.focus_scope = FocusScope::TextInput;
            next.status = "INSERT mode — focused fields own text input.".to_string();
            true
        }
        " " => {
            let mut next = state.write();
            if next.shell.mode != KeyboardMode::Visual {
                return false;
            }
            if let Some(id) = next.expanded_bookmark_id.clone() {
                if !next.shell.selected_bookmark_ids.insert(id.clone()) {
                    next.shell.selected_bookmark_ids.remove(&id);
                }
                next.status = format!("{} selected.", next.shell.selected_bookmark_ids.len());
            }
            true
        }
        "a" => {
            let mut next = state.write();
            if next.shell.mode != KeyboardMode::Visual {
                return false;
            }
            next.shell.selected_bookmark_ids = next
                .bookmarks
                .iter()
                .map(|bookmark| bookmark.id.clone())
                .collect();
            next.status = format!("{} selected.", next.shell.selected_bookmark_ids.len());
            true
        }
        "A" => {
            let mut next = state.write();
            if next.shell.mode != KeyboardMode::Visual {
                return false;
            }
            next.shell.selected_bookmark_ids.clear();
            next.status = "Selection cleared.".to_string();
            true
        }
        _ => false,
    }
}

fn move_current_bookmark(state: &mut Signal<LibraryState>, delta: isize) {
    let mut next = state.write();
    move_expanded_bookmark(&mut next, delta);
    next.status = selection_status(&next);
}

fn selection_status(state: &LibraryState) -> String {
    match state.expanded_bookmark_id.as_ref().and_then(|id| {
        state
            .bookmarks
            .iter()
            .position(|bookmark| &bookmark.id == id)
    }) {
        Some(index) => format!("Selected row {} of {}.", index + 1, state.bookmarks.len()),
        None => "No rows available.".to_string(),
    }
}

fn apply_route_navigation(
    services: &Services,
    state: &mut Signal<LibraryState>,
    route: ScreenRoute,
) {
    let should_reload = {
        let mut next = state.write();
        next.route = route.clone();
        next.shell.close_transient_ui();
        next.error = None;

        match route {
            ScreenRoute::Library => {
                next.filters = Default::default();
                true
            }
            ScreenRoute::Search => {
                next.status = "Search route opened.".to_string();
                false
            }
            ScreenRoute::Import => {
                next.status = "Import route opened.".to_string();
                false
            }
            ScreenRoute::Authors
            | ScreenRoute::Topics
            | ScreenRoute::Settings
            | ScreenRoute::Favorites
            | ScreenRoute::Onboarding
            | ScreenRoute::Entry(_)
            | ScreenRoute::Author(_)
            | ScreenRoute::Topic(_) => false,
        }
    };

    if should_reload {
        reload_library(services, state);
    }
}

fn is_palette_shortcut(key: &Key, ctrl_or_meta: bool) -> bool {
    ctrl_or_meta && matches!(key, Key::Character(value) if value.eq_ignore_ascii_case("p"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::route::ScreenRoute;
    use dioxus::prelude::Key;

    #[test]
    fn palette_shortcut_accepts_ctrl_or_meta_p() {
        assert!(is_palette_shortcut(&Key::Character("p".into()), true));
        assert!(is_palette_shortcut(&Key::Character("P".into()), true));
        assert!(!is_palette_shortcut(&Key::Character("p".into()), false));
        assert!(!is_palette_shortcut(&Key::Character("x".into()), true));
    }

    #[test]
    fn route_navigation_contract_matches_design_top_tabs() {
        let expected_nav_ids = [
            "library", "authors", "topics", "search", "import", "settings",
        ];

        for ((route_id, _), expected_nav_id) in TERMINAL_TOP_TABS.iter().zip(expected_nav_ids) {
            assert_eq!(ScreenRoute::parse(route_id).nav_id(), expected_nav_id);
        }
    }

    #[test]
    fn visual_selection_status_reports_empty_archive() {
        assert_eq!(
            selection_status(&LibraryState::default()),
            "No rows available."
        );
    }
}
