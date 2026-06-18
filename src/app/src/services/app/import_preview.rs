use crate::types::{ImportPreview, ImportPreviewItem};
use eterea_core::Bookmark;
use std::path::Path;

pub(super) fn build_import_preview(path: &Path, bookmarks: &[Bookmark]) -> ImportPreview {
    let source_label = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("bookmark export")
        .to_string();
    let format = path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
        .to_ascii_uppercase();
    let sample = bookmarks
        .iter()
        .take(5)
        .map(|bookmark| ImportPreviewItem {
            author_handle: bookmark.author_handle.clone(),
            content: preview_content(&bookmark.content),
            tag_count: bookmark.tags.len(),
            has_media: !bookmark.media.is_empty(),
        })
        .collect();

    ImportPreview {
        source_label,
        format,
        bookmark_count: bookmarks.len(),
        sample,
        duplicate_policy:
            "Existing tweet URLs are skipped; a failed parse or write leaves the archive unchanged."
                .to_string(),
    }
}

fn preview_content(content: &str) -> String {
    const LIMIT: usize = 140;
    let mut preview: String = content.chars().take(LIMIT).collect();
    if content.chars().count() > LIMIT {
        preview.push('…');
    }
    preview
}
