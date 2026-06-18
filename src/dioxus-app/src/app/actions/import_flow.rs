use super::super::state::{ImportStage, ImportState};
use eterea_app::ImportPreview;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;

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
