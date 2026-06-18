use eterea_app::AuthorSummary;

const AUTHOR_DIRECTORY_INITIAL_LIMIT: usize = 250;
const AUTHOR_DIRECTORY_FILTER_LIMIT: usize = 500;

fn author_matches_filter(author: &AuthorSummary, normalized_filter: &str) -> bool {
    normalized_filter.is_empty()
        || author
            .handle
            .to_ascii_lowercase()
            .contains(normalized_filter)
        || author.name.to_ascii_lowercase().contains(normalized_filter)
}

pub(crate) fn visible_author_directory(
    authors: &[AuthorSummary],
    filter: &str,
) -> (Vec<AuthorSummary>, bool) {
    let normalized_filter = filter.trim().to_ascii_lowercase();
    let limit = if normalized_filter.is_empty() {
        AUTHOR_DIRECTORY_INITIAL_LIMIT
    } else {
        AUTHOR_DIRECTORY_FILTER_LIMIT
    };
    if normalized_filter.is_empty() {
        return (
            authors.iter().take(limit).cloned().collect(),
            authors.len() > limit,
        );
    }

    let mut matched = 0usize;
    let mut visible = Vec::with_capacity(limit.min(authors.len()));
    for author in authors {
        if author_matches_filter(author, &normalized_filter) {
            matched += 1;
            if visible.len() < limit {
                visible.push(author.clone());
            }
        }
    }

    (visible, matched > limit)
}

pub(crate) fn author_directory_status(
    visible_count: usize,
    total_count: usize,
    filter: &str,
    has_more: bool,
) -> String {
    let trimmed_filter = filter.trim();
    if trimmed_filter.is_empty() {
        format!(
            "Showing top {visible_count} of {total_count} authors. Type in the author field above to filter instantly without rendering the entire archive."
        )
    } else if has_more {
        format!("Showing first {visible_count} matching authors for “{trimmed_filter}”. Keep typing to narrow.")
    } else {
        format!("Showing {visible_count} matching authors for “{trimmed_filter}”.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn author(index: usize) -> AuthorSummary {
        AuthorSummary {
            handle: format!("author_{index:03}"),
            name: format!("Author {index:03}"),
            profile_image: None,
            bookmark_count: 1,
            favorite_count: 0,
        }
    }

    #[test]
    fn author_directory_caps_initial_render() {
        let authors = (0..300).map(author).collect::<Vec<_>>();

        let (visible, has_more) = visible_author_directory(&authors, "");

        assert_eq!(visible.len(), AUTHOR_DIRECTORY_INITIAL_LIMIT);
        assert!(has_more);
    }

    #[test]
    fn author_directory_filters_without_initial_cap() {
        let authors = (0..300).map(author).collect::<Vec<_>>();

        let (visible, has_more) = visible_author_directory(&authors, "author_299");

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].handle, "author_299");
        assert!(!has_more);
    }
}
