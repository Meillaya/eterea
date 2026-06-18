use super::actions::{
    apply_import_error, apply_import_preview, apply_import_success, enter_focus_scope,
    leave_focus_scope, mark_importing, reload_library, set_import_source,
};
use super::route::ScreenRoute;
use super::state::{FocusScope, ImportStage, LibraryState, Services};
use dioxus::prelude::*;
use std::path::PathBuf;

pub(crate) fn import_modal(mut state: Signal<LibraryState>, services: Services) -> Element {
    let import_state = state.read().import.clone();
    let import_button_label = if import_state.preview.is_some() {
        "Import preview"
    } else {
        "Import without preview"
    };
    let import_preview_services = services.clone();
    let import_commit_services = services;

    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| state.write().import.open = false,
            div {
                class: "modal panel",
                onclick: move |event| event.stop_propagation(),
                p { class: "eyebrow", "Bring more into the room" }
                h3 { class: "section-title", "Import bookmarks" }
                p { class: "muted-copy", "Paste a local path to a CSV, JSON, or X archive JS export. Preview runs as a dry parse first; the final write uses the Rust parser and local SQLite transaction." }
                div { class: "import-steps",
                    for step in [ImportStage::Source, ImportStage::Preview, ImportStage::Importing, ImportStage::Done] {
                        span {
                            class: if step == import_state.stage { "import-step active" } else { "import-step" },
                            "{step.as_str()}"
                        }
                    }
                }
                label {
                    class: "picker-button",
                    "Choose a file"
                    input {
                        key: "{import_state.picker_key}",
                        class: "hidden-file-input",
                        r#type: "file",
                        accept: ".csv,.json,.js",
                        onchange: move |event| {
                            if let Some(file) = event.files().into_iter().next() {
                                let path = file.path().display().to_string();
                                let mut next = state.write();
                                set_import_source(
                                    &mut next.import,
                                    path,
                                    Some("Selected file from native picker.".to_string()),
                                );
                                next.import.picker_key = next.import.picker_key.wrapping_add(1);
                            }
                        }
                    }
                }
                input {
                    class: "path-input",
                    r#type: "text",
                    value: "{import_state.path}",
                    placeholder: "/home/you/Downloads/bookmarks.json",
                    onfocus: move |_| enter_focus_scope(&mut state, FocusScope::ImportForm),
                    onblur: move |_| leave_focus_scope(&mut state, FocusScope::ImportForm),
                    oninput: move |event| {
                        let mut next = state.write();
                        set_import_source(&mut next.import, event.value(), None);
                    },
                }
                if let Some(preview) = &import_state.preview {
                    div { class: "preview-card",
                        div { class: "preview-metrics",
                            div { class: "detail-metric", span { "Format" } strong { "{preview.format}" } }
                            div { class: "detail-metric", span { "Detected" } strong { "{preview.bookmark_count}" } }
                            div { class: "detail-metric", span { "Source" } strong { "{preview.source_label}" } }
                        }
                        p { class: "muted-copy tiny", "{preview.duplicate_policy}" }
                        if !preview.sample.is_empty() {
                            div { class: "preview-list",
                                for item in &preview.sample {
                                    article { class: "preview-row",
                                        strong { "@{item.author_handle}" }
                                        p { "{item.content}" }
                                        small { "{item.tag_count} tags" }
                                        if item.has_media {
                                            small { "media" }
                                        } else {
                                            small { "text" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(message) = import_state.message {
                    p { class: "success-copy", "{message}" }
                }
                if let Some(error) = import_state.error {
                    p { class: "error-copy", "{error}" }
                }
                div {
                    class: "modal-actions",
                    button {
                        class: "ghost-button",
                        onclick: move |_| state.write().import.open = false,
                        "Close"
                    }
                    button {
                        class: "ghost-button",
                        onclick: move |_| {
                            let path = PathBuf::from(state.read().import.path.trim());
                            if path.as_os_str().is_empty() {
                                state.write().import.error = Some("Enter a file path before importing.".to_string());
                                return;
                            }
                            match import_preview_services.borrow().preview_import_file(&path) {
                                Ok(preview) => {
                                    let mut next = state.write();
                                    apply_import_preview(&mut next.import, preview);
                                    next.status = "Import preview ready.".to_string();
                                }
                                Err(error) => {
                                    let mut next = state.write();
                                    apply_import_error(&mut next.import, error.to_string());
                                }
                            }
                        },
                        "Preview"
                    }
                    button {
                        class: "accent-button",
                        onclick: move |_| {
                            let path = PathBuf::from(state.read().import.path.trim());
                            if path.as_os_str().is_empty() {
                                state.write().import.error = Some("Enter a file path before importing.".to_string());
                                return;
                            }
                            {
                                let mut next = state.write();
                                mark_importing(&mut next.import);
                                next.status = "Importing bookmarks…".to_string();
                            }
                            match import_commit_services.borrow().import_file(&path) {
                                Ok(imported) => {
                                    {
                                        let mut next = state.write();
                                        apply_import_success(&mut next.import, &path, imported);
                                        next.route = ScreenRoute::Library;
                                        next.status = format!("Imported {imported} bookmarks.");
                                    }
                                    reload_library(&import_commit_services, &mut state);
                                }
                                Err(error) => {
                                    let mut next = state.write();
                                    apply_import_error(&mut next.import, error.to_string());
                                }
                            }
                        },
                        "{import_button_label}"
                    }
                }
                p { class: "muted-copy tiny", "Direct X sync remains deferred; import stays local-first and reversible by deleting imported rows." }
            }
        }
    }
}
