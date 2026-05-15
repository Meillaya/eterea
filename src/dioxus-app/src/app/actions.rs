use super::route::ScreenRoute;
use super::state::{Filters, ImportStage, ImportState, LibraryState, Services};
use chrono::{DateTime, Local, LocalResult, NaiveDate, TimeZone, Utc};
use dioxus::prelude::{ReadableExt, Signal, WritableExt};
use eterea_app::BookmarkQuery;
use eterea_app::ImportPreview;
use std::path::Path;

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

pub(crate) fn toggle_expanded_bookmark(state: &mut LibraryState, id: String) {
    state.expanded_bookmark_id = if state.expanded_bookmark_id.as_deref() == Some(id.as_str()) {
        None
    } else {
        Some(id)
    };
}

pub(crate) fn move_expanded_bookmark(state: &mut LibraryState, delta: isize) {
    if state.bookmarks.is_empty() {
        state.expanded_bookmark_id = None;
        return;
    }

    let current = state.expanded_bookmark_id.as_ref().and_then(|id| {
        state
            .bookmarks
            .iter()
            .position(|bookmark| &bookmark.id == id)
    });
    let base = current.unwrap_or(if delta < 0 {
        state.bookmarks.len()
    } else {
        usize::MAX
    });
    let next = if delta < 0 {
        base.saturating_sub(1)
    } else if base == usize::MAX {
        0
    } else {
        (base + 1).min(state.bookmarks.len() - 1)
    };

    state.expanded_bookmark_id = Some(state.bookmarks[next].id.clone());
}

pub(crate) fn clear_expanded_bookmark(state: &mut LibraryState) {
    state.expanded_bookmark_id = None;
}

pub(crate) fn set_remote_images_enabled(state: &mut Signal<LibraryState>, enabled: bool) {
    let mut next = state.write();
    next.remote_images_enabled = enabled;
    next.status = if enabled {
        "Remote tweet images enabled for this session."
    } else {
        "Remote tweet images hidden."
    }
    .to_string();
}

pub(crate) fn set_import_source(import: &mut ImportState, path: String, message: Option<String>) {
    import.path = path;
    import.stage = ImportStage::Source;
    import.preview = None;
    import.imported_count = None;
    import.error = None;
    import.message = message;
}

pub(crate) fn apply_import_preview(import: &mut ImportState, preview: ImportPreview) {
    import.stage = ImportStage::Preview;
    import.message = Some(format!(
        "Preview ready: {} bookmarks detected in {}.",
        preview.bookmark_count, preview.source_label
    ));
    import.error = None;
    import.preview = Some(preview);
    import.imported_count = None;
}

pub(crate) fn mark_importing(import: &mut ImportState) {
    import.stage = ImportStage::Importing;
    import.error = None;
    import.message = Some("Importing previewed bookmarks into the local archive…".to_string());
}

pub(crate) fn apply_import_success(import: &mut ImportState, path: &Path, imported: usize) {
    import.stage = ImportStage::Done;
    import.error = None;
    import.imported_count = Some(imported);
    import.message = Some(format!(
        "Imported {imported} bookmarks from {}.",
        path.display()
    ));
}

pub(crate) fn apply_import_error(import: &mut ImportState, error: String) {
    import.stage = ImportStage::Source;
    import.error = Some(error);
}

pub(crate) fn format_timestamp(value: &str) -> String {
    DateTime::parse_from_rfc3339(value)
        .map(|parsed| {
            parsed
                .with_timezone(&Local)
                .format("%b %-d, %Y")
                .to_string()
        })
        .unwrap_or_else(|_| value.to_string())
}

fn query_or_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub(crate) fn open_external_url(url: &str) -> std::io::Result<()> {
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

fn build_query(state: &LibraryState, offset: usize) -> BookmarkQuery {
    BookmarkQuery {
        query: query_or_none(&state.filters.query),
        author: query_or_none(&state.filters.author_query),
        from_date: normalize_date_boundary(&state.filters.from_date, false),
        to_date: normalize_date_boundary(&state.filters.to_date, true),
        tag: state.filters.selected_tag.clone(),
        favorites_only: state.filters.favorites_only,
        has_media: state.filters.has_media_only.then_some(true),
        offset,
        limit: state.page_size,
    }
}

fn normalize_date_boundary(value: &str, end_of_day: bool) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let date = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()?;
    let naive = if end_of_day {
        date.and_hms_opt(23, 59, 59)?
    } else {
        date.and_hms_opt(0, 0, 0)?
    };
    let local_boundary = match Local.from_local_datetime(&naive) {
        LocalResult::Single(boundary) => boundary,
        LocalResult::Ambiguous(earliest, latest) => {
            if end_of_day {
                latest
            } else {
                earliest
            }
        }
        LocalResult::None => return None,
    };
    Some(local_boundary.with_timezone(&Utc).to_rfc3339())
}

pub(crate) fn count_active_filters(filters: &Filters) -> usize {
    [
        !filters.query.trim().is_empty(),
        !filters.author_query.trim().is_empty(),
        !filters.from_date.trim().is_empty(),
        !filters.to_date.trim().is_empty(),
        filters.selected_tag.is_some(),
        filters.favorites_only,
        filters.has_media_only,
    ]
    .into_iter()
    .filter(|active| *active)
    .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::state::Filters;
    use chrono::Timelike;

    #[test]
    fn build_query_maps_all_library_filters() {
        let state = LibraryState {
            filters: Filters {
                query: "rust".to_string(),
                author_query: "alice".to_string(),
                from_date: "2024-05-01".to_string(),
                to_date: "2024-05-31".to_string(),
                selected_tag: Some("lang".to_string()),
                favorites_only: true,
                has_media_only: true,
            },
            page_size: 24,
            ..LibraryState::default()
        };

        let query = build_query(&state, 48);
        let from_local = DateTime::parse_from_rfc3339(
            query
                .from_date
                .as_deref()
                .expect("from_date should be present"),
        )
        .expect("from_date should parse")
        .with_timezone(&Local);
        let to_local = DateTime::parse_from_rfc3339(
            query.to_date.as_deref().expect("to_date should be present"),
        )
        .expect("to_date should parse")
        .with_timezone(&Local);

        assert_eq!(query.query.as_deref(), Some("rust"));
        assert_eq!(query.author.as_deref(), Some("alice"));
        assert_eq!(from_local.date_naive().to_string(), "2024-05-01");
        assert_eq!(
            (from_local.hour(), from_local.minute(), from_local.second()),
            (0, 0, 0)
        );
        assert_eq!(to_local.date_naive().to_string(), "2024-05-31");
        assert_eq!(
            (to_local.hour(), to_local.minute(), to_local.second()),
            (23, 59, 59)
        );
        assert_eq!(query.tag.as_deref(), Some("lang"));
        assert!(query.favorites_only);
        assert_eq!(query.has_media, Some(true));
        assert_eq!(query.offset, 48);
        assert_eq!(query.limit, 24);
    }

    #[test]
    fn count_active_filters_includes_new_controls() {
        let filters = Filters {
            query: "rust".to_string(),
            author_query: "alice".to_string(),
            from_date: "2024-05-01".to_string(),
            to_date: "2024-05-31".to_string(),
            selected_tag: Some("lang".to_string()),
            favorites_only: true,
            has_media_only: true,
        };

        assert_eq!(count_active_filters(&filters), 7);
    }

    #[test]
    fn expanded_bookmark_helpers_toggle_move_and_clear() {
        let mut state = LibraryState {
            bookmarks: vec![
                eterea_core::Bookmark::new(
                    "https://x.com/a/status/1".into(),
                    "first".into(),
                    Utc::now(),
                    "a".into(),
                    "A".into(),
                ),
                eterea_core::Bookmark::new(
                    "https://x.com/b/status/2".into(),
                    "second".into(),
                    Utc::now(),
                    "b".into(),
                    "B".into(),
                ),
            ],
            ..LibraryState::default()
        };
        let first = state.bookmarks[0].id.clone();
        let second = state.bookmarks[1].id.clone();

        toggle_expanded_bookmark(&mut state, first.clone());
        assert_eq!(state.expanded_bookmark_id.as_deref(), Some(first.as_str()));

        move_expanded_bookmark(&mut state, 1);
        assert_eq!(state.expanded_bookmark_id.as_deref(), Some(second.as_str()));

        move_expanded_bookmark(&mut state, 1);
        assert_eq!(state.expanded_bookmark_id.as_deref(), Some(second.as_str()));

        move_expanded_bookmark(&mut state, -1);
        assert_eq!(state.expanded_bookmark_id.as_deref(), Some(first.as_str()));

        clear_expanded_bookmark(&mut state);
        assert!(state.expanded_bookmark_id.is_none());
    }

    #[test]
    fn import_state_helpers_drive_source_preview_importing_done_flow() {
        let mut import = ImportState {
            path: "/tmp/old.json".to_string(),
            error: Some("old error".to_string()),
            ..ImportState::default()
        };

        set_import_source(
            &mut import,
            "/tmp/bookmarks.json".to_string(),
            Some("Selected file.".to_string()),
        );
        assert_eq!(import.stage, ImportStage::Source);
        assert_eq!(import.path, "/tmp/bookmarks.json");
        assert!(import.error.is_none());
        assert!(import.preview.is_none());

        apply_import_preview(
            &mut import,
            eterea_app::ImportPreview {
                source_label: "bookmarks.json".to_string(),
                format: "JSON".to_string(),
                bookmark_count: 2,
                sample: Vec::new(),
                duplicate_policy: "duplicates skipped".to_string(),
            },
        );
        assert_eq!(import.stage, ImportStage::Preview);
        assert_eq!(
            import
                .preview
                .as_ref()
                .map(|preview| preview.bookmark_count),
            Some(2)
        );

        mark_importing(&mut import);
        assert_eq!(import.stage, ImportStage::Importing);

        apply_import_success(&mut import, std::path::Path::new("/tmp/bookmarks.json"), 2);
        assert_eq!(import.stage, ImportStage::Done);
        assert_eq!(import.imported_count, Some(2));

        apply_import_error(&mut import, "broken".to_string());
        assert_eq!(import.stage, ImportStage::Source);
        assert_eq!(import.error.as_deref(), Some("broken"));
    }
}
