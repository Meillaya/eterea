//! Search module
//!
//! Provides additional search utilities beyond FTS5.
//! Can be extended with Tantivy for advanced features.

mod highlighter;

pub use highlighter::highlight_matches;

/// Extract search snippets with context
pub fn extract_snippet(text: &str, query: &str, context_chars: usize) -> String {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();
    
    // Find first match
    if let Some(pos) = text_lower.find(&query_lower) {
        let start = pos.saturating_sub(context_chars);
        let end = (pos + query.len() + context_chars).min(text.len());
        
        let mut snippet = String::new();
        if start > 0 {
            snippet.push_str("...");
        }
        snippet.push_str(&text[start..end]);
        if end < text.len() {
            snippet.push_str("...");
        }
        
        snippet
    } else {
        // No match, return beginning of text
        if text.len() > context_chars * 2 {
            format!("{}...", &text[..context_chars * 2])
        } else {
            text.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_snippet() {
        let text = "This is a long piece of text about Rust programming language and its benefits.";
        let snippet = extract_snippet(text, "Rust", 10);
        assert!(snippet.contains("Rust"));
        assert!(snippet.contains("..."));
    }
}

