//! Text highlighting for search results

/// Highlight search matches in text
pub fn highlight_matches(text: &str, query: &str, highlight_start: &str, highlight_end: &str) -> String {
    if query.is_empty() {
        return text.to_string();
    }
    
    let text_lower = text.to_lowercase();
    let terms: Vec<&str> = query.split_whitespace().collect();
    
    let mut result = text.to_string();
    
    // Process each term (in reverse order to preserve positions)
    for term in terms {
        let term_lower = term.to_lowercase();
        let mut positions: Vec<usize> = Vec::new();
        
        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&term_lower) {
            positions.push(start + pos);
            start = start + pos + term.len();
        }
        
        // Apply highlights in reverse order
        for pos in positions.into_iter().rev() {
            let end_pos = pos + term.len();
            if end_pos <= result.len() {
                let matched = &result[pos..end_pos];
                let highlighted = format!("{}{}{}", highlight_start, matched, highlight_end);
                result.replace_range(pos..end_pos, &highlighted);
            }
        }
    }
    
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

