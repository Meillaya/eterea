use super::design_system::{
    AccentChoice, Density, FontChoice, PaperTone, WeightChoice, DEFAULT_ACCENT,
};
use super::route::ScreenRoute;
use eterea_app::{AppServices, AuthorSummary, BookmarkStats, ImportPreview, TopicSummary};
use eterea_core::Bookmark;
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

pub(crate) const PAGE_SIZE: usize = 48;

pub(crate) type Services = Rc<RefCell<AppServices>>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum KeyboardMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Command,
    Search,
}

impl KeyboardMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Visual => "VISUAL",
            Self::Command => "COMMAND",
            Self::Search => "SEARCH",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum FocusScope {
    #[default]
    Shell,
    TextInput,
    CommandPalette,
    ImportForm,
    SettingsControl,
    OnboardingAction,
}

impl FocusScope {
    pub(crate) fn suppresses_global_shortcuts(self) -> bool {
        !matches!(self, Self::Shell)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShellState {
    pub(crate) mode: KeyboardMode,
    pub(crate) focus_scope: FocusScope,
    pub(crate) command_buffer: String,
    pub(crate) palette_query: String,
    pub(crate) palette_open: bool,
    pub(crate) keybindings_open: bool,
    pub(crate) selected_bookmark_ids: BTreeSet<String>,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            mode: KeyboardMode::Normal,
            focus_scope: FocusScope::Shell,
            command_buffer: String::new(),
            palette_query: String::new(),
            palette_open: false,
            keybindings_open: false,
            selected_bookmark_ids: BTreeSet::new(),
        }
    }
}

impl ShellState {
    pub(crate) fn enter_mode(&mut self, mode: KeyboardMode) {
        self.mode = mode;
        self.command_buffer.clear();
    }

    pub(crate) fn close_transient_ui(&mut self) {
        self.mode = KeyboardMode::Normal;
        self.focus_scope = FocusScope::Shell;
        self.command_buffer.clear();
        self.palette_query.clear();
        self.palette_open = false;
        self.keybindings_open = false;
    }

    pub(crate) fn suppresses_global_shortcuts(&self) -> bool {
        self.focus_scope.suppresses_global_shortcuts()
    }
}

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
    Table,
    Tree,
    Dashboard,
    Graph,
    Calendar,
}

impl LayoutMode {
    pub(crate) const ALL: [Self; 5] = [
        Self::Table,
        Self::Tree,
        Self::Dashboard,
        Self::Graph,
        Self::Calendar,
    ];

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Table => "Table",
            Self::Tree => "Tree",
            Self::Dashboard => "Dashboard",
            Self::Graph => "Graph",
            Self::Calendar => "Calendar",
        }
    }

    pub(crate) fn description(&self) -> &'static str {
        match self {
            Self::Table => "Dense terminal rows matching the default design.html library.",
            Self::Tree => "Group bookmarks by author in a filesystem-style tree.",
            Self::Dashboard => "Summarize totals, tags, authors, and recent saves.",
            Self::Graph => "Show tag/topic co-occurrence as a terminal graph.",
            Self::Calendar => "Show save/import activity as heatmaps and hourly bars.",
        }
    }

    pub(crate) fn class_name(&self) -> &'static str {
        match self {
            Self::Table => "view-table",
            Self::Tree => "view-tree",
            Self::Dashboard => "view-dashboard",
            Self::Graph => "view-graph",
            Self::Calendar => "view-calendar",
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
    pub(crate) font: FontChoice,
    pub(crate) weight: WeightChoice,
    pub(crate) accent_choice: AccentChoice,
    pub(crate) accent: String,
}

impl Default for AppearanceState {
    fn default() -> Self {
        Self {
            paper_tone: PaperTone::Mocha,
            density: Density::Regular,
            font: FontChoice::Mono,
            weight: WeightChoice::Regular,
            accent_choice: AccentChoice::Mauve,
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
    pub(crate) shell: ShellState,
    pub(crate) remote_images_enabled: bool,
    pub(crate) loaded_media_bookmark_ids: BTreeSet<String>,
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
            layout: LayoutMode::Table,
            total: 0,
            has_more: false,
            page_size: PAGE_SIZE,
            status: "Archive ready.".to_string(),
            error: None,
            import: ImportState::default(),
            appearance: AppearanceState::default(),
            shell: ShellState::default(),
            remote_images_enabled: false,
            loaded_media_bookmark_ids: BTreeSet::new(),
            expanded_bookmark_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_state_keeps_remote_images_off_by_default() {
        let state = LibraryState::default();

        assert!(!state.remote_images_enabled);
        assert!(state.loaded_media_bookmark_ids.is_empty());
    }

    #[test]
    fn shell_state_defaults_to_global_normal_mode() {
        let state = LibraryState::default();

        assert_eq!(state.shell.mode, KeyboardMode::Normal);
        assert_eq!(state.shell.mode.label(), "NORMAL");
        assert_eq!(state.shell.focus_scope, FocusScope::Shell);
        assert!(!state.shell.suppresses_global_shortcuts());
    }

    #[test]
    fn focus_scopes_protect_inputs_from_global_shortcuts() {
        for scope in [
            FocusScope::TextInput,
            FocusScope::CommandPalette,
            FocusScope::ImportForm,
            FocusScope::SettingsControl,
            FocusScope::OnboardingAction,
        ] {
            assert!(
                scope.suppresses_global_shortcuts(),
                "{scope:?} should own keystrokes while focused"
            );
        }
    }

    #[test]
    fn insert_mode_requires_owned_text_focus_scope() {
        let mut shell = ShellState::default();
        shell.enter_mode(KeyboardMode::Insert);
        shell.focus_scope = FocusScope::TextInput;

        assert_eq!(shell.mode, KeyboardMode::Insert);
        assert!(shell.suppresses_global_shortcuts());

        shell.close_transient_ui();
        assert_eq!(shell.mode, KeyboardMode::Normal);
        assert_eq!(shell.focus_scope, FocusScope::Shell);
    }
}
