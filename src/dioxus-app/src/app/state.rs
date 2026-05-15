use super::design_system::{Density, PaperTone, DEFAULT_ACCENT};
use super::route::ScreenRoute;
use eterea_app::{AppServices, AuthorSummary, BookmarkStats, ImportPreview, TopicSummary};
use eterea_core::Bookmark;
use std::{cell::RefCell, rc::Rc};

pub(crate) const PAGE_SIZE: usize = 48;

pub(crate) type Services = Rc<RefCell<AppServices>>;

#[derive(Clone, Default, PartialEq)]
pub(crate) struct Filters {
    pub(crate) query: String,
    pub(crate) author_query: String,
    pub(crate) from_date: String,
    pub(crate) to_date: String,
    pub(crate) selected_tag: Option<String>,
    pub(crate) favorites_only: bool,
    pub(crate) has_media_only: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) enum LayoutMode {
    Issue,
    FrontPage,
    LongRead,
    Spread,
}

impl LayoutMode {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::FrontPage => "Front Page",
            Self::LongRead => "Long-Read",
            Self::Spread => "Spread",
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::Issue => "A numbered editorial issue with one lead entry.",
            Self::FrontPage => "A newspaper-like lead story plus columns.",
            Self::LongRead => "A slower single-column reading mode.",
            Self::Spread => "A two-column spread for browsing more of the archive.",
        }
    }

    pub(crate) fn class_name(&self) -> &'static str {
        match self {
            Self::Issue => "feed-issue",
            Self::FrontPage => "feed-front",
            Self::LongRead => "feed-long",
            Self::Spread => "feed-spread",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum ImportStage {
    #[default]
    Source,
    Preview,
    Importing,
    Done,
}

impl ImportStage {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Preview => "Preview",
            Self::Importing => "Importing",
            Self::Done => "Done",
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct ImportState {
    pub(crate) open: bool,
    pub(crate) path: String,
    pub(crate) stage: ImportStage,
    pub(crate) preview: Option<ImportPreview>,
    pub(crate) imported_count: Option<usize>,
    pub(crate) message: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) picker_key: u32,
}

#[derive(Clone, PartialEq)]
pub(crate) struct AppearanceState {
    pub(crate) paper_tone: PaperTone,
    pub(crate) density: Density,
    pub(crate) accent: String,
}

impl Default for AppearanceState {
    fn default() -> Self {
        Self {
            paper_tone: PaperTone::Cream,
            density: Density::Regular,
            accent: DEFAULT_ACCENT.to_string(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct LibraryState {
    pub(crate) bookmarks: Vec<Bookmark>,
    pub(crate) stats: Option<BookmarkStats>,
    pub(crate) top_tags: Vec<(String, i64)>,
    pub(crate) authors: Vec<AuthorSummary>,
    pub(crate) topics: Vec<TopicSummary>,
    pub(crate) route: ScreenRoute,
    pub(crate) filters: Filters,
    pub(crate) layout: LayoutMode,
    pub(crate) total: i64,
    pub(crate) has_more: bool,
    pub(crate) page_size: usize,
    pub(crate) status: String,
    pub(crate) error: Option<String>,
    pub(crate) import: ImportState,
    pub(crate) appearance: AppearanceState,
    pub(crate) remote_images_enabled: bool,
    pub(crate) expanded_bookmark_id: Option<String>,
}

impl Default for LibraryState {
    fn default() -> Self {
        Self {
            bookmarks: Vec::new(),
            stats: None,
            top_tags: Vec::new(),
            authors: Vec::new(),
            topics: Vec::new(),
            route: ScreenRoute::Library,
            filters: Filters::default(),
            layout: LayoutMode::Issue,
            total: 0,
            has_more: false,
            page_size: PAGE_SIZE,
            status: "Archive ready.".to_string(),
            error: None,
            import: ImportState::default(),
            appearance: AppearanceState::default(),
            remote_images_enabled: false,
            expanded_bookmark_id: None,
        }
    }
}
