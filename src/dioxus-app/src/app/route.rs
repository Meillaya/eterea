#![allow(dead_code)]

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenRoute {
    Library,
    Favorites,
    Authors,
    Topics,
    Search,
    Import,
    Settings,
    Onboarding,
    Entry(String),
    Author(String),
    Topic(String),
}

pub(crate) const TERMINAL_TOP_TABS: [(&str, &str); 6] = [
    ("library", "library"),
    ("authors", "authors"),
    ("topics", "topics"),
    ("search", "search"),
    ("import", "import"),
    ("settings", "settings"),
];

impl ScreenRoute {
    pub(crate) fn parse(value: &str) -> Self {
        if let Some(id) = value.strip_prefix("entry:") {
            return Self::Entry(id.to_string());
        }
        if let Some(handle) = value.strip_prefix("author:") {
            return Self::Author(handle.to_string());
        }
        if let Some(tag) = value.strip_prefix("topic:") {
            return Self::Topic(tag.to_string());
        }

        match value {
            "favorites" => Self::Favorites,
            "authors" => Self::Authors,
            "topics" => Self::Topics,
            "search" => Self::Search,
            "import" => Self::Import,
            "settings" => Self::Settings,
            "onboarding" => Self::Onboarding,
            _ => Self::Library,
        }
    }

    pub(crate) fn nav_id(&self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Favorites => "favorites",
            Self::Authors | Self::Author(_) => "authors",
            Self::Topics | Self::Topic(_) => "topics",
            Self::Search => "search",
            Self::Import => "import",
            Self::Settings => "settings",
            Self::Onboarding => "onboarding",
            Self::Entry(_) => "library",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenRoute;

    #[test]
    fn parses_static_and_parameterized_routes() {
        assert_eq!(ScreenRoute::parse("library"), ScreenRoute::Library);
        assert_eq!(ScreenRoute::parse("favorites"), ScreenRoute::Favorites);
        assert_eq!(
            ScreenRoute::parse("entry:abc"),
            ScreenRoute::Entry("abc".to_string())
        );
        assert_eq!(
            ScreenRoute::parse("author:alice"),
            ScreenRoute::Author("alice".to_string())
        );
        assert_eq!(
            ScreenRoute::parse("topic:rust"),
            ScreenRoute::Topic("rust".to_string())
        );
        assert_eq!(ScreenRoute::parse("unknown"), ScreenRoute::Library);
    }

    #[test]
    fn maps_detail_routes_to_parent_navigation() {
        assert_eq!(ScreenRoute::Entry("1".to_string()).nav_id(), "library");
        assert_eq!(ScreenRoute::Author("alice".to_string()).nav_id(), "authors");
        assert_eq!(ScreenRoute::Topic("rust".to_string()).nav_id(), "topics");
    }

    #[test]
    fn terminal_top_tabs_match_design_html_navigation_contract() {
        assert_eq!(
            super::TERMINAL_TOP_TABS,
            [
                ("library", "library"),
                ("authors", "authors"),
                ("topics", "topics"),
                ("search", "search"),
                ("import", "import"),
                ("settings", "settings"),
            ]
        );

        for (route_id, expected_nav_id) in super::TERMINAL_TOP_TABS {
            assert_eq!(
                ScreenRoute::parse(route_id).nav_id(),
                expected_nav_id,
                "{route_id} should remain reachable from the terminal top tab bar"
            );
        }
    }
}
