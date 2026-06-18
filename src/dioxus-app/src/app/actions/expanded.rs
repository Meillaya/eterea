use super::super::state::LibraryState;

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

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
}
