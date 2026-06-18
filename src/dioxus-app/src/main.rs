mod app;

use dioxus::desktop::{tao::window::Fullscreen, Config as DesktopConfig, WindowBuilder};
use dioxus::prelude::desktop;
use dioxus::LaunchBuilder;

const FULLSCREEN_ENV: &str = "ETEREA_WINDOW_MODE";

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(desktop! {
            DesktopConfig::new()
                .with_window(window_builder())
                .with_menu(None)
        })
        .launch(app::App);
}

fn window_builder() -> WindowBuilder {
    let builder = WindowBuilder::new()
        .with_title("Eterea")
        .with_decorations(false)
        .with_resizable(true);

    if fullscreen_requested() {
        builder.with_fullscreen(Some(Fullscreen::Borderless(None)))
    } else {
        builder.with_maximized(true)
    }
}

fn fullscreen_requested() -> bool {
    std::env::var(FULLSCREEN_ENV)
        .map(|value| matches!(value.trim(), "fullscreen" | "kiosk"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::FULLSCREEN_ENV;

    #[test]
    fn documents_fullscreen_window_mode_env() {
        assert_eq!(FULLSCREEN_ENV, "ETEREA_WINDOW_MODE");
    }
}
