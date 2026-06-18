#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaperTone {
    Mocha,
    Macchiato,
    Latte,
}

impl PaperTone {
    pub(crate) const ALL: [Self; 3] = [Self::Mocha, Self::Macchiato, Self::Latte];

    pub(crate) fn css_class(self) -> &'static str {
        match self {
            Self::Mocha => "theme-mocha",
            Self::Macchiato => "theme-macchiato",
            Self::Latte => "theme-latte",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Mocha => "Mocha",
            Self::Macchiato => "Macchiato",
            Self::Latte => "Latte",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Density {
    Compact,
    Regular,
    Comfy,
}

impl Density {
    pub(crate) const ALL: [Self; 3] = [Self::Compact, Self::Regular, Self::Comfy];

    pub(crate) fn css_class(self) -> &'static str {
        match self {
            Self::Compact => "density-compact",
            Self::Regular => "density-regular",
            Self::Comfy => "density-comfy",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Regular => "Regular",
            Self::Comfy => "Comfy",
        }
    }
}

pub(crate) const DEFAULT_ACCENT: &str = "#f5c2e7";
pub(crate) const FONT_MONO: &str =
    "JetBrains Mono, Iosevka, IBM Plex Mono, Berkeley Mono, SFMono-Regular, ui-monospace, monospace";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontChoice {
    Mono,
    Plex,
    System,
}

impl FontChoice {
    pub(crate) const ALL: [Self; 3] = [Self::Mono, Self::Plex, Self::System];

    pub(crate) fn css_class(self) -> &'static str {
        match self {
            Self::Mono => "font-mono",
            Self::Plex => "font-plex",
            Self::System => "font-system",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Mono => "Mono",
            Self::Plex => "Plex",
            Self::System => "System",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WeightChoice {
    Regular,
    Medium,
    Bold,
}

impl WeightChoice {
    pub(crate) const ALL: [Self; 3] = [Self::Regular, Self::Medium, Self::Bold];

    pub(crate) fn css_class(self) -> &'static str {
        match self {
            Self::Regular => "weight-regular",
            Self::Medium => "weight-medium",
            Self::Bold => "weight-bold",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Regular => "Regular",
            Self::Medium => "Medium",
            Self::Bold => "Bold",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AccentChoice {
    Mauve,
    Rosewater,
    Blue,
    Green,
}

impl AccentChoice {
    pub(crate) const ALL: [Self; 4] = [Self::Mauve, Self::Rosewater, Self::Blue, Self::Green];

    pub(crate) fn css_class(self) -> &'static str {
        match self {
            Self::Mauve => "accent-mauve",
            Self::Rosewater => "accent-rosewater",
            Self::Blue => "accent-blue",
            Self::Green => "accent-green",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Mauve => "Mauve",
            Self::Rosewater => "Rosewater",
            Self::Blue => "Blue",
            Self::Green => "Green",
        }
    }

    pub(crate) fn color(self) -> &'static str {
        match self {
            Self::Mauve => DEFAULT_ACCENT,
            Self::Rosewater => "#f5e0dc",
            Self::Blue => "#89b4fa",
            Self::Green => "#a6e3a1",
        }
    }
}

pub(crate) fn shell_classes(
    paper: PaperTone,
    density: Density,
    font: FontChoice,
    weight: WeightChoice,
    accent: AccentChoice,
) -> String {
    format!(
        "app-shell terminal-shell {} {} {} {} {}",
        paper.css_class(),
        density.css_class(),
        font.css_class(),
        weight.css_class(),
        accent.css_class()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_canonical_theme_and_density_classes() {
        assert_eq!(
            PaperTone::ALL.map(PaperTone::css_class),
            ["theme-mocha", "theme-macchiato", "theme-latte",]
        );
        assert_eq!(
            Density::ALL.map(Density::css_class),
            ["density-compact", "density-regular", "density-comfy",]
        );
        assert_eq!(
            PaperTone::ALL.map(PaperTone::label),
            ["Mocha", "Macchiato", "Latte"]
        );
        assert_eq!(
            Density::ALL.map(Density::label),
            ["Compact", "Regular", "Comfy"]
        );
        assert_eq!(
            FontChoice::ALL.map(FontChoice::css_class),
            ["font-mono", "font-plex", "font-system"]
        );
        assert_eq!(
            WeightChoice::ALL.map(WeightChoice::css_class),
            ["weight-regular", "weight-medium", "weight-bold"]
        );
        assert_eq!(
            AccentChoice::ALL.map(AccentChoice::css_class),
            [
                "accent-mauve",
                "accent-rosewater",
                "accent-blue",
                "accent-green"
            ]
        );
    }

    #[test]
    fn shell_classes_include_base_paper_and_density() {
        assert_eq!(
            shell_classes(
                PaperTone::Mocha,
                Density::Regular,
                FontChoice::Mono,
                WeightChoice::Regular,
                AccentChoice::Mauve,
            ),
            "app-shell terminal-shell theme-mocha density-regular font-mono weight-regular accent-mauve"
        );
    }
}
