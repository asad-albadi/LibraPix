use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Gallery,
    Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootLifecycle {
    Active,
    Unavailable,
    Deactivated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryRootView {
    pub id: i64,
    pub normalized_path: PathBuf,
    pub lifecycle: RootLifecycle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IndexingSummary {
    pub scanned_roots: usize,
    pub candidate_files: usize,
    pub ignored_entries: usize,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub active_route: Route,
    pub root_input: String,
    pub selected_root_id: Option<i64>,
    pub library_roots: Vec<LibraryRootView>,
    pub indexing_summary: IndexingSummary,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_route: Route::Gallery,
            root_input: String::new(),
            selected_root_id: None,
            library_roots: Vec::new(),
            indexing_summary: IndexingSummary::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMessage {
    OpenGallery,
    OpenTimeline,
    SetRootInput,
    SetSelectedRoot,
    ReplaceLibraryRoots,
    ClearRootSelection,
    RecordIndexingSummary,
}

impl AppState {
    pub fn apply(&mut self, message: AppMessage) {
        match message {
            AppMessage::OpenGallery => self.active_route = Route::Gallery,
            AppMessage::OpenTimeline => self.active_route = Route::Timeline,
            AppMessage::SetRootInput
            | AppMessage::SetSelectedRoot
            | AppMessage::ReplaceLibraryRoots
            | AppMessage::ClearRootSelection
            | AppMessage::RecordIndexingSummary => {}
        }
    }

    pub fn set_root_input(&mut self, value: String) {
        self.root_input = value;
    }

    pub fn set_selected_root(&mut self, id: Option<i64>) {
        self.selected_root_id = id;
        if let Some(selected_id) = id
            && let Some(root) = self
                .library_roots
                .iter()
                .find(|root| root.id == selected_id)
        {
            self.root_input = root.normalized_path.to_string_lossy().to_string();
        }
    }

    pub fn replace_library_roots(&mut self, roots: Vec<LibraryRootView>) {
        self.library_roots = roots;
        if let Some(selected) = self.selected_root_id
            && !self.library_roots.iter().any(|root| root.id == selected)
        {
            self.selected_root_id = None;
        }
    }

    pub fn clear_selection_and_input(&mut self) {
        self.selected_root_id = None;
        self.root_input.clear();
    }

    pub fn record_indexing_summary(&mut self, summary: IndexingSummary) {
        self.indexing_summary = summary;
    }
}
