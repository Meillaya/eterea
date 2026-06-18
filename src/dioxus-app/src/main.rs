mod app;

use dioxus::desktop::{
    tao::{dpi::LogicalSize, window::Fullscreen},
    Config as DesktopConfig, WindowBuilder,
};
use dioxus::prelude::desktop;
use dioxus::LaunchBuilder;

const FULLSCREEN_ENV: &str = "ETEREA_WINDOW_MODE";
const DEFAULT_WINDOW_WIDTH: f64 = 1280.0;
const DEFAULT_WINDOW_HEIGHT: f64 = 820.0;

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
        .with_decorations(true)
        .with_resizable(true)
        .with_inner_size(LogicalSize::new(
            DEFAULT_WINDOW_WIDTH,
            DEFAULT_WINDOW_HEIGHT,
        ));

    if fullscreen_requested() {
        builder.with_fullscreen(Some(Fullscreen::Borderless(None)))
    } else {
        builder
    }
}

fn fullscreen_requested() -> bool {
    std::env::var(FULLSCREEN_ENV)
        .map(|value| matches!(value.trim(), "fullscreen" | "kiosk"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH, FULLSCREEN_ENV};

    #[test]
    fn documents_fullscreen_window_mode_env() {
        assert_eq!(FULLSCREEN_ENV, "ETEREA_WINDOW_MODE");
    }

    #[test]
    fn documents_regular_default_window_size() {
        assert_eq!(DEFAULT_WINDOW_WIDTH, 1280.0);
        assert_eq!(DEFAULT_WINDOW_HEIGHT, 820.0);
    }
}
