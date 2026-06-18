use super::super::state::{Filters, LibraryState};
use chrono::{Local, LocalResult, NaiveDate, TimeZone, Utc};
use eterea_app::BookmarkQuery;

pub(super) fn build_query(state: &LibraryState, offset: usize) -> BookmarkQuery {
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

fn query_or_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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
    use chrono::{DateTime, Timelike};

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
}
