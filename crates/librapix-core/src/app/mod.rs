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
    pub unreadable_entries: usize,
    pub new_files: usize,
    pub changed_files: usize,
    pub unchanged_files: usize,
    pub missing_marked: usize,
    pub read_model_count: usize,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub active_route: Route,
    pub root_input: String,
    pub selected_root_id: Option<i64>,
    pub selected_media_id: Option<i64>,
    pub library_roots: Vec<LibraryRootView>,
    pub indexing_summary: IndexingSummary,
    pub search_query: String,
    pub search_preview: Vec<String>,
    pub timeline_preview: Vec<String>,
    pub gallery_preview: Vec<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_route: Route::Gallery,
            root_input: String::new(),
            selected_root_id: None,
            selected_media_id: None,
            library_roots: Vec::new(),
            indexing_summary: IndexingSummary::default(),
            search_query: String::new(),
            search_preview: Vec::new(),
            timeline_preview: Vec::new(),
            gallery_preview: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMessage {
    OpenGallery,
    OpenTimeline,
    SetRootInput,
    SetSelectedRoot,
    SetSelectedMedia,
    ReplaceLibraryRoots,
    ClearRootSelection,
    RecordIndexingSummary,
    SetSearchQuery,
    ReplaceSearchPreview,
    ReplaceTimelinePreview,
    ReplaceGalleryPreview,
}

impl AppState {
    pub fn apply(&mut self, message: AppMessage) {
        match message {
            AppMessage::OpenGallery => self.active_route = Route::Gallery,
            AppMessage::OpenTimeline => self.active_route = Route::Timeline,
            AppMessage::SetRootInput
            | AppMessage::SetSelectedRoot
            | AppMessage::SetSelectedMedia
            | AppMessage::ReplaceLibraryRoots
            | AppMessage::ClearRootSelection
            | AppMessage::RecordIndexingSummary
            | AppMessage::SetSearchQuery
            | AppMessage::ReplaceSearchPreview
            | AppMessage::ReplaceTimelinePreview
            | AppMessage::ReplaceGalleryPreview => {}
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

    pub fn set_selected_media(&mut self, id: Option<i64>) {
        self.selected_media_id = id;
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

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn replace_search_preview(&mut self, rows: Vec<String>) {
        self.search_preview = rows;
    }

    pub fn replace_timeline_preview(&mut self, rows: Vec<String>) {
        self.timeline_preview = rows;
    }

    pub fn replace_gallery_preview(&mut self, rows: Vec<String>) {
        self.gallery_preview = rows;
    }
}
