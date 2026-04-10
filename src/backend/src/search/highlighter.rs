//! Text highlighting for search results

/// Highlight search matches in text
pub fn highlight_matches(
    text: &str,
    query: &str,
    highlight_start: &str,
    highlight_end: &str,
) -> String {
    if query.trim().is_empty() {
        return text.to_string();
    }

    let terms: Vec<String> = query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .map(regex::escape)
        .collect();

    if terms.is_empty() {
        return text.to_string();
    }

    let pattern = format!("(?i){}", terms.join("|"));
    let Ok(regex) = regex::Regex::new(&pattern) else {
        return text.to_string();
    };

    let mut result = String::with_capacity(text.len() + query.len() * 2);
    let mut last_end = 0;

    for matched in regex.find_iter(text) {
        result.push_str(&text[last_end..matched.start()]);
        result.push_str(highlight_start);
        result.push_str(matched.as_str());
        result.push_str(highlight_end);
        last_end = matched.end();
    }

    result.push_str(&text[last_end..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_matches() {
        let text = "Hello world, Rust is great!";
        let result = highlight_matches(text, "rust great", "<mark>", "</mark>");
        assert!(result.contains("<mark>Rust</mark>"));
        assert!(result.contains("<mark>great</mark>"));
    }

    #[test]
    fn test_highlight_empty_query() {
        let text = "Hello world";
        let result = highlight_matches(text, "", "<mark>", "</mark>");
        assert_eq!(result, text);
    }
}
