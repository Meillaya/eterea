use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("app crate should have repo parent")
        .parent()
        .expect("src directory should have repo parent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(repo_root().join(relative)).expect("file should be readable")
}

#[test]
fn workspace_targets_the_dioxus_app_path() {
    let cargo_toml = read("Cargo.toml");
    assert!(cargo_toml.contains("\"src/app\""));
    assert!(cargo_toml.contains("\"src/dioxus-app\""));
    assert!(!cargo_toml.contains("\"src-tauri\""));
}

#[test]
fn legacy_shell_directories_are_removed() {
    let root = repo_root();
    assert!(!root.join("src-tauri").exists());
    assert!(!root.join("src/frontend").exists());
}

#[test]
fn readme_uses_the_new_run_path() {
    let readme = read("README.md");
    assert!(readme.contains("cargo run -p eterea-dioxus"));
    assert!(!readme.contains("cargo tauri"));
    assert!(!readme.contains("bun run"));
}
