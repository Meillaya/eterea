#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LibraryScreenKind {
    Library,
    Favorites,
}

impl LibraryScreenKind {
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Library => "Library",
            Self::Favorites => "Favorites",
        }
    }
}
