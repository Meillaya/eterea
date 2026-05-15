#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaperTone {
    Cream,
    Offwhite,
    Gray,
}

impl PaperTone {
    pub(crate) const ALL: [Self; 3] = [Self::Cream, Self::Offwhite, Self::Gray];

    pub(crate) fn css_class(self) -> &'static str {
        match self {
            Self::Cream => "paper-tone-cream",
            Self::Offwhite => "paper-tone-offwhite",
            Self::Gray => "paper-tone-gray",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Cream => "Cream",
            Self::Offwhite => "Offwhite",
            Self::Gray => "Gray",
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

pub(crate) const DEFAULT_ACCENT: &str = "#a8421f";
pub(crate) const FONT_SERIF: &str = "Source Serif 4, Charter, Iowan Old Style, Georgia, serif";
pub(crate) const FONT_MONO: &str = "JetBrains Mono, ui-monospace, Menlo, monospace";

pub(crate) fn shell_classes(paper: PaperTone, density: Density) -> String {
    format!("app-shell {} {}", paper.css_class(), density.css_class())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_canonical_paper_and_density_classes() {
        assert_eq!(
            PaperTone::ALL.map(PaperTone::css_class),
            ["paper-tone-cream", "paper-tone-offwhite", "paper-tone-gray",]
        );
        assert_eq!(
            Density::ALL.map(Density::css_class),
            ["density-compact", "density-regular", "density-comfy",]
        );
        assert_eq!(
            PaperTone::ALL.map(PaperTone::label),
            ["Cream", "Offwhite", "Gray"]
        );
        assert_eq!(
            Density::ALL.map(Density::label),
            ["Compact", "Regular", "Comfy"]
        );
    }

    #[test]
    fn shell_classes_include_base_paper_and_density() {
        assert_eq!(
            shell_classes(PaperTone::Cream, Density::Regular),
            "app-shell paper-tone-cream density-regular"
        );
    }
}
