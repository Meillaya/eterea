mod expanded;
mod import_flow;
mod keyboard;
mod query;

pub(crate) use expanded::{
    clear_expanded_bookmark, move_expanded_bookmark, toggle_expanded_bookmark,
};
pub(crate) use import_flow::{
    apply_import_error, apply_import_preview, apply_import_success, mark_importing,
    set_import_source,
};
pub(crate) use keyboard::{
    enter_focus_scope, handle_global_keyboard, leave_focus_scope, navigate_terminal_tab,
};

use super::route::ScreenRoute;
use super::state::{LibraryState, Services};
use chrono::{DateTime, Local, Utc};
use dioxus::prelude::{ReadableExt, Signal, WritableExt};
use query::build_query;

pub(crate) fn load_initial_state(services: &Services) -> LibraryState {
    let mut state = LibraryState::default();
    refresh_from_services(services, &mut state, false);
    if state.total == 0 && state.error.is_none() {
        state.route = ScreenRoute::Onboarding;
    }
    state
}

pub(crate) fn reload_library(services: &Services, state: &mut Signal<LibraryState>) {
    let mut next = state.write();
    refresh_from_services(services, &mut next, false);
}

pub(crate) fn load_more(services: &Services, state: &mut Signal<LibraryState>) {
    let query = {
        let current = state.read();
        build_query(&current, current.bookmarks.len())
    };

    match services.borrow().query_bookmarks(&query) {
        Ok(page) => {
            let mut next = state.write();
            next.bookmarks.extend(page.items);
            next.total = page.total;
            next.has_more = page.has_more;
            next.status = "Loaded more bookmarks.".to_string();
            next.error = None;
        }
        Err(error) => state.write().error = Some(error.to_string()),
    }
}

fn refresh_from_services(services: &Services, state: &mut LibraryState, preserve_status: bool) {
    let query = build_query(state, 0);

    match services.borrow().query_bookmarks(&query) {
        Ok(page) => {
            state.bookmarks = page.items;
            state.total = page.total;
            state.has_more = page.has_more;
            state.error = None;
            if !preserve_status {
                state.status = if state.total == 0 {
                    "Archive is empty — import a bookmark export to begin.".to_string()
                } else {
                    format!("Archive ready — {} bookmarks loaded.", state.total)
                };
            }
        }
        Err(error) => {
            state.error = Some(error.to_string());
            state.bookmarks.clear();
            state.total = 0;
            state.has_more = false;
        }
    }

    match services.borrow().stats() {
        Ok(stats) => {
            state.top_tags = stats.top_tags.clone();
            state.stats = Some(stats);
            state.authors = services.borrow().author_index().unwrap_or_default();
            state.topics = services.borrow().topic_index().unwrap_or_default();
        }
        Err(error) => {
            state.error = Some(error.to_string());
            state.stats = None;
            state.top_tags.clear();
            state.authors.clear();
            state.topics.clear();
        }
    }
}

pub(crate) fn set_remote_images_enabled(state: &mut Signal<LibraryState>, enabled: bool) {
    let mut next = state.write();
    apply_remote_images_enabled(&mut next, enabled);
}

pub(crate) fn load_bookmark_remote_images(state: &mut Signal<LibraryState>, bookmark_id: &str) {
    let mut next = state.write();
    apply_bookmark_remote_images(&mut next, bookmark_id);
}

fn apply_remote_images_enabled(state: &mut LibraryState, enabled: bool) {
    state.remote_images_enabled = enabled;
    if !enabled {
        state.loaded_media_bookmark_ids.clear();
    }
    state.status = if enabled {
        "Remote tweet images enabled for this session."
    } else {
        "Remote tweet images hidden."
    }
    .to_string();
}

fn apply_bookmark_remote_images(state: &mut LibraryState, bookmark_id: &str) {
    state
        .loaded_media_bookmark_ids
        .insert(bookmark_id.to_string());
    state.status = "Remote tweet images enabled for this bookmark.".to_string();
}

pub(crate) fn format_timestamp(value: &DateTime<Utc>) -> String {
    value.with_timezone(&Local).format("%b %-d, %Y").to_string()
}

fn normalize_external_url_for_opening(url: &str) -> std::io::Result<&str> {
    let trimmed = url.trim();
    if trimmed.len() > "https://".len()
        && trimmed
            .get(.."https://".len())
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
    {
        Ok(trimmed)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "external URL opening is limited to HTTPS URLs",
        ))
    }
}

pub(crate) fn open_external_url(url: &str) -> std::io::Result<()> {
    let url = normalize_external_url_for_opening(url)?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported platform for opening external URLs",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_url_policy_rejects_non_https_schemes() {
        assert!(matches!(
            normalize_external_url_for_opening(" https://x.com/alice/status/1 "),
            Ok("https://x.com/alice/status/1")
        ));
        assert!(normalize_external_url_for_opening("http://x.com/alice/status/1").is_err());
        assert!(normalize_external_url_for_opening("javascript:alert(1)").is_err());
        assert!(normalize_external_url_for_opening("file:///tmp/bookmarks.html").is_err());
        assert!(normalize_external_url_for_opening("").is_err());
    }

    #[test]
    fn hiding_remote_images_clears_bookmark_level_media_loads() {
        let mut state = LibraryState::default();

        apply_bookmark_remote_images(&mut state, "bookmark-1");
        apply_remote_images_enabled(&mut state, false);

        assert!(!state.remote_images_enabled);
        assert!(state.loaded_media_bookmark_ids.is_empty());
    }
}
