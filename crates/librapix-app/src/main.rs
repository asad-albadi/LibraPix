#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod assets;
mod format;
mod ui;

use chrono::Local;
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::image::FilterMethod;
use iced::widget::{
    Id, Space, button, column, container, image, mouse_area, operation, progress_bar, responsive,
    row, scrollable, stack, svg, text, text_input, vertical_slider,
};
use iced::{ContentFit, Element, Length, Size, Subscription, Task, Theme};
use librapix_config::{
    LocalePreference, ThemePreference, lexical_normalize_path, load_from_path, load_or_create,
    save_to_path,
};
use librapix_core::app::{
    AppMessage, AppState, IndexingSummary, LibraryRootView, RootLifecycle, Route,
};
use librapix_core::domain::non_destructive;
use librapix_i18n::{Locale, TextKey, Translator};
use librapix_indexer::{IgnoreEngine, ScanOptions, ScanRoot, scan_roots};
use librapix_projections::ProjectionMedia;
use librapix_projections::gallery::{GalleryQuery, GallerySort, project_gallery};
use librapix_projections::timeline::{
    TimelineAnchor, TimelineGranularity, build_timeline_anchors, project_timeline,
};
use librapix_search::{FuzzySearchStrategy, SearchDocument, SearchQuery, SearchStrategy};
use librapix_storage::{
    IndexedMediaWrite, IndexedMetadataStatus, SourceRootLifecycle, SourceRootStatisticsRecord,
    Storage, TagKind,
};
use librapix_thumbnails::{ensure_image_thumbnail, ensure_video_thumbnail, thumbnail_path};
use notify::{EventKind, RecursiveMode, Watcher};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime};
use ui::*;

fn main() -> iced::Result {
    iced::application(init, update, view)
        .title(title)
        .theme(theme)
        .subscription(subscription)
        .run()
}

fn init() -> (Librapix, Task<Message>) {
    (Librapix::default(), Task::done(Message::StartupRestore))
}

#[derive(Debug, Clone)]
enum Message {
    OpenGallery,
    OpenTimeline,
    SelectRoot(i64),
    DeactivateRoot,
    ReactivateRoot,
    RemoveRoot,
    RefreshRoots,
    RunIndexing,
    SearchQueryChanged(String),
    RunSearchQuery,
    RunTimelineProjection,
    RunGalleryProjection,
    SelectMedia(i64),
    DetailsTagInputChanged(String),
    AttachAppTag,
    AttachGameTag,
    DetachTagByName(String),
    DetailsStartEditTag(String),
    DetailsApplyTagEdit,
    DetailsCancelTagEdit,
    OpenSelectedFile,
    OpenSelectedFolder,
    CopySelectedFile,
    CopySelectedPath,
    IgnoreRuleInputChanged(String),
    IgnoreRuleAdd,
    IgnoreRuleToggleEnabled(i64),
    IgnoreRuleRemove(i64),
    IgnoreRuleStartEdit(i64),
    IgnoreRuleApplyEdit,
    IgnoreRuleCancelEdit,
    StartupRestore,
    FilesystemChanged(Vec<PathBuf>),
    SetFilterMediaKind(Option<String>),
    SetFilterExtension(Option<String>),
    SetFilterTag(Option<String>),
    MinFileSizeInputChanged(String),
    ApplyMinFileSize,
    TimelineScrubChanged(f32),
    TimelineScrubReleased,
    JumpToTimelineAnchor(usize),
    MediaViewportChanged { absolute_y: f32, max_y: f32 },
    KeyboardEvent(keyboard::Event),
    HydrateSnapshotComplete(Box<SnapshotHydrateResult>),
    ScanJobComplete(Box<ScanJobResult>),
    ProjectionJobComplete(Box<ProjectionJobResult>),
    ThumbnailBatchComplete(Box<ThumbnailBatchResult>),
    ThumbnailRetryEnqueue(ThumbnailWorkItem),
    OpenMediaById(i64),
    CopyMediaFileById(i64),
    DismissNewMediaAnnouncement,
    RefreshDiagnostics,
    OpenGitHub,
    UpdateChipPressed,
    UpdateCheckTick,
    UpdateCheckCompleted(UpdateCheckTaskResult),
    ToggleFilterDialog,
    OpenSettings,
    CloseSettings,
    OpenAbout,
    CloseAbout,
    OpenAddLibraryDialog,
    OpenEditLibraryDialog(i64),
    OpenLibraryStatisticsDialog(i64),
    CloseLibraryDialog,
    CloseLibraryStatisticsDialog,
    CloseAllDialogs,
    ModalContentClicked,
    LibraryDialogBrowseFolder,
    LibraryDialogPathInputChanged(String),
    LibraryDialogDisplayNameChanged(String),
    ToggleLibraryDialogManualPath,
    LibraryDialogTagInputChanged(String),
    LibraryDialogAddAppTag,
    LibraryDialogAddGameTag,
    LibraryDialogRemoveTag(String),
    LibraryDialogStartEditTag(String),
    LibraryDialogApplyTagEdit,
    LibraryDialogCancelTagEdit,
    SaveLibraryDialog,
    SaveLibraryAndAddAnother,
    SetFilterLibrary(Option<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BackgroundWorkReason {
    #[default]
    UserOrSystem,
    FilesystemWatch,
}

#[derive(Debug, Clone, Default)]
struct ActivityProgressState {
    stage_text: String,
    detail_text: String,
    items_done: usize,
    items_total: Option<usize>,
    roots_done: usize,
    roots_total: Option<usize>,
    queue_depth: usize,
    busy: bool,
    indeterminate: bool,
    started_at: Option<Instant>,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
enum ThumbnailState {
    Missing,
    Queued,
    Generating,
    Ready(PathBuf),
    Failed { retryable: bool, last_error: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThumbnailPresentationState {
    Ready,
    Loading,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThumbnailPresentation {
    resolved_path: Option<PathBuf>,
    state: ThumbnailPresentationState,
}

#[derive(Debug, Clone)]
struct ThumbnailWorkItem {
    generation: u64,
    media_id: i64,
    absolute_path: PathBuf,
    media_kind: String,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    attempt: u8,
}

#[derive(Debug, Clone)]
struct ThumbnailWorkOutcome {
    media_id: i64,
    state: ThumbnailState,
    thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct SnapshotHydrateInput {
    generation: u64,
    database_file: PathBuf,
}

#[derive(Debug, Clone, Default)]
struct SnapshotHydrateResult {
    generation: u64,
    roots: Vec<LibraryRootView>,
    ignore_rules: Vec<librapix_storage::IgnoreRuleRecord>,
    snapshot: Option<PersistedProjectionSnapshot>,
    snapshot_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ScanJobInput {
    generation: u64,
    reason: BackgroundWorkReason,
    database_file: PathBuf,
    min_file_size_bytes: u64,
    visible_media_ids: Vec<i64>,
    i18n: Translator,
}

#[derive(Debug, Clone, Default)]
struct ScanJobResult {
    generation: u64,
    reason: BackgroundWorkReason,
    roots: Vec<LibraryRootView>,
    indexing_summary: Option<IndexingSummary>,
    indexing_status: String,
    ignore_rules: Vec<librapix_storage::IgnoreRuleRecord>,
    scanned_root_ids: Vec<i64>,
    root_count: usize,
    changed_media: Vec<ThumbnailWorkItem>,
    visible_media: Vec<ThumbnailWorkItem>,
    has_media_changes: bool,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct ProjectionJobInput {
    generation: u64,
    reason: BackgroundWorkReason,
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    filter_media_kind: Option<String>,
    filter_extension: Option<String>,
    filter_tag: Option<String>,
    filter_source_root_id: Option<i64>,
    search_query: String,
    active_route: Route,
    i18n: Translator,
}

#[derive(Debug, Clone, Default)]
struct ProjectionJobResult {
    generation: u64,
    reason: BackgroundWorkReason,
    roots: Vec<LibraryRootView>,
    gallery_items: Vec<BrowseItem>,
    timeline_items: Vec<BrowseItem>,
    search_items: Vec<BrowseItem>,
    timeline_anchors: Vec<TimelineAnchor>,
    gallery_preview_lines: Vec<String>,
    timeline_preview_lines: Vec<String>,
    search_preview_lines: Vec<String>,
    media_cache: HashMap<i64, CachedDetails>,
    available_filter_tags: Vec<String>,
    browse_status: String,
    ignore_rules: Vec<librapix_storage::IgnoreRuleRecord>,
    snapshot_payload: Option<String>,
}

#[derive(Debug, Clone)]
struct ThumbnailBatchInput {
    generation: u64,
    thumbnails_dir: PathBuf,
    items: Vec<ThumbnailWorkItem>,
}

#[derive(Debug, Clone, Default)]
struct ThumbnailBatchResult {
    generation: u64,
    outcomes: Vec<ThumbnailWorkOutcome>,
    retry_items: Vec<ThumbnailWorkItem>,
    generated: usize,
    reused: usize,
    failed: usize,
}

#[derive(Debug, Clone, Default)]
struct BackgroundCoordinator {
    snapshot_generation: u64,
    reconcile_generation: u64,
    projection_generation: u64,
    thumbnail_generation: u64,
    snapshot_loaded: bool,
    startup_reconcile_queued: bool,
    reconcile_in_flight: bool,
    projection_in_flight: bool,
    thumbnail_in_flight: bool,
    pending_reconcile: bool,
    pending_reconcile_reason: BackgroundWorkReason,
    pending_projection: bool,
    pending_projection_reason: BackgroundWorkReason,
    pending_watch_paths: HashSet<PathBuf>,
    reconcile_has_media_changes: bool,
    reconcile_force_projection: bool,
    deferred_thumbnail_items: Vec<ThumbnailWorkItem>,
    thumbnail_queue: VecDeque<ThumbnailWorkItem>,
    thumbnail_queued_ids: HashSet<i64>,
    thumbnail_done: usize,
    thumbnail_total: usize,
    thumbnail_generated: usize,
    thumbnail_reused: usize,
    thumbnail_failed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedTimelineAnchor {
    group_index: usize,
    label: String,
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    item_count: usize,
    normalized_position: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedProjectionSnapshot {
    version: u32,
    gallery_items: Vec<BrowseItem>,
    timeline_items: Vec<BrowseItem>,
    timeline_anchors: Vec<PersistedTimelineAnchor>,
    available_filter_tags: Vec<String>,
    updated_unix_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UpdateCheckState {
    Unknown,
    Checking,
    UpToDate,
    UpdateAvailable { version: String, url: String },
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateCheckTrigger {
    Startup,
    Automatic,
    Manual,
}

#[derive(Debug, Clone)]
struct UpdateCheckTaskResult {
    trigger: UpdateCheckTrigger,
    checked_at: SystemTime,
    state: UpdateCheckState,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubLatestReleaseResponse {
    tag_name: String,
    html_url: String,
    prerelease: bool,
    draft: bool,
}

struct Librapix {
    state: AppState,
    i18n: Translator,
    theme_preference: ThemePreference,
    runtime: RuntimeContext,
    thumbnail_status: String,
    details_tag_input: String,
    details_lines: Vec<String>,
    details_action_status: String,
    details_preview_path: Option<PathBuf>,
    details_title: String,
    details_tags: Vec<DetailsTagChip>,
    details_editing_tag: Option<String>,
    ignore_rule_input: String,
    ignore_rules: Vec<librapix_storage::IgnoreRuleRecord>,
    ignore_rule_editing_id: Option<i64>,
    gallery_items: Vec<BrowseItem>,
    timeline_items: Vec<BrowseItem>,
    timeline_anchors: Vec<TimelineAnchor>,
    search_items: Vec<BrowseItem>,
    indexing_status: String,
    browse_status: String,
    root_status: String,
    last_click_media_id: Option<i64>,
    last_click_time: Option<Instant>,
    activity_status: String,
    activity_progress: ActivityProgressState,
    filter_media_kind: Option<String>,
    filter_extension: Option<String>,
    filter_tag: Option<String>,
    available_filter_tags: Vec<String>,
    min_file_size_bytes: u64,
    min_file_size_input: String,
    media_cache: HashMap<i64, CachedDetails>,
    thumbnail_states: HashMap<i64, ThumbnailState>,
    background: BackgroundCoordinator,
    diagnostics_lines: Vec<String>,
    diagnostics_events: Vec<String>,
    timeline_scrub_value: f32,
    timeline_scrubbing: bool,
    timeline_scrub_anchor_index: Option<usize>,
    timeline_scroll_max_y: f32,
    new_media_announcement: Option<NewMediaAnnouncement>,
    filter_dialog_open: bool,
    settings_open: bool,
    about_open: bool,
    library_dialog_open: bool,
    library_dialog_mode: LibraryDialogMode,
    library_dialog_path_input: String,
    library_dialog_display_name_input: String,
    library_dialog_manual_path_open: bool,
    library_dialog_tag_input: String,
    library_dialog_tags: Vec<(String, TagKind)>,
    library_dialog_editing_tag: Option<String>,
    library_stats_dialog_open: bool,
    library_stats_root_id: Option<i64>,
    library_stats_record: Option<SourceRootStatisticsRecord>,
    filter_source_root_id: Option<i64>,
    update_check_state: UpdateCheckState,
    last_successful_update_check: Option<SystemTime>,
    last_manual_update_check: Option<SystemTime>,
    last_auto_update_check_attempt: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryDialogMode {
    Add,
    Edit(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrowseItem {
    media_id: i64,
    title: String,
    thumbnail_path: Option<PathBuf>,
    media_kind: String,
    metadata_line: String,
    is_group_header: bool,
    line: String,
    aspect_ratio: f32,
    /// For group headers: image count in the group.
    group_image_count: Option<usize>,
    /// For group headers: video count in the group.
    group_video_count: Option<usize>,
}

#[derive(Debug, Clone)]
struct CachedDetails {
    absolute_path: PathBuf,
    media_kind: String,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    width_px: Option<u32>,
    height_px: Option<u32>,
    detail_thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct DetailsTagChip {
    name: String,
    kind: TagKind,
    inherited: bool,
}

#[derive(Debug, Clone)]
struct NewMediaAnnouncement {
    media_id: i64,
    title: String,
    metadata_line: String,
    preview_path: Option<PathBuf>,
    media_kind: String,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    width_px: Option<u32>,
    height_px: Option<u32>,
    absolute_path: PathBuf,
    additional_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyboardShortcutAction {
    CopyFile,
    CopyPath,
}

#[derive(Debug, Clone, Copy, Default)]
struct BrowseStats {
    shown_items: usize,
    image_count: usize,
    video_count: usize,
}

const GALLERY_THUMB_SIZE: u32 = 400;
const DETAIL_THUMB_SIZE: u32 = 800;
const TARGET_ROW_HEIGHT: f32 = 200.0;
const MAX_ROW_HEIGHT: f32 = 350.0;
const MAX_DIAGNOSTICS_EVENTS: usize = 100;
const MEDIA_SCROLLABLE_ID: &str = "media-pane-scrollable";
const SCRUBBER_YEAR_MARKER_LIMIT: usize = 10;
const MEDIA_SCROLLBAR_SPACING: f32 = SPACE_XS as f32;
const PANEL_SCROLLBAR_SPACING: f32 = SPACE_XS as f32;
const SCRUBBER_PANEL_WIDTH: f32 = 168.0;
const SCRUBBER_CHIP_TRACK_WIDTH: f32 = 96.0;
const FILTER_DIALOG_MAX_WIDTH: f32 = 480.0;
const FILTER_DIALOG_CHIP_ROW_MAX_WIDTH: f32 = FILTER_DIALOG_MAX_WIDTH - (SPACE_LG as f32 * 2.0);
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/asad-albadi/LibraPix/releases/latest";
const GITHUB_RELEASES_PAGE_URL: &str = "https://github.com/asad-albadi/LibraPix/releases";
const UPDATE_CHECK_TICK_INTERVAL: Duration = Duration::from_secs(60);
const AUTO_UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MANUAL_UPDATE_CHECK_COOLDOWN: Duration = Duration::from_secs(5 * 60);
const PROJECTION_SNAPSHOT_KEY: &str = "default";
const PROJECTION_SNAPSHOT_VERSION: u32 = 1;
const THUMBNAIL_BATCH_SIZE: usize = 32;
const THUMBNAIL_MAX_RETRY_ATTEMPTS: u8 = 5;
const THUMBNAIL_RETRY_DELAY_MS: u64 = 1200;
const VISIBLE_THUMBNAIL_QUEUE_LIMIT: usize = 180;

#[derive(Debug, Clone)]
struct RuntimeContext {
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    config_file: PathBuf,
}

impl Default for Librapix {
    fn default() -> Self {
        let bootstrap = bootstrap_runtime();
        let mut app = Self {
            state: AppState {
                library_roots: bootstrap.roots,
                ..AppState::default()
            },
            i18n: Translator::new(bootstrap.locale),
            theme_preference: bootstrap.theme_preference,
            runtime: RuntimeContext {
                database_file: bootstrap.database_file,
                thumbnails_dir: bootstrap.thumbnails_dir,
                config_file: bootstrap.config_file,
            },
            thumbnail_status: String::new(),
            details_tag_input: String::new(),
            details_lines: Vec::new(),
            details_action_status: String::new(),
            details_preview_path: None,
            details_title: String::new(),
            details_tags: Vec::new(),
            details_editing_tag: None,
            ignore_rule_input: String::new(),
            ignore_rules: Vec::new(),
            ignore_rule_editing_id: None,
            gallery_items: Vec::new(),
            timeline_items: Vec::new(),
            timeline_anchors: Vec::new(),
            search_items: Vec::new(),
            indexing_status: String::new(),
            browse_status: String::new(),
            root_status: String::new(),
            last_click_media_id: None,
            last_click_time: None,
            activity_status: String::new(),
            activity_progress: ActivityProgressState::default(),
            filter_media_kind: None,
            filter_extension: None,
            filter_tag: None,
            available_filter_tags: Vec::new(),
            min_file_size_bytes: 0,
            min_file_size_input: String::new(),
            media_cache: HashMap::new(),
            thumbnail_states: HashMap::new(),
            background: BackgroundCoordinator::default(),
            diagnostics_lines: Vec::new(),
            diagnostics_events: Vec::new(),
            timeline_scrub_value: 0.0,
            timeline_scrubbing: false,
            timeline_scrub_anchor_index: None,
            timeline_scroll_max_y: 0.0,
            new_media_announcement: None,
            filter_dialog_open: false,
            settings_open: false,
            about_open: false,
            library_dialog_open: false,
            library_dialog_mode: LibraryDialogMode::Add,
            library_dialog_path_input: String::new(),
            library_dialog_display_name_input: String::new(),
            library_dialog_manual_path_open: false,
            library_dialog_tag_input: String::new(),
            library_dialog_tags: Vec::new(),
            library_dialog_editing_tag: None,
            library_stats_dialog_open: false,
            library_stats_root_id: None,
            library_stats_record: None,
            filter_source_root_id: None,
            update_check_state: UpdateCheckState::Unknown,
            last_successful_update_check: None,
            last_manual_update_check: None,
            last_auto_update_check_attempt: None,
        };
        refresh_ignore_rules(&mut app);
        app
    }
}

fn title(app: &Librapix) -> String {
    let _ = app.i18n.locale();
    app.i18n.text(TextKey::AppTitle).to_owned()
}

fn theme(app: &Librapix) -> Theme {
    match app.theme_preference {
        ThemePreference::System => Theme::TokyoNight,
        ThemePreference::Dark => Theme::Dark,
        ThemePreference::Light => Theme::Light,
    }
}

#[derive(Debug, Clone, Hash)]
struct WatchSubscriptionConfig {
    roots: Vec<PathBuf>,
}

fn subscription(app: &Librapix) -> Subscription<Message> {
    let keyboard_subscription = keyboard::listen().map(Message::KeyboardEvent);
    let update_tick_subscription = Subscription::run(update_check_tick_stream);

    let roots = app
        .state
        .library_roots
        .iter()
        .filter(|root| matches!(root.lifecycle, RootLifecycle::Active))
        .map(|root| root.normalized_path.clone())
        .collect::<Vec<_>>();

    if roots.is_empty() {
        Subscription::batch(vec![keyboard_subscription, update_tick_subscription])
    } else {
        Subscription::batch(vec![
            keyboard_subscription,
            update_tick_subscription,
            Subscription::run_with(WatchSubscriptionConfig { roots }, watch_filesystem),
        ])
    }
}

fn update_check_tick_stream() -> impl iced::futures::Stream<Item = Message> + use<> {
    use iced::futures::sink::SinkExt;
    use iced::stream;

    stream::channel(1, async move |mut output| {
        loop {
            std::thread::sleep(UPDATE_CHECK_TICK_INTERVAL);
            let _ = output.send(Message::UpdateCheckTick).await;
        }
    })
}

fn watch_filesystem(
    config: &WatchSubscriptionConfig,
) -> impl iced::futures::Stream<Item = Message> + use<> {
    use iced::futures::StreamExt;
    use iced::futures::channel::mpsc;
    use iced::futures::sink::SinkExt;
    use iced::stream;

    let roots = config.roots.clone();
    stream::channel(100, async move |mut output| {
        let (tx, mut rx) = mpsc::unbounded::<notify::Result<notify::Event>>();
        let mut watcher = match notify::recommended_watcher(move |res| {
            let _ = tx.unbounded_send(res);
        }) {
            Ok(watcher) => watcher,
            Err(_) => {
                return;
            }
        };

        for root in &roots {
            let _ = watcher.watch(root, RecursiveMode::Recursive);
        }

        loop {
            match rx.next().await {
                Some(Ok(event)) => {
                    let should_trigger = matches!(
                        event.kind,
                        EventKind::Create(_)
                            | EventKind::Modify(_)
                            | EventKind::Remove(_)
                            | EventKind::Any
                    );
                    if should_trigger {
                        let mut batch = event
                            .paths
                            .into_iter()
                            .collect::<HashSet<_>>()
                            .into_iter()
                            .collect::<Vec<_>>();
                        batch.sort();
                        if !batch.is_empty() {
                            let _ = output.send(Message::FilesystemChanged(batch)).await;
                        }
                    }
                }
                Some(Err(_)) => {}
                None => break,
            }
        }
    })
}

fn message_event_label(msg: &Message) -> String {
    match msg {
        Message::OpenGallery => "OpenGallery".into(),
        Message::OpenTimeline => "OpenTimeline".into(),
        Message::SelectRoot(id) => format!("SelectRoot({id})"),
        Message::DeactivateRoot => "DeactivateRoot".into(),
        Message::ReactivateRoot => "ReactivateRoot".into(),
        Message::RemoveRoot => "RemoveRoot".into(),
        Message::RefreshRoots => "RefreshRoots".into(),
        Message::RunIndexing => "RunIndexing".into(),
        Message::SearchQueryChanged(v) => format!("SearchQueryChanged({})", v.len()),
        Message::RunSearchQuery => "RunSearchQuery".into(),
        Message::RunTimelineProjection => "RunTimelineProjection".into(),
        Message::RunGalleryProjection => "RunGalleryProjection".into(),
        Message::SelectMedia(id) => format!("SelectMedia({id})"),
        Message::DetailsTagInputChanged(v) => format!("DetailsTagInputChanged({})", v.len()),
        Message::AttachAppTag => "AttachAppTag".into(),
        Message::AttachGameTag => "AttachGameTag".into(),
        Message::DetachTagByName(name) => format!("DetachTagByName({name})"),
        Message::DetailsStartEditTag(name) => format!("DetailsStartEditTag({name})"),
        Message::DetailsApplyTagEdit => "DetailsApplyTagEdit".into(),
        Message::DetailsCancelTagEdit => "DetailsCancelTagEdit".into(),
        Message::OpenSelectedFile => "OpenSelectedFile".into(),
        Message::OpenSelectedFolder => "OpenSelectedFolder".into(),
        Message::CopySelectedFile => "CopySelectedFile".into(),
        Message::CopySelectedPath => "CopySelectedPath".into(),
        Message::IgnoreRuleInputChanged(v) => format!("IgnoreRuleInputChanged({})", v.len()),
        Message::IgnoreRuleAdd => "IgnoreRuleAdd".into(),
        Message::IgnoreRuleToggleEnabled(id) => format!("IgnoreRuleToggleEnabled({id})"),
        Message::IgnoreRuleRemove(id) => format!("IgnoreRuleRemove({id})"),
        Message::IgnoreRuleStartEdit(id) => format!("IgnoreRuleStartEdit({id})"),
        Message::IgnoreRuleApplyEdit => "IgnoreRuleApplyEdit".into(),
        Message::IgnoreRuleCancelEdit => "IgnoreRuleCancelEdit".into(),
        Message::StartupRestore => "StartupRestore".into(),
        Message::FilesystemChanged(paths) => {
            format!("FilesystemChanged(paths={})", paths.len())
        }
        Message::SetFilterMediaKind(k) => format!("SetFilterMediaKind({:?})", k.as_deref()),
        Message::SetFilterExtension(e) => format!("SetFilterExtension({:?})", e.as_deref()),
        Message::SetFilterTag(tag) => format!("SetFilterTag({:?})", tag.as_deref()),
        Message::MinFileSizeInputChanged(v) => format!("MinFileSizeInputChanged({})", v.len()),
        Message::ApplyMinFileSize => "ApplyMinFileSize".into(),
        Message::TimelineScrubChanged(value) => format!("TimelineScrubChanged({value:.3})"),
        Message::TimelineScrubReleased => "TimelineScrubReleased".into(),
        Message::JumpToTimelineAnchor(index) => format!("JumpToTimelineAnchor({index})"),
        Message::MediaViewportChanged { absolute_y, max_y } => {
            format!("MediaViewportChanged({absolute_y:.1}/{max_y:.1})")
        }
        Message::KeyboardEvent(_) => "KeyboardEvent".into(),
        Message::HydrateSnapshotComplete(_) => "HydrateSnapshotComplete".into(),
        Message::ScanJobComplete(_) => "ScanJobComplete".into(),
        Message::ProjectionJobComplete(_) => "ProjectionJobComplete".into(),
        Message::ThumbnailBatchComplete(_) => "ThumbnailBatchComplete".into(),
        Message::ThumbnailRetryEnqueue(_) => "ThumbnailRetryEnqueue".into(),
        Message::OpenMediaById(id) => format!("OpenMediaById({id})"),
        Message::CopyMediaFileById(id) => format!("CopyMediaFileById({id})"),
        Message::DismissNewMediaAnnouncement => "DismissNewMediaAnnouncement".into(),
        Message::RefreshDiagnostics => "RefreshDiagnostics".into(),
        Message::OpenGitHub => "OpenGitHub".into(),
        Message::UpdateChipPressed => "UpdateChipPressed".into(),
        Message::UpdateCheckTick => "UpdateCheckTick".into(),
        Message::UpdateCheckCompleted(result) => {
            format!("UpdateCheckCompleted({:?})", result.trigger)
        }
        Message::ToggleFilterDialog => "ToggleFilterDialog".into(),
        Message::OpenSettings => "OpenSettings".into(),
        Message::CloseSettings => "CloseSettings".into(),
        Message::OpenAbout => "OpenAbout".into(),
        Message::CloseAbout => "CloseAbout".into(),
        Message::OpenAddLibraryDialog => "OpenAddLibraryDialog".into(),
        Message::OpenEditLibraryDialog(id) => format!("OpenEditLibraryDialog({id})"),
        Message::OpenLibraryStatisticsDialog(id) => format!("OpenLibraryStatisticsDialog({id})"),
        Message::CloseLibraryDialog => "CloseLibraryDialog".into(),
        Message::CloseLibraryStatisticsDialog => "CloseLibraryStatisticsDialog".into(),
        Message::CloseAllDialogs => "CloseAllDialogs".into(),
        Message::ModalContentClicked => "ModalContentClicked".into(),
        Message::LibraryDialogBrowseFolder => "LibraryDialogBrowseFolder".into(),
        Message::LibraryDialogPathInputChanged(v) => {
            format!("LibraryDialogPathInputChanged({})", v.len())
        }
        Message::LibraryDialogDisplayNameChanged(v) => {
            format!("LibraryDialogDisplayNameChanged({})", v.len())
        }
        Message::ToggleLibraryDialogManualPath => "ToggleLibraryDialogManualPath".into(),
        Message::LibraryDialogTagInputChanged(v) => {
            format!("LibraryDialogTagInputChanged({})", v.len())
        }
        Message::LibraryDialogAddAppTag => "LibraryDialogAddAppTag".into(),
        Message::LibraryDialogAddGameTag => "LibraryDialogAddGameTag".into(),
        Message::LibraryDialogRemoveTag(name) => format!("LibraryDialogRemoveTag({name})"),
        Message::LibraryDialogStartEditTag(name) => format!("LibraryDialogStartEditTag({name})"),
        Message::LibraryDialogApplyTagEdit => "LibraryDialogApplyTagEdit".into(),
        Message::LibraryDialogCancelTagEdit => "LibraryDialogCancelTagEdit".into(),
        Message::SaveLibraryDialog => "SaveLibraryDialog".into(),
        Message::SaveLibraryAndAddAnother => "SaveLibraryAndAddAnother".into(),
        Message::SetFilterLibrary(_) => "SetFilterLibrary".into(),
    }
}

fn log_diagnostic_event(app: &mut Librapix, label: &str) {
    let ts = Local::now().format("%H:%M:%S%.3f");
    app.diagnostics_events.push(format!("{ts} {label}"));
    if app.diagnostics_events.len() > MAX_DIAGNOSTICS_EVENTS {
        app.diagnostics_events
            .drain(0..(app.diagnostics_events.len() - MAX_DIAGNOSTICS_EVENTS));
    }
}

fn close_all_dialogs(app: &mut Librapix) {
    app.filter_dialog_open = false;
    app.settings_open = false;
    app.about_open = false;
    app.library_dialog_open = false;
    app.library_stats_dialog_open = false;
    app.new_media_announcement = None;
}

fn update(app: &mut Librapix, message: Message) -> Task<Message> {
    log_diagnostic_event(app, &message_event_label(&message));

    match message {
        Message::OpenGallery => {
            app.state.apply(AppMessage::OpenGallery);
            app.timeline_scrubbing = false;
        }
        Message::OpenTimeline => {
            app.state.apply(AppMessage::OpenTimeline);
            app.timeline_scrubbing = false;
            sync_timeline_scrub_selection(app, app.timeline_scrub_value);
        }
        Message::SelectRoot(id) => {
            app.state.apply(AppMessage::SetSelectedRoot);
            app.state.set_selected_root(Some(id));
        }
        Message::DeactivateRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| {
                    storage.set_source_root_lifecycle(id, SourceRootLifecycle::Deactivated)
                })
                .is_ok()
            {
                sync_roots_to_config(&app.runtime.database_file, &app.runtime.config_file);
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
                if matches!(app.library_dialog_mode, LibraryDialogMode::Edit(_)) {
                    app.library_dialog_open = false;
                }
                app.library_stats_dialog_open = false;
            }
        }
        Message::ReactivateRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| {
                    storage.set_source_root_lifecycle(id, SourceRootLifecycle::Active)
                })
                .is_ok()
            {
                sync_roots_to_config(&app.runtime.database_file, &app.runtime.config_file);
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
                if matches!(app.library_dialog_mode, LibraryDialogMode::Edit(_)) {
                    app.library_dialog_open = false;
                }
                app.library_stats_dialog_open = false;
            }
        }
        Message::RemoveRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| storage.remove_source_root(id)).is_ok()
            {
                sync_roots_to_config(&app.runtime.database_file, &app.runtime.config_file);
                refresh_roots(app);
                app.state.apply(AppMessage::ClearRootSelection);
                app.state.clear_selection_and_input();
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
                app.library_dialog_open = false;
                app.library_stats_dialog_open = false;
                return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
            }
        }
        Message::RefreshRoots => {
            refresh_roots(app);
        }
        Message::RunIndexing => {
            return request_reconcile(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::SearchQueryChanged(value) => {
            app.state.apply(AppMessage::SetSearchQuery);
            app.state.set_search_query(value);
        }
        Message::RunSearchQuery => {
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::RunTimelineProjection => {
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::RunGalleryProjection => {
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::SelectMedia(media_id) => {
            if app
                .new_media_announcement
                .as_ref()
                .is_some_and(|announcement| announcement.media_id == media_id)
            {
                app.new_media_announcement = None;
            }
            let now = Instant::now();
            let is_double_click = app.last_click_media_id == Some(media_id)
                && app
                    .last_click_time
                    .is_some_and(|t| now.duration_since(t).as_millis() < 400);
            app.last_click_media_id = Some(media_id);
            app.last_click_time = Some(now);

            if is_double_click {
                open_selected_path(app, false);
            } else {
                app.state.apply(AppMessage::SetSelectedMedia);
                app.state.set_selected_media(Some(media_id));
                load_media_details_cached(app);
                let task = queue_selected_media_thumbnail(app);
                if app.background.thumbnail_in_flight {
                    return task;
                }
            }
        }
        Message::DetailsTagInputChanged(value) => {
            app.details_tag_input = value;
        }
        Message::AttachAppTag => {
            attach_tag_to_selected_media(app, TagKind::App);
        }
        Message::AttachGameTag => {
            attach_tag_to_selected_media(app, TagKind::Game);
        }
        Message::DetachTagByName(tag_name) => {
            detach_tag_from_selected_media(app, &tag_name);
        }
        Message::DetailsStartEditTag(tag_name) => {
            app.details_editing_tag = Some(tag_name.clone());
            app.details_tag_input = tag_name;
        }
        Message::DetailsApplyTagEdit => {
            apply_details_tag_edit(app);
        }
        Message::DetailsCancelTagEdit => {
            app.details_editing_tag = None;
            app.details_tag_input.clear();
        }
        Message::OpenSelectedFile => {
            open_selected_path(app, false);
        }
        Message::OpenSelectedFolder => {
            open_selected_path(app, true);
        }
        Message::CopySelectedFile => {
            copy_selected_file(app);
        }
        Message::CopySelectedPath => {
            copy_selected_path(app);
        }
        Message::IgnoreRuleInputChanged(value) => {
            app.ignore_rule_input = value;
        }
        Message::IgnoreRuleAdd => {
            add_ignore_rule(app);
        }
        Message::IgnoreRuleToggleEnabled(rule_id) => {
            toggle_ignore_rule(app, rule_id);
        }
        Message::IgnoreRuleRemove(rule_id) => {
            remove_ignore_rule(app, rule_id);
        }
        Message::IgnoreRuleStartEdit(rule_id) => {
            start_ignore_rule_edit(app, rule_id);
        }
        Message::IgnoreRuleApplyEdit => {
            apply_ignore_rule_edit(app);
        }
        Message::IgnoreRuleCancelEdit => {
            app.ignore_rule_editing_id = None;
            app.ignore_rule_input.clear();
        }
        Message::StartupRestore => {
            let mut tasks = Vec::new();

            tasks.push(start_snapshot_hydrate(app));
            if !app.state.library_roots.is_empty() {
                app.background.startup_reconcile_queued = true;
            } else {
                set_activity_ready(app);
            }

            if !matches!(app.update_check_state, UpdateCheckState::Checking) {
                tasks.push(start_update_check(app, UpdateCheckTrigger::Startup));
            }

            if tasks.is_empty() {
                return Task::none();
            }
            return Task::batch(tasks);
        }
        Message::FilesystemChanged(paths) => {
            return on_filesystem_changed(app, paths);
        }
        Message::SetFilterMediaKind(kind) => {
            app.filter_media_kind = kind;
            app.filter_extension = None;
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::SetFilterExtension(ext) => {
            app.filter_extension = ext;
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::SetFilterTag(tag) => {
            app.filter_tag = tag;
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::MinFileSizeInputChanged(value) => {
            app.min_file_size_input = value;
        }
        Message::ApplyMinFileSize => {
            if let Ok(kb) = app.min_file_size_input.trim().parse::<u64>() {
                app.min_file_size_bytes = kb * 1024;
            } else if app.min_file_size_input.trim().is_empty() {
                app.min_file_size_bytes = 0;
            }
            return request_reconcile(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::TimelineScrubChanged(value) => {
            return apply_timeline_scrub(app, value, true);
        }
        Message::TimelineScrubReleased => {
            app.timeline_scrubbing = false;
            app.timeline_scrub_anchor_index =
                scrub_value_to_anchor_index(&app.timeline_anchors, app.timeline_scrub_value);
        }
        Message::JumpToTimelineAnchor(index) => {
            return jump_to_timeline_anchor(app, index);
        }
        Message::MediaViewportChanged { absolute_y, max_y } => {
            sync_timeline_scrub_from_viewport(app, absolute_y, max_y);
        }
        Message::KeyboardEvent(event) => {
            if let Some(action) = shortcut_action_from_keyboard_event(&event) {
                match action {
                    KeyboardShortcutAction::CopyFile => copy_selected_file(app),
                    KeyboardShortcutAction::CopyPath => copy_selected_path(app),
                }
            }
        }
        Message::HydrateSnapshotComplete(result) => {
            let task = apply_snapshot_hydrate_result(app, *result);
            refresh_diagnostics(app);
            return task;
        }
        Message::ScanJobComplete(result) => {
            return apply_scan_job_result(app, *result);
        }
        Message::ProjectionJobComplete(result) => {
            return apply_projection_job_result(app, *result);
        }
        Message::ThumbnailBatchComplete(result) => {
            return apply_thumbnail_batch_result(app, *result);
        }
        Message::ThumbnailRetryEnqueue(item) => {
            if item_retry_generation_is_current(app, &item) {
                enqueue_thumbnail_items(app, vec![item]);
                return run_next_thumbnail_batch_if_idle(app);
            }
            refresh_diagnostics(app);
        }
        Message::OpenMediaById(media_id) => {
            open_media_by_id(app, media_id, false);
            app.new_media_announcement = None;
        }
        Message::CopyMediaFileById(media_id) => {
            copy_media_file_by_id(app, media_id);
            app.new_media_announcement = None;
        }
        Message::DismissNewMediaAnnouncement => {
            app.new_media_announcement = None;
        }
        Message::RefreshDiagnostics => {
            refresh_diagnostics(app);
        }
        Message::OpenGitHub => {
            let _ = opener::open(assets::REPO_URL);
        }
        Message::UpdateChipPressed => {
            if let UpdateCheckState::UpdateAvailable { url, .. } = &app.update_check_state {
                let target = if url.trim().is_empty() {
                    GITHUB_RELEASES_PAGE_URL
                } else {
                    url.as_str()
                };
                let _ = opener::open(target);
                return Task::none();
            }

            if matches!(app.update_check_state, UpdateCheckState::Checking) {
                return Task::none();
            }

            if !can_run_manual_update_check(app, SystemTime::now()) {
                return Task::none();
            }

            return start_update_check(app, UpdateCheckTrigger::Manual);
        }
        Message::UpdateCheckTick => {
            if should_run_auto_update_check(app, SystemTime::now()) {
                return start_update_check(app, UpdateCheckTrigger::Automatic);
            }
        }
        Message::UpdateCheckCompleted(result) => {
            apply_update_check_result(app, result);
        }
        Message::ToggleFilterDialog => {
            app.filter_dialog_open = !app.filter_dialog_open;
        }
        Message::OpenSettings => {
            app.settings_open = true;
        }
        Message::CloseSettings => {
            app.settings_open = false;
        }
        Message::OpenAbout => {
            app.about_open = true;
        }
        Message::CloseAbout => {
            app.about_open = false;
        }
        Message::OpenAddLibraryDialog => {
            open_add_library_dialog(app);
        }
        Message::OpenEditLibraryDialog(root_id) => {
            open_edit_library_dialog(app, root_id);
        }
        Message::OpenLibraryStatisticsDialog(root_id) => {
            open_library_statistics_dialog(app, root_id);
        }
        Message::CloseLibraryDialog => {
            app.library_dialog_open = false;
        }
        Message::CloseLibraryStatisticsDialog => {
            app.library_stats_dialog_open = false;
        }
        Message::CloseAllDialogs => {
            close_all_dialogs(app);
        }
        Message::ModalContentClicked => {}
        Message::LibraryDialogBrowseFolder => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.library_dialog_path_input = path.display().to_string();
            }
        }
        Message::LibraryDialogPathInputChanged(value) => {
            app.library_dialog_path_input = value;
        }
        Message::LibraryDialogDisplayNameChanged(value) => {
            app.library_dialog_display_name_input = value;
        }
        Message::ToggleLibraryDialogManualPath => {
            app.library_dialog_manual_path_open = !app.library_dialog_manual_path_open;
        }
        Message::LibraryDialogTagInputChanged(value) => {
            app.library_dialog_tag_input = value;
        }
        Message::LibraryDialogAddAppTag => {
            add_library_dialog_tag(app, TagKind::App);
        }
        Message::LibraryDialogAddGameTag => {
            add_library_dialog_tag(app, TagKind::Game);
        }
        Message::LibraryDialogRemoveTag(tag_name) => {
            app.library_dialog_tags
                .retain(|(name, _)| !name.eq_ignore_ascii_case(&tag_name));
            if app
                .library_dialog_editing_tag
                .as_ref()
                .is_some_and(|editing| editing.eq_ignore_ascii_case(&tag_name))
            {
                app.library_dialog_editing_tag = None;
                app.library_dialog_tag_input.clear();
            }
        }
        Message::LibraryDialogStartEditTag(tag_name) => {
            start_library_dialog_tag_edit(app, &tag_name);
        }
        Message::LibraryDialogApplyTagEdit => {
            apply_library_dialog_tag_edit(app);
        }
        Message::LibraryDialogCancelTagEdit => {
            app.library_dialog_editing_tag = None;
            app.library_dialog_tag_input.clear();
        }
        Message::SaveLibraryDialog => {
            if let Some(task) = save_library_dialog(app, false) {
                return task;
            }
        }
        Message::SaveLibraryAndAddAnother => {
            if let Some(task) = save_library_dialog(app, true) {
                return task;
            }
        }
        Message::SetFilterLibrary(root_id) => {
            app.filter_source_root_id = root_id;
            return request_projection_refresh(app, BackgroundWorkReason::UserOrSystem);
        }
    }

    Task::none()
}

#[cfg(test)]
fn projection_loading_key(search_query: &str, active_route: Route) -> TextKey {
    if !search_query.trim().is_empty() {
        TextKey::LoadingSearchLabel
    } else if matches!(active_route, Route::Timeline) {
        TextKey::LoadingTimelineLabel
    } else {
        TextKey::LoadingGalleryLabel
    }
}

fn update_chip_text_key(state: &UpdateCheckState) -> TextKey {
    match state {
        UpdateCheckState::Unknown | UpdateCheckState::Failed => TextKey::UpdateChipUnknownLabel,
        UpdateCheckState::Checking => TextKey::UpdateChipCheckingLabel,
        UpdateCheckState::UpToDate => TextKey::UpdateChipUpToDateLabel,
        UpdateCheckState::UpdateAvailable { .. } => TextKey::UpdateChipNewReleaseLabel,
    }
}

fn should_run_auto_update_check(app: &Librapix, now: SystemTime) -> bool {
    if matches!(app.update_check_state, UpdateCheckState::Checking) {
        return false;
    }
    let reference = app
        .last_successful_update_check
        .or(app.last_auto_update_check_attempt);
    match reference.and_then(|at| now.duration_since(at).ok()) {
        None => true,
        Some(elapsed) => elapsed >= AUTO_UPDATE_CHECK_INTERVAL,
    }
}

fn can_run_manual_update_check(app: &Librapix, now: SystemTime) -> bool {
    match app
        .last_manual_update_check
        .and_then(|at| now.duration_since(at).ok())
    {
        Some(elapsed) => elapsed >= MANUAL_UPDATE_CHECK_COOLDOWN,
        None => true,
    }
}

fn start_update_check(app: &mut Librapix, trigger: UpdateCheckTrigger) -> Task<Message> {
    app.update_check_state = UpdateCheckState::Checking;
    let now = SystemTime::now();
    match trigger {
        UpdateCheckTrigger::Startup | UpdateCheckTrigger::Automatic => {
            app.last_auto_update_check_attempt = Some(now);
        }
        UpdateCheckTrigger::Manual => {
            app.last_manual_update_check = Some(now);
        }
    }

    Task::perform(
        async move { run_update_check(trigger) },
        Message::UpdateCheckCompleted,
    )
}

fn run_update_check(trigger: UpdateCheckTrigger) -> UpdateCheckTaskResult {
    let checked_at = SystemTime::now();
    let state = match fetch_latest_release() {
        Ok(Some(latest)) => match compare_release_versions(APP_VERSION, &latest.version) {
            Some(true) => UpdateCheckState::UpdateAvailable {
                version: latest.version,
                url: latest.url,
            },
            Some(false) => UpdateCheckState::UpToDate,
            None => UpdateCheckState::Failed,
        },
        Ok(None) => UpdateCheckState::UpToDate,
        Err(()) => UpdateCheckState::Failed,
    };

    UpdateCheckTaskResult {
        trigger,
        checked_at,
        state,
    }
}

#[derive(Debug, Clone)]
struct LatestRelease {
    version: String,
    url: String,
}

fn fetch_latest_release() -> Result<Option<LatestRelease>, ()> {
    let mut response = ureq::get(GITHUB_LATEST_RELEASE_API_URL)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "librapix-update-checker")
        .call()
        .map_err(|_| ())?;
    let payload_text = response.body_mut().read_to_string().map_err(|_| ())?;
    let payload =
        serde_json::from_str::<GitHubLatestReleaseResponse>(&payload_text).map_err(|_| ())?;

    if payload.draft || payload.prerelease {
        return Ok(None);
    }

    let version = normalize_version_string(&payload.tag_name);
    if version.is_empty() {
        return Err(());
    }

    let url = if payload.html_url.trim().is_empty() {
        GITHUB_RELEASES_PAGE_URL.to_owned()
    } else {
        payload.html_url
    };

    Ok(Some(LatestRelease { version, url }))
}

fn normalize_version_string(version: &str) -> String {
    version.trim().trim_start_matches(['v', 'V']).to_owned()
}

fn compare_release_versions(current: &str, latest: &str) -> Option<bool> {
    let current = normalize_version_string(current);
    let latest = normalize_version_string(latest);
    match (Version::parse(&current), Version::parse(&latest)) {
        (Ok(current), Ok(latest)) => Some(latest > current),
        _ => None,
    }
}

fn apply_update_check_result(app: &mut Librapix, result: UpdateCheckTaskResult) {
    let success = matches!(
        result.state,
        UpdateCheckState::UpToDate | UpdateCheckState::UpdateAvailable { .. }
    );
    app.update_check_state = result.state;
    if success {
        app.last_successful_update_check = Some(result.checked_at);
    }
}

fn update_check_status_label(state: &UpdateCheckState) -> String {
    match state {
        UpdateCheckState::Unknown => "unknown".to_owned(),
        UpdateCheckState::Checking => "checking".to_owned(),
        UpdateCheckState::UpToDate => "up_to_date".to_owned(),
        UpdateCheckState::UpdateAvailable { version, .. } => {
            format!("update_available({version})")
        }
        UpdateCheckState::Failed => "failed".to_owned(),
    }
}

fn view(app: &Librapix) -> Element<'_, Message> {
    let _required_rules = non_destructive::required_rules();
    let is_gallery = matches!(app.state.active_route, Route::Gallery);
    let is_timeline = matches!(app.state.active_route, Route::Timeline);

    // ── Header ──
    let brand = row![
        svg(assets::logo_svg())
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .content_fit(ContentFit::Contain),
        row![
            text("Libra").size(FONT_DISPLAY).color(TEXT_PRIMARY),
            text("Pix").size(FONT_DISPLAY).color(ACCENT),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    let settings_btn = button(text(app.i18n.text(TextKey::SettingsButtonLabel)).size(FONT_BODY))
        .on_press(Message::OpenSettings)
        .style(subtle_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);
    let about_btn = button(text(app.i18n.text(TextKey::AboutButtonLabel)).size(FONT_BODY))
        .on_press(Message::OpenAbout)
        .style(subtle_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);

    let github_btn = button(
        image(assets::icon_github())
            .width(Length::Fixed(20.0))
            .height(Length::Fixed(20.0))
            .content_fit(ContentFit::Contain)
            .filter_method(FilterMethod::Linear),
    )
    .on_press(Message::OpenGitHub)
    .style(subtle_button_style)
    .padding([SPACE_XS as u16, SPACE_XS as u16]);

    let mut update_chip = button(
        text(app.i18n.text(update_chip_text_key(&app.update_check_state))).size(FONT_CAPTION),
    )
    .padding([SPACE_XS as u16, SPACE_MD as u16]);
    if !matches!(app.update_check_state, UpdateCheckState::Checking) {
        update_chip = update_chip.on_press(Message::UpdateChipPressed);
    }
    let update_chip = match &app.update_check_state {
        UpdateCheckState::UpdateAvailable { .. } => update_chip.style(primary_button_style),
        UpdateCheckState::Checking => update_chip.style(action_button_style),
        UpdateCheckState::UpToDate => update_chip.style(action_button_style),
        UpdateCheckState::Unknown | UpdateCheckState::Failed => {
            update_chip.style(subtle_button_style)
        }
    };

    let header = container(
        row![
            brand,
            Space::new().width(Length::Fill),
            row![
                image(assets::icon_search())
                    .width(Length::Fixed(18.0))
                    .height(Length::Fixed(18.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text_input(
                    app.i18n.text(TextKey::SearchInputLabel),
                    &app.state.search_query
                )
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::RunSearchQuery)
                .width(Length::Fixed(380.0))
                .style(search_input_style),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Space::new().width(Length::Fill),
            update_chip,
            render_activity_status(app),
            settings_btn,
            about_btn,
            github_btn,
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
    )
    .padding([0, SPACE_XL as u16])
    .center_y(HEADER_HEIGHT)
    .width(Length::Fill)
    .style(header_style);

    // ── Sidebar: Browse navigation ──
    let nav_section = column![
        section_heading(app.i18n.text(TextKey::BrowseSectionLabel)),
        button(
            row![
                image(assets::icon_gallery())
                    .width(Length::Fixed(18.0))
                    .height(Length::Fixed(18.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::GalleryTab)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .on_press(Message::OpenGallery)
        .style(nav_button_style(is_gallery))
        .padding([SPACE_SM as u16, SPACE_MD as u16]),
        button(
            row![
                image(assets::icon_timeline())
                    .width(Length::Fixed(18.0))
                    .height(Length::Fixed(18.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::TimelineTab)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .on_press(Message::OpenTimeline)
        .style(nav_button_style(is_timeline))
        .padding([SPACE_SM as u16, SPACE_MD as u16]),
    ]
    .spacing(SPACE_XS);

    // ── Sidebar: Library roots ──
    let roots_list: Element<'_, Message> = if app.state.library_roots.is_empty() {
        text(app.i18n.text(TextKey::EmptyRootsLabel))
            .size(FONT_BODY)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        app.state
            .library_roots
            .iter()
            .fold(column![].spacing(SPACE_2XS), |col, root| {
                let is_selected = app.state.selected_root_id == Some(root.id);
                let label = display_name_for_root(root);
                let status_color = match root.lifecycle {
                    RootLifecycle::Active => SUCCESS_COLOR,
                    RootLifecycle::Unavailable => WARNING_COLOR,
                    RootLifecycle::Deactivated => TEXT_DISABLED,
                };
                col.push(
                    row![
                        button(
                            row![
                                text("\u{25CF}").size(FONT_CAPTION).color(status_color),
                                text(label).size(FONT_BODY).color(if is_selected {
                                    TEXT_PRIMARY
                                } else {
                                    TEXT_SECONDARY
                                }),
                            ]
                            .spacing(SPACE_SM)
                            .align_y(iced::Alignment::Center),
                        )
                        .width(Length::Fill)
                        .on_press(Message::SelectRoot(root.id))
                        .style(nav_button_style(is_selected))
                        .padding([SPACE_XS as u16, SPACE_SM as u16]),
                        button(text(app.i18n.text(TextKey::RootEditButton)).size(FONT_CAPTION))
                            .on_press(Message::OpenEditLibraryDialog(root.id))
                            .style(subtle_button_style)
                            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                        button(text(app.i18n.text(TextKey::RootStatsButton)).size(FONT_CAPTION))
                            .on_press(Message::OpenLibraryStatisticsDialog(root.id))
                            .style(subtle_button_style)
                            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center),
                )
            })
            .into()
    };
    let library_section = column![
        section_heading(app.i18n.text(TextKey::LibrarySectionLabel)),
        roots_list,
        row![
            button(text(app.i18n.text(TextKey::LibraryAddButtonLabel)).size(FONT_BODY))
                .on_press(Message::OpenAddLibraryDialog)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::RootRefreshButton)).size(FONT_BODY))
                .on_press(Message::RefreshRoots)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .spacing(SPACE_XS),
        text(app.root_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_SM);

    let sidebar = container(
        scrollable(
            column![nav_section, h_divider(), library_section,]
                .spacing(SPACE_LG)
                .padding(SPACE_LG as u16),
        )
        .height(Length::Fill),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .style(sidebar_style);

    // ── Media pane ──
    let (media_header, media_scrollable_content) = render_media_panel(app);
    let base_media_scrollable = scrollable(media_scrollable_content)
        .id(Id::new(MEDIA_SCROLLABLE_ID))
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::default().spacing(MEDIA_SCROLLBAR_SPACING),
        ))
        .height(Length::Fill);
    let media_scrollable: Element<'_, Message> =
        if matches!(app.state.active_route, Route::Timeline) {
            base_media_scrollable
                .on_scroll(|viewport| Message::MediaViewportChanged {
                    absolute_y: viewport.absolute_offset().y,
                    max_y: (viewport.content_bounds().height - viewport.bounds().height).max(0.0),
                })
                .into()
        } else {
            base_media_scrollable.into()
        };
    let media_body: Element<'_, Message> = if matches!(app.state.active_route, Route::Timeline) {
        row![
            container(media_scrollable)
                .width(Length::Fill)
                .height(Length::Fill),
            render_timeline_scrubber(app),
        ]
        .spacing(SPACE_SM)
        .height(Length::Fill)
        .into()
    } else {
        container(media_scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    // ── Details pane ──
    let details_content = render_details_panel(app);

    // ── Body ──
    let body = row![
        sidebar,
        container(column![media_header, media_body,].spacing(SPACE_SM),)
            .padding(SPACE_LG as u16)
            .width(Length::Fill),
        container(
            scrollable(details_content)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default().spacing(PANEL_SCROLLBAR_SPACING),
                ))
                .height(Length::Fill),
        )
        .width(Length::Fixed(DETAILS_WIDTH))
        .padding(SPACE_LG as u16)
        .style(details_pane_style),
    ]
    .height(Length::Fill);

    let shell: Element<'_, Message> = container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app_bg_style)
        .into();

    let mut overlay: Element<'_, Message> = shell;
    if app.filter_dialog_open {
        overlay = stack([overlay, render_filter_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.new_media_announcement.is_some() {
        overlay = stack([overlay, render_new_media_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.settings_open {
        overlay = stack([overlay, render_settings_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.about_open {
        overlay = stack([overlay, render_about_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.library_dialog_open {
        overlay = stack([overlay, render_library_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.library_stats_dialog_open {
        overlay = stack([overlay, render_library_statistics_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    overlay
}

fn render_activity_status(app: &Librapix) -> Element<'_, Message> {
    let progress = &app.activity_progress;
    if !progress.busy {
        return text(app.activity_status.clone())
            .size(FONT_CAPTION)
            .color(ACCENT)
            .into();
    }

    let progress_line = if let Some(total) = progress.items_total {
        format!(
            "{}: {} / {}",
            app.i18n.text(TextKey::ProgressItemsLabel),
            progress.items_done,
            total
        )
    } else {
        format!("{}: --", app.i18n.text(TextKey::ProgressItemsLabel))
    };
    let queue_line = format!(
        "{}: {}",
        app.i18n.text(TextKey::ProgressQueueLabel),
        progress.queue_depth
    );
    let roots_line = progress.roots_total.map(|total| {
        format!(
            "{}: {} / {}",
            app.i18n.text(TextKey::ProgressRootsLabel),
            progress.roots_done.min(total),
            total
        )
    });
    let bar: Element<'_, Message> = if let Some(total) = progress.items_total {
        let capped_total = total.max(1) as f32;
        let done = progress.items_done.min(total) as f32;
        container(progress_bar(0.0..=capped_total, done))
            .height(Length::Fixed(4.0))
            .into()
    } else {
        container(progress_bar(
            0.0..=1.0,
            if progress.indeterminate { 0.35 } else { 0.0 },
        ))
        .height(Length::Fixed(4.0))
        .into()
    };

    let mut lines = column![
        text(progress.stage_text.clone())
            .size(FONT_CAPTION)
            .color(ACCENT),
        bar,
        text(progress_line).size(FONT_CAPTION).color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_2XS);
    if let Some(roots_line) = roots_line {
        lines = lines.push(text(roots_line).size(FONT_CAPTION).color(TEXT_TERTIARY));
    }
    lines = lines.push(text(queue_line).size(FONT_CAPTION).color(TEXT_TERTIARY));
    if !progress.detail_text.trim().is_empty() {
        lines = lines.push(
            text(progress.detail_text.clone())
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        );
    }
    if let Some(error) = progress.last_error.as_ref() {
        lines = lines.push(
            text(format!(
                "{}: {}",
                app.i18n.text(TextKey::ProgressErrorLabel),
                error
            ))
            .size(FONT_CAPTION)
            .color(WARNING_COLOR),
        );
    }

    container(lines).width(Length::Fixed(220.0)).into()
}

fn render_media_panel(app: &Librapix) -> (Element<'_, Message>, Element<'_, Message>) {
    let route_title = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::GalleryTab),
        Route::Timeline => app.i18n.text(TextKey::TimelineTab),
    };
    let run_msg = match app.state.active_route {
        Route::Gallery => Message::RunGalleryProjection,
        Route::Timeline => Message::RunTimelineProjection,
    };
    let browse_items = match app.state.active_route {
        Route::Gallery => &app.gallery_items,
        Route::Timeline => &app.timeline_items,
    };
    let stats_source = if app.state.search_query.trim().is_empty() {
        browse_items
    } else {
        &app.search_items
    };
    let stats = compute_browse_stats(stats_source);

    let content_header = row![
        text(route_title).size(FONT_TITLE).color(TEXT_PRIMARY),
        Space::new().width(Length::Fill),
        button(
            image(assets::icon_refresh())
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0))
                .content_fit(ContentFit::Contain)
                .filter_method(FilterMethod::Linear),
        )
        .on_press(run_msg)
        .style(subtle_button_style)
        .padding([SPACE_XS as u16, SPACE_XS as u16]),
        button(
            image(assets::icon_filter())
                .width(Length::Fixed(18.0))
                .height(Length::Fixed(18.0))
                .content_fit(ContentFit::Contain)
                .filter_method(FilterMethod::Linear),
        )
        .on_press(Message::ToggleFilterDialog)
        .style(subtle_button_style)
        .padding([SPACE_XS as u16, SPACE_XS as u16]),
        text(format!(
            "{}: {} \u{00B7} {}: {} \u{00B7} {}: {}",
            app.i18n.text(TextKey::StatsTotalLabel),
            stats.shown_items,
            app.i18n.text(TextKey::StatsImagesLabel),
            stats.image_count,
            app.i18n.text(TextKey::StatsVideosLabel),
            stats.video_count
        ))
        .size(FONT_BODY)
        .color(TEXT_SECONDARY),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    let header: Element<'_, Message> = content_header.into();

    let search_section: Element<'_, Message> = if !app.state.search_query.trim().is_empty() {
        if app.search_items.is_empty() {
            container(
                text(app.i18n.text(TextKey::EmptySearchResultsLabel))
                    .size(FONT_BODY)
                    .color(TEXT_SECONDARY),
            )
            .padding(SPACE_XL as u16)
            .width(Length::Fill)
            .style(empty_state_style)
            .into()
        } else {
            let search_header = row![
                text(app.i18n.text(TextKey::SearchResultLabel))
                    .size(FONT_SUBTITLE)
                    .color(TEXT_PRIMARY),
                text(format!("({})", app.search_items.len()))
                    .size(FONT_BODY)
                    .color(TEXT_SECONDARY),
            ]
            .spacing(SPACE_SM);

            column![
                search_header,
                render_justified_gallery(app, &app.search_items, app.state.selected_media_id),
            ]
            .spacing(SPACE_SM)
            .into()
        }
    } else {
        column![].into()
    };

    let empty_label = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::EmptyGalleryLabel),
        Route::Timeline => app.i18n.text(TextKey::EmptyTimelineLabel),
    };

    let browse_content: Element<'_, Message> = if browse_items.is_empty() {
        container(text(empty_label).size(FONT_SUBTITLE).color(TEXT_SECONDARY))
            .padding(SPACE_2XL as u16)
            .width(Length::Fill)
            .style(empty_state_style)
            .into()
    } else {
        match app.state.active_route {
            Route::Gallery => {
                render_justified_gallery(app, browse_items, app.state.selected_media_id)
            }
            Route::Timeline => render_timeline_view(app, browse_items, app.state.selected_media_id),
        }
    };

    let scrollable_content: Element<'_, Message> = column![search_section, browse_content]
        .spacing(SPACE_LG)
        .into();

    (header, scrollable_content)
}

fn render_justified_gallery<'a>(
    app: &'a Librapix,
    items: &'a [BrowseItem],
    selected_id: Option<i64>,
) -> Element<'a, Message> {
    let media: Vec<&BrowseItem> = items.iter().filter(|i| !i.is_group_header).collect();
    if media.is_empty() {
        return column![].into();
    }

    responsive(move |size: Size| {
        let available_width = size.width;
        let gap = GALLERY_GAP as f32;
        let mut grid = column![].spacing(GALLERY_GAP);
        let mut row_start = 0;

        while row_start < media.len() {
            let mut ar_sum = 0.0f32;
            let mut row_end = row_start;

            while row_end < media.len() {
                ar_sum += media[row_end].aspect_ratio;
                row_end += 1;
                let n_gaps = (row_end - row_start).saturating_sub(1) as f32;
                let row_h = (available_width - gap * n_gaps) / ar_sum;
                if row_h <= TARGET_ROW_HEIGHT {
                    break;
                }
            }

            let n = row_end - row_start;
            let n_gaps = n.saturating_sub(1) as f32;
            let row_height =
                ((available_width - gap * n_gaps) / ar_sum).clamp(100.0, MAX_ROW_HEIGHT);

            let mut row_widget = row![].spacing(GALLERY_GAP);
            for item in &media[row_start..row_end] {
                let portion = (item.aspect_ratio * 1000.0).max(1.0) as u16;
                let card =
                    render_media_card(app, item, selected_id == Some(item.media_id), row_height);
                row_widget = row_widget.push(container(card).width(Length::FillPortion(portion)));
            }
            grid = grid.push(row_widget);
            row_start = row_end;
        }

        grid.into()
    })
    .into()
}

fn render_media_card<'a>(
    app: &'a Librapix,
    item: &'a BrowseItem,
    selected: bool,
    height: f32,
) -> Element<'a, Message> {
    let thumb_presentation =
        resolve_thumbnail_presentation(app, item.media_id, item.thumbnail_path.as_ref());
    let thumb: Element<'_, Message> = match thumb_presentation.state {
        ThumbnailPresentationState::Ready => {
            let path = thumb_presentation
                .resolved_path
                .expect("ready thumbnail presentation must include path");
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fixed(height))
                .content_fit(ContentFit::Cover)
                .into()
        }
        ThumbnailPresentationState::Loading => render_thumbnail_placeholder(
            app.i18n.text(TextKey::ThumbnailLoadingLabel).to_owned(),
            None,
            height,
            true,
        ),
        ThumbnailPresentationState::Unavailable => render_thumbnail_placeholder(
            app.i18n.text(TextKey::ThumbnailUnavailable).to_owned(),
            None,
            height,
            false,
        ),
    };

    let kind_icon_handle = if item.media_kind.eq_ignore_ascii_case("video") {
        assets::icon_type_video()
    } else {
        assets::icon_type_image()
    };
    let kind_badge = image(kind_icon_handle)
        .width(Length::Fixed(16.0))
        .height(Length::Fixed(16.0))
        .content_fit(ContentFit::Contain)
        .filter_method(FilterMethod::Linear);

    let thumb_overlay: Element<'_, Message> = container(
        row![Space::new().width(Length::Fill), kind_badge].align_y(iced::Alignment::Start),
    )
    .width(Length::Fill)
    .height(Length::Fixed(height))
    .padding([SPACE_XS as u16, SPACE_XS as u16])
    .into();

    let thumb_with_badge: Element<'_, Message> = stack([thumb, thumb_overlay])
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .clip(true)
        .into();

    let card_content = column![
        thumb_with_badge,
        container(
            text(item.metadata_line.clone())
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY)
        )
        .padding([SPACE_XS as u16, SPACE_SM as u16]),
    ];

    button(card_content)
        .width(Length::Fill)
        .on_press(Message::SelectMedia(item.media_id))
        .style(card_button_style(selected))
        .padding(0)
        .into()
}

fn resolve_thumbnail_presentation(
    app: &Librapix,
    media_id: i64,
    fallback_thumbnail: Option<&PathBuf>,
) -> ThumbnailPresentation {
    let thumbnail_state = app.thumbnail_states.get(&media_id);
    if let Some(ThumbnailState::Ready(path)) = thumbnail_state {
        return ThumbnailPresentation {
            resolved_path: Some(path.clone()),
            state: ThumbnailPresentationState::Ready,
        };
    }
    if let Some(path) = fallback_thumbnail {
        return ThumbnailPresentation {
            resolved_path: Some(path.clone()),
            state: ThumbnailPresentationState::Ready,
        };
    }

    let state = if let Some(state) = thumbnail_state {
        if matches!(state, ThumbnailState::Queued | ThumbnailState::Generating)
            || matches!(
                state,
                ThumbnailState::Failed {
                    retryable: true,
                    ..
                }
            )
        {
            ThumbnailPresentationState::Loading
        } else {
            ThumbnailPresentationState::Unavailable
        }
    } else {
        if app.background.thumbnail_queued_ids.contains(&media_id) {
            ThumbnailPresentationState::Loading
        } else {
            ThumbnailPresentationState::Unavailable
        }
    };

    ThumbnailPresentation {
        resolved_path: None,
        state,
    }
}

fn render_thumbnail_placeholder<'a>(
    status_label: String,
    title: Option<&'a str>,
    height: f32,
    loading: bool,
) -> Element<'a, Message> {
    let mut body = column![Space::new().height(Length::Fill)].spacing(SPACE_XS);
    if loading {
        body = body.push(container(progress_bar(0.0..=1.0, 0.35)).height(Length::Fixed(4.0)));
    }
    if let Some(title) = title {
        body = body.push(text(title).size(FONT_CAPTION).color(TEXT_SECONDARY));
    }
    body = body.push(text(status_label).size(FONT_CAPTION).color(TEXT_TERTIARY));

    container(body.padding(SPACE_SM as u16))
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(thumb_placeholder_style)
        .into()
}

fn compute_browse_stats(items: &[BrowseItem]) -> BrowseStats {
    items.iter().filter(|item| !item.is_group_header).fold(
        BrowseStats::default(),
        |mut acc, item| {
            acc.shown_items += 1;
            if item.media_kind.eq_ignore_ascii_case("image") {
                acc.image_count += 1;
            } else if item.media_kind.eq_ignore_ascii_case("video") {
                acc.video_count += 1;
            }
            acc
        },
    )
}

fn render_timeline_view<'a>(
    app: &'a Librapix,
    items: &'a [BrowseItem],
    selected_id: Option<i64>,
) -> Element<'a, Message> {
    let mut sections = column![].spacing(SPACE_MD);
    let mut i = 0;
    let limit = items.len();

    while i < limit {
        if items[i].is_group_header {
            let header_item = &items[i];
            i += 1;

            let count_chips: Element<'_, Message> = if let (Some(img), Some(vid)) =
                (header_item.group_image_count, header_item.group_video_count)
            {
                row![
                    image(assets::icon_type_image())
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(format!("{img}"))
                        .size(FONT_CAPTION)
                        .color(TEXT_SECONDARY),
                    image(assets::icon_type_video())
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(format!("{vid}"))
                        .size(FONT_CAPTION)
                        .color(TEXT_SECONDARY),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                column![].into()
            };
            let group_header = container(
                row![
                    text(header_item.title.clone())
                        .size(FONT_SUBTITLE)
                        .color(TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    count_chips,
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center),
            )
            .padding([SPACE_SM as u16, 0]);

            let mut group_media: Vec<&BrowseItem> = Vec::new();
            while i < limit && !items[i].is_group_header {
                group_media.push(&items[i]);
                i += 1;
            }

            if group_media.is_empty() {
                sections = sections.push(group_header);
                continue;
            }

            let group_grid: Element<'_, Message> = responsive(move |size: Size| {
                let available_width = size.width;
                let gap = GALLERY_GAP as f32;
                let mut grid = column![].spacing(GALLERY_GAP);
                let mut row_start = 0;

                while row_start < group_media.len() {
                    let mut ar_sum = 0.0f32;
                    let mut row_end = row_start;

                    while row_end < group_media.len() {
                        ar_sum += group_media[row_end].aspect_ratio;
                        row_end += 1;
                        let n_gaps = (row_end - row_start).saturating_sub(1) as f32;
                        let row_h = (available_width - gap * n_gaps) / ar_sum;
                        if row_h <= TARGET_ROW_HEIGHT {
                            break;
                        }
                    }

                    let n_gaps = (row_end - row_start).saturating_sub(1) as f32;
                    let row_height =
                        ((available_width - gap * n_gaps) / ar_sum).clamp(100.0, MAX_ROW_HEIGHT);

                    let mut row_widget = row![].spacing(GALLERY_GAP);
                    for item in &group_media[row_start..row_end] {
                        let portion = (item.aspect_ratio * 1000.0).max(1.0) as u16;
                        let card = render_media_card(
                            app,
                            item,
                            selected_id == Some(item.media_id),
                            row_height,
                        );
                        row_widget =
                            row_widget.push(container(card).width(Length::FillPortion(portion)));
                    }
                    grid = grid.push(row_widget);
                    row_start = row_end;
                }

                grid.into()
            })
            .into();

            sections = sections.push(column![group_header, group_grid].spacing(SPACE_XS));
        } else {
            i += 1;
        }
    }

    sections.into()
}

fn render_timeline_scrubber(app: &Librapix) -> Element<'_, Message> {
    if app.timeline_anchors.is_empty() {
        return container(column![])
            .width(Length::Fixed(SCRUBBER_PANEL_WIDTH))
            .height(Length::Fill)
            .into();
    }

    let slider_value = (1.0 - app.timeline_scrub_value).clamp(0.0, 1.0);
    let slider = vertical_slider(0.0..=1.0, slider_value, |value| {
        Message::TimelineScrubChanged(1.0 - value)
    })
    .on_release(Message::TimelineScrubReleased)
    .step(0.001)
    .width(12.0)
    .height(Length::Fill);

    let active_anchor = app
        .timeline_scrub_anchor_index
        .and_then(|index| app.timeline_anchors.get(index));
    let chip_label = active_anchor
        .map(|anchor| format!("{} ({})", anchor.label, anchor.item_count))
        .unwrap_or_default();
    let chip_position = app.timeline_scrub_value.clamp(0.0, 1.0);

    let chip_track_content: Element<'_, Message> = if app.timeline_scrubbing {
        let top = ((chip_position * 1000.0).round() as u16).min(1000);
        let bottom = 1000u16.saturating_sub(top);
        column![
            Space::new().height(Length::FillPortion(top.max(1))),
            container(text(chip_label).size(FONT_CAPTION).color(TEXT_PRIMARY))
                .padding([SPACE_2XS as u16, SPACE_SM as u16])
                .style(scrubber_chip_style),
            Space::new().height(Length::FillPortion(bottom.max(1))),
        ]
        .height(Length::Fill)
        .into()
    } else {
        Space::new().height(Length::Fill).into()
    };
    let chip_track = container(chip_track_content)
        .width(Length::Fixed(SCRUBBER_CHIP_TRACK_WIDTH))
        .height(Length::Fill);

    let year_markers = timeline_year_markers(&app.timeline_anchors);
    let marker_track = render_timeline_year_marker_track(&year_markers);
    let scrub_controls = row![
        marker_track,
        row![chip_track, slider]
            .spacing(SPACE_XS)
            .height(Length::Fill),
    ]
    .spacing(SPACE_SM)
    .height(Length::Fill);

    container(column![scrub_controls].height(Length::Fill))
        .width(Length::Fixed(SCRUBBER_PANEL_WIDTH))
        .height(Length::Fill)
        .padding([SPACE_SM as u16, SPACE_XS as u16])
        .style(scrubber_panel_style)
        .into()
}

#[derive(Debug, Clone)]
struct FilterChipSpec {
    label: String,
    on_press: Message,
    active: bool,
}

fn estimate_filter_chip_width(label: &str) -> f32 {
    // Approximate text + horizontal button chrome for wrapping decisions.
    // Keep this estimate slightly conservative, but not so high that rows
    // break too early and leave obvious right-side whitespace.
    let glyph_width = 6.4;
    let text_width = label.chars().count() as f32 * glyph_width;
    let horizontal_padding = (SPACE_MD * 2) as f32;
    let estimated = text_width + horizontal_padding + 8.0;
    estimated.max(56.0)
}

fn render_wrapped_filter_chips(chips: Vec<FilterChipSpec>) -> Element<'static, Message> {
    let mut rows: Vec<Vec<FilterChipSpec>> = Vec::new();
    let mut current_row: Vec<FilterChipSpec> = Vec::new();
    let mut current_row_width = 0.0_f32;

    for chip in chips {
        let chip_width = estimate_filter_chip_width(&chip.label);
        let spacing = if current_row.is_empty() {
            0.0
        } else {
            SPACE_SM as f32
        };
        let would_overflow = !current_row.is_empty()
            && (current_row_width + spacing + chip_width) > FILTER_DIALOG_CHIP_ROW_MAX_WIDTH;

        if would_overflow {
            rows.push(current_row);
            current_row = Vec::new();
            current_row_width = 0.0;
        }

        if !current_row.is_empty() {
            current_row_width += SPACE_SM as f32;
        }
        current_row_width += chip_width;
        current_row.push(chip);
    }

    if !current_row.is_empty() {
        rows.push(current_row);
    }

    let mut wrapped_rows = column![].spacing(SPACE_SM);
    for row_chips in rows {
        let mut chip_row = row![].spacing(SPACE_SM).align_y(iced::Alignment::Center);
        for chip in row_chips {
            chip_row = chip_row.push(
                button(text(chip.label).size(FONT_BODY))
                    .on_press(chip.on_press)
                    .style(filter_chip_style(chip.active))
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
            );
        }
        wrapped_rows = wrapped_rows.push(chip_row);
    }

    wrapped_rows.into()
}

fn render_filter_dialog(app: &Librapix) -> Element<'_, Message> {
    let type_chips = render_wrapped_filter_chips(vec![
        FilterChipSpec {
            label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
            on_press: Message::SetFilterMediaKind(None),
            active: app.filter_media_kind.is_none(),
        },
        FilterChipSpec {
            label: app.i18n.text(TextKey::FilterImagesLabel).to_owned(),
            on_press: Message::SetFilterMediaKind(Some("image".to_owned())),
            active: app.filter_media_kind.as_deref() == Some("image"),
        },
        FilterChipSpec {
            label: app.i18n.text(TextKey::FilterVideosLabel).to_owned(),
            on_press: Message::SetFilterMediaKind(Some("video".to_owned())),
            active: app.filter_media_kind.as_deref() == Some("video"),
        },
    ]);

    let ext_list: &[&str] = match app.filter_media_kind.as_deref() {
        Some("image") => &["png", "jpg", "gif", "webp"],
        Some("video") => &["mp4", "mov", "mkv", "webm", "avi"],
        _ => &["png", "jpg", "gif", "webp", "mp4", "mov", "mkv", "webm"],
    };
    let mut ext_chips = vec![FilterChipSpec {
        label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
        on_press: Message::SetFilterExtension(None),
        active: app.filter_extension.is_none(),
    }];
    for ext in ext_list {
        let is_active = app.filter_extension.as_deref() == Some(ext);
        ext_chips.push(FilterChipSpec {
            label: ext.to_uppercase(),
            on_press: Message::SetFilterExtension(Some((*ext).to_owned())),
            active: is_active,
        });
    }
    let ext_chips = render_wrapped_filter_chips(ext_chips);

    let mut library_chip_specs = vec![FilterChipSpec {
        label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
        on_press: Message::SetFilterLibrary(None),
        active: app.filter_source_root_id.is_none(),
    }];

    for root in &app.state.library_roots {
        let label = root
            .display_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                root.normalized_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.normalized_path.display().to_string())
            });
        let is_active = app.filter_source_root_id == Some(root.id);
        library_chip_specs.push(FilterChipSpec {
            label,
            on_press: Message::SetFilterLibrary(Some(root.id)),
            active: is_active,
        });
    }

    let mut tag_chip_specs = vec![FilterChipSpec {
        label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
        on_press: Message::SetFilterTag(None),
        active: app.filter_tag.is_none(),
    }];

    for tag in &app.available_filter_tags {
        let active = app
            .filter_tag
            .as_ref()
            .is_some_and(|selected| selected == tag);
        tag_chip_specs.push(FilterChipSpec {
            label: tag.clone(),
            on_press: Message::SetFilterTag(Some(tag.clone())),
            active,
        });
    }

    let tag_section: Element<'_, Message> = if app.available_filter_tags.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_BODY)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        render_wrapped_filter_chips(tag_chip_specs)
    };

    let library_section: Element<'_, Message> = if app.state.library_roots.len() > 1 {
        column![
            section_heading(app.i18n.text(TextKey::FilterLibraryLabel)),
            render_wrapped_filter_chips(library_chip_specs),
        ]
        .spacing(SPACE_SM)
        .into()
    } else {
        column![].into()
    };

    let dialog_content = column![
        text(app.i18n.text(TextKey::FiltersButtonLabel))
            .size(FONT_TITLE)
            .color(TEXT_PRIMARY),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::FilterTypeLabel)),
            type_chips,
        ]
        .spacing(SPACE_SM),
        column![
            section_heading(app.i18n.text(TextKey::FilterExtensionLabel)),
            ext_chips,
        ]
        .spacing(SPACE_SM),
        library_section,
        column![
            section_heading(app.i18n.text(TextKey::FilterTagsLabel)),
            tag_section,
        ]
        .spacing(SPACE_SM),
        button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
            .on_press(Message::ToggleFilterDialog)
            .style(primary_button_style)
            .padding([SPACE_SM as u16, SPACE_MD as u16]),
    ]
    .spacing(SPACE_LG);

    let dialog = container(dialog_content)
        .width(Length::Fill)
        .max_width(FILTER_DIALOG_MAX_WIDTH)
        .padding(SPACE_LG as u16)
        .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

fn render_modal_overlay(dialog: Element<'_, Message>) -> Element<'_, Message> {
    let dialog_surface = mouse_area(dialog).on_press(Message::ModalContentClicked);
    let centered_dialog = container(dialog_surface)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    mouse_area(
        container(centered_dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([SPACE_2XL as u16, SPACE_XL as u16])
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseAllDialogs)
    .into()
}

fn render_management_chip(
    label: impl Into<String>,
    tone_key: &str,
    status_label: Option<String>,
    toggle: Option<(String, Message)>,
    edit: Option<(String, Message)>,
    remove: Option<Message>,
) -> Element<'static, Message> {
    let label = label.into();
    let tone = chip_tone_for_key(tone_key);
    let mut controls = row![].spacing(SPACE_2XS).align_y(iced::Alignment::Center);
    if let Some((toggle_label, msg)) = toggle {
        controls = controls.push(
            button(text(toggle_label).size(FONT_CAPTION))
                .on_press(msg)
                .style(managed_chip_action_style(tone, false))
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
    }
    if let Some((edit_label, msg)) = edit {
        controls = controls.push(
            button(text(edit_label).size(FONT_CAPTION))
                .on_press(msg)
                .style(managed_chip_action_style(tone, false))
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
    }
    if let Some(msg) = remove {
        controls = controls.push(
            button(text("x").size(FONT_CAPTION))
                .on_press(msg)
                .style(managed_chip_action_style(tone, true))
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
    }
    let status: Element<'static, Message> = if let Some(value) = status_label {
        text(value)
            .size(FONT_CAPTION)
            .color(tone.accent_text)
            .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };
    container(
        row![
            column![text(label).size(FONT_BODY).color(tone.text), status,].spacing(SPACE_2XS),
            Space::new().width(Length::Fill),
            controls,
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
    )
    .width(Length::Fill)
    .padding([SPACE_XS as u16, SPACE_SM as u16])
    .style(managed_chip_style(tone))
    .into()
}

fn render_settings_dialog(app: &Librapix) -> Element<'_, Message> {
    let indexing_section = column![
        section_heading(app.i18n.text(TextKey::IndexingSectionLabel)),
        button(
            row![
                image(assets::icon_index())
                    .width(Length::Fixed(18.0))
                    .height(Length::Fixed(18.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::IndexRunButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::RunIndexing)
        .style(primary_button_style)
        .width(Length::Fill)
        .padding([SPACE_SM as u16, SPACE_MD as u16]),
        text(app.indexing_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
        text(app.thumbnail_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_SM);

    let ignore_input_actions: Element<'_, Message> = if app.ignore_rule_editing_id.is_some() {
        row![
            button(text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_CAPTION))
                .on_press(Message::IgnoreRuleApplyEdit)
                .style(primary_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_CAPTION))
                .on_press(Message::IgnoreRuleCancelEdit)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        row![
            button(text(app.i18n.text(TextKey::IgnoreRuleAddButton)).size(FONT_CAPTION))
                .on_press(Message::IgnoreRuleAdd)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS)
        .into()
    };

    let ignore_chip_list: Element<'_, Message> = if app.ignore_rules.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        app.ignore_rules
            .iter()
            .fold(column![].spacing(SPACE_XS), |col, rule| {
                let status = if rule.is_enabled {
                    app.i18n.text(TextKey::IgnoreRuleEnabled)
                } else {
                    app.i18n.text(TextKey::IgnoreRuleDisabled)
                };
                let toggle_label = if rule.is_enabled {
                    app.i18n.text(TextKey::IgnoreRuleDisableButton)
                } else {
                    app.i18n.text(TextKey::IgnoreRuleAddButton)
                };
                col.push(render_management_chip(
                    &rule.pattern,
                    &rule.pattern,
                    Some(status.to_owned()),
                    Some((
                        toggle_label.to_owned(),
                        Message::IgnoreRuleToggleEnabled(rule.id),
                    )),
                    Some((
                        app.i18n.text(TextKey::RootEditButton).to_owned(),
                        Message::IgnoreRuleStartEdit(rule.id),
                    )),
                    Some(Message::IgnoreRuleRemove(rule.id)),
                ))
            })
            .into()
    };

    let ignore_section = column![
        section_heading(app.i18n.text(TextKey::IgnoreRuleInputLabel)),
        text_input("*.tmp, **/cache/**", &app.ignore_rule_input)
            .on_input(Message::IgnoreRuleInputChanged)
            .on_submit(if app.ignore_rule_editing_id.is_some() {
                Message::IgnoreRuleApplyEdit
            } else {
                Message::IgnoreRuleAdd
            })
            .style(field_input_style),
        ignore_input_actions,
        ignore_chip_list,
        h_divider(),
        row![
            text(app.i18n.text(TextKey::MinFileSizeLabel))
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
            text_input(
                app.i18n.text(TextKey::MinFileSizeKbSuffix),
                &app.min_file_size_input
            )
            .on_input(Message::MinFileSizeInputChanged)
            .on_submit(Message::ApplyMinFileSize)
            .width(Length::Fixed(60.0))
            .style(field_input_style),
            text(app.i18n.text(TextKey::MinFileSizeKbSuffix))
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
            button(text(app.i18n.text(TextKey::ApplyLabel)).size(FONT_CAPTION))
                .on_press(Message::ApplyMinFileSize)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(SPACE_SM);

    let diagnostics_section = {
        let state_lines = if app.diagnostics_lines.is_empty() {
            column![
                text("Click Refresh to load state.")
                    .size(FONT_CAPTION)
                    .color(TEXT_TERTIARY)
            ]
        } else {
            app.diagnostics_lines
                .iter()
                .fold(column![].spacing(SPACE_2XS), |col, line| {
                    col.push(
                        text(line.as_str())
                            .size(FONT_CAPTION)
                            .color(TEXT_TERTIARY)
                            .font(iced::Font::MONOSPACE),
                    )
                })
        };
        let event_lines = if app.diagnostics_events.is_empty() {
            column![
                text("(no events yet)")
                    .size(FONT_CAPTION)
                    .color(TEXT_TERTIARY)
            ]
        } else {
            app.diagnostics_events
                .iter()
                .rev()
                .fold(column![].spacing(SPACE_2XS), |col, line| {
                    col.push(
                        text(line.as_str())
                            .size(FONT_CAPTION)
                            .color(TEXT_TERTIARY)
                            .font(iced::Font::MONOSPACE),
                    )
                })
        };
        column![
            row![
                section_heading(app.i18n.text(TextKey::DiagnosticsSectionLabel)),
                Space::new().width(Length::Fill),
                button(text(app.i18n.text(TextKey::RefreshButton)).size(FONT_CAPTION))
                    .on_press(Message::RefreshDiagnostics)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_XS as u16]),
            ]
            .align_y(iced::Alignment::Center),
            text("Events (newest first)")
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
            scrollable(event_lines).height(Length::Fixed(120.0)),
            text("State").size(FONT_CAPTION).color(TEXT_SECONDARY),
            state_lines,
        ]
        .spacing(SPACE_SM)
    };

    let dialog_content = column![
        row![
            text(app.i18n.text(TextKey::SettingsDialogTitle))
                .size(FONT_TITLE)
                .color(TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
                .on_press(Message::CloseSettings)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .align_y(iced::Alignment::Center),
        h_divider(),
        indexing_section,
        h_divider(),
        ignore_section,
        h_divider(),
        diagnostics_section,
        h_divider(),
    ]
    .spacing(SPACE_LG);

    let dialog = container(
        scrollable(dialog_content)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default().spacing(PANEL_SCROLLBAR_SPACING),
            ))
            .height(Length::Fill)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .max_width(480.0)
    .max_height(560.0)
    .padding(SPACE_LG as u16)
    .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

fn render_about_dialog(app: &Librapix) -> Element<'_, Message> {
    let dialog_content = column![
        row![
            text(app.i18n.text(TextKey::AboutDialogTitle))
                .size(FONT_TITLE)
                .color(TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
                .on_press(Message::CloseAbout)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .align_y(iced::Alignment::Center),
        h_divider(),
        row![
            svg(assets::logo_svg())
                .width(Length::Fixed(44.0))
                .height(Length::Fixed(44.0))
                .content_fit(ContentFit::Contain),
            row![
                text("Libra").size(FONT_SUBTITLE).color(TEXT_PRIMARY),
                text("Pix").size(FONT_SUBTITLE).color(ACCENT),
            ]
            .spacing(0)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::AboutVersionLabel),
            APP_VERSION
        ))
        .size(FONT_BODY)
        .color(TEXT_SECONDARY),
        text(app.i18n.text(TextKey::AboutCreatorLabel))
            .size(FONT_BODY)
            .color(TEXT_SECONDARY),
        text(app.i18n.text(TextKey::AboutWeekendProjectNote))
            .size(FONT_BODY)
            .color(TEXT_SECONDARY),
        text(app.i18n.text(TextKey::AboutSecondNote))
            .size(FONT_BODY)
            .color(TEXT_SECONDARY),
    ]
    .spacing(SPACE_LG);

    let dialog = container(dialog_content)
        .width(Length::Fill)
        .max_width(460.0)
        .padding(SPACE_LG as u16)
        .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

fn render_library_dialog(app: &Librapix) -> Element<'_, Message> {
    let (title, is_add_mode, edit_root_id) = match app.library_dialog_mode {
        LibraryDialogMode::Add => (app.i18n.text(TextKey::LibraryDialogAddTitle), true, None),
        LibraryDialogMode::Edit(root_id) => (
            app.i18n.text(TextKey::LibraryDialogEditTitle),
            false,
            Some(root_id),
        ),
    };

    let path_toggle_label = if app.library_dialog_manual_path_open {
        app.i18n.text(TextKey::HidePathFieldLabel)
    } else {
        app.i18n.text(TextKey::ShowPathFieldLabel)
    };

    let path_field: Element<'_, Message> = if app.library_dialog_manual_path_open {
        text_input(
            app.i18n.text(TextKey::FolderPathPlaceholder),
            &app.library_dialog_path_input,
        )
        .on_input(Message::LibraryDialogPathInputChanged)
        .style(field_input_style)
        .into()
    } else {
        column![].into()
    };

    let tags_list: Element<'_, Message> = if app.library_dialog_tags.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        app.library_dialog_tags
            .iter()
            .fold(column![].spacing(SPACE_XS), |col, (name, kind)| {
                col.push(render_management_chip(
                    name,
                    name,
                    Some(kind.as_str().to_owned()),
                    None,
                    Some((
                        app.i18n.text(TextKey::RootEditButton).to_owned(),
                        Message::LibraryDialogStartEditTag(name.clone()),
                    )),
                    Some(Message::LibraryDialogRemoveTag(name.clone())),
                ))
            })
            .into()
    };

    let library_tag_actions: Element<'_, Message> = if app.library_dialog_editing_tag.is_some() {
        row![
            button(text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_CAPTION))
                .on_press(Message::LibraryDialogApplyTagEdit)
                .style(primary_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_CAPTION))
                .on_press(Message::LibraryDialogCancelTagEdit)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        row![
            button(text(app.i18n.text(TextKey::RootTagAddButton)).size(FONT_CAPTION))
                .on_press(Message::LibraryDialogAddAppTag)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            button(text(app.i18n.text(TextKey::RootTagGameButton)).size(FONT_CAPTION))
                .on_press(Message::LibraryDialogAddGameTag)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS)
        .into()
    };

    let lifecycle_actions: Element<'_, Message> = if edit_root_id.is_some() {
        column![
            h_divider(),
            row![
                button(text(app.i18n.text(TextKey::RootDeactivateButton)).size(FONT_CAPTION))
                    .on_press(Message::DeactivateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                button(text(app.i18n.text(TextKey::RootReactivateButton)).size(FONT_CAPTION))
                    .on_press(Message::ReactivateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                button(text(app.i18n.text(TextKey::RootRemoveButton)).size(FONT_CAPTION))
                    .on_press(Message::RemoveRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            ]
            .spacing(SPACE_XS),
        ]
        .into()
    } else {
        column![].into()
    };

    let add_another_button: Element<'_, Message> = if is_add_mode {
        button(text(app.i18n.text(TextKey::LibrarySaveAndAddAnotherButton)).size(FONT_BODY))
            .on_press(Message::SaveLibraryAndAddAnother)
            .style(action_button_style)
            .padding([SPACE_SM as u16, SPACE_MD as u16])
            .into()
    } else {
        column![].into()
    };

    let dialog_content = column![
        row![
            text(title).size(FONT_TITLE).color(TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
                .on_press(Message::CloseLibraryDialog)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .align_y(iced::Alignment::Center),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::LibraryPathLabel)),
            row![
                button(text(app.i18n.text(TextKey::BrowseFolderButton)).size(FONT_BODY))
                    .on_press(Message::LibraryDialogBrowseFolder)
                    .style(primary_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(path_toggle_label).size(FONT_CAPTION))
                    .on_press(Message::ToggleLibraryDialogManualPath)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            ]
            .spacing(SPACE_XS),
            path_field,
            text(app.library_dialog_path_input.as_str())
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        ]
        .spacing(SPACE_SM),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::LibraryDisplayNameLabel)),
            text_input(
                app.i18n.text(TextKey::DisplayNamePlaceholder),
                &app.library_dialog_display_name_input,
            )
            .on_input(Message::LibraryDialogDisplayNameChanged)
            .style(field_input_style),
        ]
        .spacing(SPACE_SM),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::LibraryTagsLabel)),
            text_input(
                app.i18n.text(TextKey::RootTagInputPlaceholder),
                &app.library_dialog_tag_input,
            )
            .on_input(Message::LibraryDialogTagInputChanged)
            .on_submit(if app.library_dialog_editing_tag.is_some() {
                Message::LibraryDialogApplyTagEdit
            } else {
                Message::LibraryDialogAddAppTag
            })
            .style(field_input_style),
            library_tag_actions,
            tags_list,
        ]
        .spacing(SPACE_SM),
        lifecycle_actions,
        h_divider(),
        row![
            button(text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_BODY))
                .on_press(Message::SaveLibraryDialog)
                .style(primary_button_style)
                .padding([SPACE_SM as u16, SPACE_MD as u16]),
            add_another_button,
        ]
        .spacing(SPACE_XS),
    ]
    .spacing(SPACE_LG);

    let dialog = container(
        scrollable(dialog_content)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default().spacing(PANEL_SCROLLBAR_SPACING),
            ))
            .height(Length::Shrink)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .max_width(560.0)
    .max_height(640.0)
    .padding(SPACE_LG as u16)
    .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

fn render_library_statistics_dialog(app: &Librapix) -> Element<'_, Message> {
    let root = app.library_stats_root_id.and_then(|root_id| {
        app.state
            .library_roots
            .iter()
            .find(|root| root.id == root_id)
    });

    let title_suffix = root
        .map(display_name_for_root)
        .unwrap_or_else(|| app.i18n.text(TextKey::LibrarySectionLabel).to_owned());
    let dialog_title = format!(
        "{}: {title_suffix}",
        app.i18n.text(TextKey::LibraryStatsDialogTitle)
    );

    let path_line: Element<'_, Message> = if let Some(root) = root {
        text(root.normalized_path.display().to_string())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        column![].into()
    };

    let stats_body: Element<'_, Message> = if let Some(stats) = &app.library_stats_record {
        column![
            section_heading(app.i18n.text(TextKey::LibraryStatsSummarySectionLabel)),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalSizeLabel),
                format::format_file_size(stats.total_size_bytes),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalMediaLabel),
                stats.total_media_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalImagesLabel),
                stats.total_images_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsTotalVideosLabel),
                stats.total_videos_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsImageSizeLabel),
                format::format_file_size(stats.total_image_size_bytes),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsVideoSizeLabel),
                format::format_file_size(stats.total_video_size_bytes),
            ),
            h_divider(),
            section_heading(app.i18n.text(TextKey::LibraryStatsIndexingSectionLabel)),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsMissingLabel),
                stats.missing_count.to_string(),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsLastIndexedLabel),
                format::format_timestamp(stats.last_indexed_unix_seconds),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsOldestFileLabel),
                format::format_timestamp(stats.oldest_modified_unix_seconds),
            ),
            render_stats_row(
                app.i18n.text(TextKey::LibraryStatsNewestFileLabel),
                format::format_timestamp(stats.newest_modified_unix_seconds),
            ),
        ]
        .spacing(SPACE_SM)
        .into()
    } else {
        text(app.i18n.text(TextKey::LibraryStatsNotAvailableLabel))
            .size(FONT_BODY)
            .color(TEXT_SECONDARY)
            .into()
    };

    let dialog_content = column![
        row![
            text(dialog_title).size(FONT_TITLE).color(TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
                .on_press(Message::CloseLibraryStatisticsDialog)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .align_y(iced::Alignment::Center),
        path_line,
        h_divider(),
        stats_body,
    ]
    .spacing(SPACE_LG);

    let dialog = container(dialog_content)
        .width(Length::Fill)
        .max_width(560.0)
        .padding(SPACE_LG as u16)
        .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

fn render_stats_row<'a>(label: &'a str, value: String) -> Element<'a, Message> {
    row![
        text(label).size(FONT_BODY).color(TEXT_SECONDARY),
        Space::new().width(Length::Fill),
        text(value).size(FONT_BODY).color(TEXT_PRIMARY),
    ]
    .align_y(iced::Alignment::Center)
    .into()
}

fn render_new_media_dialog(app: &Librapix) -> Element<'_, Message> {
    let Some(announcement) = &app.new_media_announcement else {
        return column![].into();
    };

    let more_line: Element<'_, Message> = if announcement.additional_count > 0 {
        text(format!(
            "{} {}",
            announcement.additional_count,
            app.i18n.text(TextKey::NewFileAnnouncementMoreLabel)
        ))
        .size(FONT_CAPTION)
        .color(TEXT_TERTIARY)
        .into()
    } else {
        column![].into()
    };

    let preview_fallback = announcement.preview_path.as_ref().or_else(|| {
        app.media_cache
            .get(&announcement.media_id)
            .and_then(|details| details.detail_thumbnail_path.as_ref())
    });
    let preview_presentation =
        resolve_thumbnail_presentation(app, announcement.media_id, preview_fallback);
    let preview: Element<'_, Message> = match preview_presentation.state {
        ThumbnailPresentationState::Ready => {
            let path = preview_presentation
                .resolved_path
                .expect("ready thumbnail presentation must include path");
            container(
                image(image::Handle::from_path(path))
                    .width(Length::Fill)
                    .height(Length::Fixed(220.0))
                    .content_fit(ContentFit::Contain),
            )
            .style(card_style)
            .into()
        }
        ThumbnailPresentationState::Loading => render_thumbnail_placeholder(
            app.i18n.text(TextKey::ThumbnailLoadingLabel).to_owned(),
            Some(announcement.title.as_str()),
            220.0,
            true,
        ),
        ThumbnailPresentationState::Unavailable => render_thumbnail_placeholder(
            app.i18n.text(TextKey::ThumbnailUnavailable).to_owned(),
            Some(announcement.title.as_str()),
            220.0,
            false,
        ),
    };

    let metadata_lines = column![
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsKindLabel),
            localized_media_kind_label(app.i18n, &announcement.media_kind),
        ))
        .size(FONT_CAPTION)
        .color(TEXT_SECONDARY),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsSizeLabel),
            format::format_file_size(announcement.file_size_bytes)
        ))
        .size(FONT_CAPTION)
        .color(TEXT_SECONDARY),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsModifiedLabel),
            format::format_timestamp(announcement.modified_unix_seconds)
        ))
        .size(FONT_CAPTION)
        .color(TEXT_SECONDARY),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsDimensionsLabel),
            format::format_dimensions(announcement.width_px, announcement.height_px),
        ))
        .size(FONT_CAPTION)
        .color(TEXT_SECONDARY),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsPathLabel),
            announcement.absolute_path.display()
        ))
        .size(FONT_CAPTION)
        .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_2XS);

    let primary_padding = [SPACE_XS as u16, SPACE_MD as u16];
    let subtle_padding = [SPACE_XS as u16, SPACE_MD as u16];
    let actions = row![
        button(
            row![
                image(assets::icon_gallery())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::MediaSelectButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::SelectMedia(announcement.media_id))
        .width(Length::Fill)
        .style(subtle_button_style)
        .padding(subtle_padding),
        button(
            row![
                image(assets::icon_open())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::OpenMediaById(announcement.media_id))
        .width(Length::Fill)
        .style(action_button_style)
        .padding(primary_padding),
        button(
            row![
                image(assets::icon_copy_file())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsCopyFileButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::CopyMediaFileById(announcement.media_id))
        .width(Length::Fill)
        .style(action_button_style)
        .padding(primary_padding),
        button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
            .on_press(Message::DismissNewMediaAnnouncement)
            .width(Length::Fill)
            .style(subtle_button_style)
            .padding(subtle_padding),
    ]
    .spacing(SPACE_XS);

    let dialog_content = column![
        text(app.i18n.text(TextKey::NewFileAnnouncementTitle))
            .size(FONT_CAPTION)
            .color(ACCENT),
        text(announcement.title.as_str())
            .size(FONT_SUBTITLE)
            .color(TEXT_PRIMARY),
        text(announcement.metadata_line.as_str())
            .size(FONT_CAPTION)
            .color(TEXT_SECONDARY),
        more_line,
        h_divider(),
        preview,
        metadata_lines,
        actions,
    ]
    .spacing(SPACE_SM);

    let dialog = container(dialog_content)
        .width(Length::Fill)
        .max_width(640.0)
        .padding(SPACE_LG as u16)
        .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

#[derive(Debug, Clone, PartialEq)]
struct TimelineYearMarker {
    label: String,
    group_index: usize,
    normalized_position: f32,
}

fn render_timeline_year_marker_track(markers: &[TimelineYearMarker]) -> Element<'static, Message> {
    if markers.is_empty() {
        return Space::new().width(Length::Fixed(0.0)).into();
    }

    let mut track = column![].height(Length::Fill);
    let mut previous_position = 0.0f32;

    for marker in markers {
        let current = marker.normalized_position.clamp(0.0, 1.0);
        let spacer = ((current - previous_position).max(0.0) * 1000.0).round() as u16;
        if spacer > 0 {
            track = track.push(Space::new().height(Length::FillPortion(spacer)));
        }

        track = track.push(
            button(text(marker.label.clone()).size(FONT_CAPTION))
                .on_press(Message::JumpToTimelineAnchor(marker.group_index))
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
        previous_position = current;
    }

    let trailing = ((1.0 - previous_position).max(0.0) * 1000.0).round() as u16;
    if trailing > 0 {
        track = track.push(Space::new().height(Length::FillPortion(trailing)));
    }

    track.into()
}

fn timeline_year_markers(anchors: &[TimelineAnchor]) -> Vec<TimelineYearMarker> {
    let mut markers = Vec::new();
    let mut last_year: Option<i32> = None;

    for anchor in anchors {
        if let Some(year) = anchor.year
            && last_year != Some(year)
        {
            markers.push(TimelineYearMarker {
                label: year.to_string(),
                group_index: anchor.group_index,
                normalized_position: anchor.normalized_position.clamp(0.0, 1.0),
            });
            last_year = Some(year);
        }
    }

    if markers.len() <= SCRUBBER_YEAR_MARKER_LIMIT {
        return markers;
    }

    let step = (markers.len() as f32 / SCRUBBER_YEAR_MARKER_LIMIT as f32).ceil() as usize;
    let mut reduced = markers.iter().cloned().step_by(step).collect::<Vec<_>>();
    if let Some(last) = markers.last().cloned()
        && reduced.last().map(|marker| marker.group_index) != Some(last.group_index)
    {
        reduced.push(last);
    }
    reduced
}

fn scroll_task_to_timeline_position(app: &Librapix, normalized: f32) -> Task<Message> {
    let clamped = normalized.clamp(0.0, 1.0);
    let id = Id::new(MEDIA_SCROLLABLE_ID);

    if app.timeline_scroll_max_y > 0.0 {
        let target_y = app.timeline_scroll_max_y * clamped;
        operation::scroll_to(
            id,
            operation::AbsoluteOffset {
                x: Some(0.0),
                y: Some(target_y),
            },
        )
    } else {
        operation::snap_to(id, operation::RelativeOffset { x: 0.0, y: clamped })
    }
}

fn apply_timeline_scrub(
    app: &mut Librapix,
    normalized_value: f32,
    interactive: bool,
) -> Task<Message> {
    if app.timeline_anchors.is_empty() {
        return Task::none();
    }

    let clamped = normalized_value.clamp(0.0, 1.0);
    app.timeline_scrub_value = clamped;
    app.timeline_scrubbing = interactive;
    app.timeline_scrub_anchor_index = scrub_value_to_anchor_index(&app.timeline_anchors, clamped);

    if !matches!(app.state.active_route, Route::Timeline) {
        return Task::none();
    }

    scroll_task_to_timeline_position(app, clamped)
}

fn jump_to_timeline_anchor(app: &mut Librapix, group_index: usize) -> Task<Message> {
    let Some((anchor_index, anchor)) = app
        .timeline_anchors
        .iter()
        .enumerate()
        .find(|(_, anchor)| anchor.group_index == group_index)
    else {
        return Task::none();
    };

    app.timeline_scrubbing = false;
    app.timeline_scrub_anchor_index = Some(anchor_index);
    app.timeline_scrub_value = anchor.normalized_position.clamp(0.0, 1.0);

    if !matches!(app.state.active_route, Route::Timeline) {
        return Task::none();
    }

    scroll_task_to_timeline_position(app, app.timeline_scrub_value)
}

fn sync_timeline_scrub_from_viewport(app: &mut Librapix, absolute_y: f32, max_y: f32) {
    if !matches!(app.state.active_route, Route::Timeline) {
        return;
    }

    app.timeline_scroll_max_y = max_y.max(0.0);
    if app.timeline_scrubbing {
        return;
    }

    let normalized = if app.timeline_scroll_max_y > 0.0 {
        (absolute_y / app.timeline_scroll_max_y).clamp(0.0, 1.0)
    } else {
        0.0
    };
    app.timeline_scrub_anchor_index =
        scrub_value_to_anchor_index(&app.timeline_anchors, normalized);
    app.timeline_scrub_value = normalized;
}

fn sync_timeline_scrub_selection(app: &mut Librapix, preferred_value: f32) {
    if app.timeline_anchors.is_empty() {
        app.timeline_scrub_value = 0.0;
        app.timeline_scrub_anchor_index = None;
        app.timeline_scrubbing = false;
        app.timeline_scroll_max_y = 0.0;
        return;
    }

    let clamped = preferred_value.clamp(0.0, 1.0);
    app.timeline_scrub_anchor_index = scrub_value_to_anchor_index(&app.timeline_anchors, clamped);
    app.timeline_scrub_value = clamped;
    app.timeline_scrubbing = false;
    app.timeline_scroll_max_y = 0.0;
}

fn scrub_value_to_anchor_index(anchors: &[TimelineAnchor], normalized: f32) -> Option<usize> {
    if anchors.is_empty() {
        return None;
    }

    let clamped = normalized.clamp(0.0, 1.0);
    let mut best_index = 0usize;
    let mut best_distance = f32::MAX;

    for (index, anchor) in anchors.iter().enumerate() {
        let distance = (anchor.normalized_position.clamp(0.0, 1.0) - clamped).abs();
        if distance < best_distance {
            best_distance = distance;
            best_index = index;
        }
    }

    Some(best_index)
}

fn render_details_panel(app: &Librapix) -> Element<'_, Message> {
    if app.state.selected_media_id.is_none() {
        return container(
            column![
                text(app.i18n.text(TextKey::SelectPhotoTitle))
                    .size(FONT_SUBTITLE)
                    .color(TEXT_SECONDARY),
                text(app.i18n.text(TextKey::SelectPhotoSubtitle))
                    .size(FONT_BODY)
                    .color(TEXT_TERTIARY),
            ]
            .spacing(SPACE_SM)
            .padding(SPACE_2XL as u16),
        )
        .width(Length::Fill)
        .style(empty_state_style)
        .into();
    }

    let preview: Element<'_, Message> = if let Some(path) = &app.details_preview_path {
        container(
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .content_fit(ContentFit::Contain),
        )
        .width(Length::Fill)
        .style(card_style)
        .into()
    } else {
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fixed(160.0))
            .style(thumb_placeholder_style)
            .into()
    };

    let metadata = if app.details_lines.is_empty() {
        column![
            text(app.i18n.text(TextKey::DetailsNoSelectionLabel))
                .size(FONT_BODY)
                .color(TEXT_TERTIARY)
        ]
    } else {
        app.details_lines
            .iter()
            .fold(column![].spacing(SPACE_XS), |col, line| {
                col.push(text(line.clone()).size(FONT_BODY).color(TEXT_SECONDARY))
            })
    };

    let details_tag_input_actions: Element<'_, Message> = if app.details_editing_tag.is_some() {
        row![
            button(text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_BODY))
                .on_press(Message::DetailsApplyTagEdit)
                .style(primary_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
                .on_press(Message::DetailsCancelTagEdit)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        row![
            button(text(app.i18n.text(TextKey::DetailsAttachTagButton)).size(FONT_BODY))
                .on_press(Message::AttachAppTag)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::DetailsAttachGameTagButton)).size(FONT_BODY))
                .on_press(Message::AttachGameTag)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .spacing(SPACE_XS)
        .into()
    };

    let inherited_tag_chips: Vec<&DetailsTagChip> = app
        .details_tags
        .iter()
        .filter(|tag| tag.inherited)
        .collect();
    let manual_tag_chips: Vec<&DetailsTagChip> = app
        .details_tags
        .iter()
        .filter(|tag| !tag.inherited)
        .collect();
    let details_tag_list: Element<'_, Message> = if app.details_tags.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY)
            .into()
    } else {
        let inherited_section: Element<'_, Message> = if inherited_tag_chips.is_empty() {
            Space::new().height(Length::Shrink).into()
        } else {
            let list = inherited_tag_chips
                .iter()
                .fold(column![].spacing(SPACE_XS), |col, tag| {
                    col.push(render_management_chip(
                        &tag.name,
                        &tag.name,
                        Some(app.i18n.text(TextKey::InheritedTagLabel).to_owned()),
                        None,
                        None,
                        Some(Message::DetachTagByName(tag.name.clone())),
                    ))
                });
            column![
                section_heading(app.i18n.text(TextKey::InheritedTagLabel)),
                list,
            ]
            .spacing(SPACE_XS)
            .into()
        };
        let manual_section =
            manual_tag_chips
                .iter()
                .fold(column![].spacing(SPACE_XS), |col, tag| {
                    col.push(render_management_chip(
                        &tag.name,
                        &tag.name,
                        Some(tag.kind.as_str().to_owned()),
                        None,
                        Some((
                            app.i18n.text(TextKey::RootEditButton).to_owned(),
                            Message::DetailsStartEditTag(tag.name.clone()),
                        )),
                        Some(Message::DetachTagByName(tag.name.clone())),
                    ))
                });
        column![inherited_section, manual_section]
            .spacing(SPACE_SM)
            .into()
    };

    column![
        preview,
        text(app.details_title.clone())
            .size(FONT_SUBTITLE)
            .color(TEXT_PRIMARY),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::FileInfoLabel)),
            metadata,
        ]
        .spacing(SPACE_SM),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::DetailsTagsSectionLabel)),
            text_input(
                app.i18n.text(TextKey::DetailsTagInputLabel),
                &app.details_tag_input
            )
            .on_input(Message::DetailsTagInputChanged)
            .on_submit(if app.details_editing_tag.is_some() {
                Message::DetailsApplyTagEdit
            } else {
                Message::AttachAppTag
            })
            .style(field_input_style),
            details_tag_input_actions,
            details_tag_list,
        ]
        .spacing(SPACE_SM),
        h_divider(),
        column![
            section_heading(app.i18n.text(TextKey::DetailsActionsSectionLabel)),
            render_details_actions(app),
            text(app.i18n.text(TextKey::DetailsCopyShortcutHint))
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        ]
        .spacing(SPACE_SM),
        text(app.details_action_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
        text(app.i18n.text(TextKey::NonDestructiveNotice))
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_LG)
    .into()
}

fn render_details_actions(app: &Librapix) -> Element<'_, Message> {
    responsive(move |size: Size| {
        let open = button(
            row![
                image(assets::icon_open())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::OpenSelectedFile)
        .width(Length::Fill)
        .style(action_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);
        let open_folder = button(
            row![
                image(assets::icon_show_in_folder())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsOpenFolderButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::OpenSelectedFolder)
        .width(Length::Fill)
        .style(action_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);
        let copy_file = button(
            row![
                image(assets::icon_copy_file())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsCopyFileButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::CopySelectedFile)
        .width(Length::Fill)
        .style(action_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);
        let copy_path = button(
            row![
                image(assets::icon_copy_path())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsCopyPathButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::CopySelectedPath)
        .width(Length::Fill)
        .style(action_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);

        if size.width < 220.0 {
            column![open, open_folder, copy_file, copy_path]
                .spacing(SPACE_XS)
                .into()
        } else if size.width < 420.0 {
            column![
                row![open, open_folder].spacing(SPACE_XS),
                row![copy_file, copy_path].spacing(SPACE_XS),
            ]
            .spacing(SPACE_XS)
            .into()
        } else {
            row![open, open_folder, copy_file, copy_path]
                .spacing(SPACE_XS)
                .into()
        }
    })
    .into()
}

struct BootstrapRuntime {
    locale: Locale,
    theme_preference: ThemePreference,
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    config_file: PathBuf,
    roots: Vec<LibraryRootView>,
}

fn bootstrap_runtime() -> BootstrapRuntime {
    let mut runtime = BootstrapRuntime {
        locale: Locale::EnUs,
        theme_preference: ThemePreference::System,
        database_file: PathBuf::from("librapix.db"),
        thumbnails_dir: PathBuf::from("thumbnails"),
        config_file: PathBuf::new(),
        roots: Vec::new(),
    };

    let loaded = match load_or_create() {
        Ok(config) => config,
        Err(_) => return runtime,
    };

    runtime.locale = match loaded.config.locale {
        LocalePreference::EnUs => Locale::EnUs,
    };
    runtime.theme_preference = loaded.config.theme.clone();

    let database_file = loaded
        .config
        .path_overrides
        .database_file
        .clone()
        .unwrap_or(loaded.paths.database_file);
    runtime.database_file = database_file.clone();
    runtime.thumbnails_dir = loaded
        .config
        .path_overrides
        .thumbnails_dir
        .clone()
        .unwrap_or(loaded.paths.thumbnails_dir);
    runtime.config_file = loaded.paths.config_file.clone();

    let storage = match Storage::open(&database_file) {
        Ok(storage) => storage,
        Err(_) => return runtime,
    };

    let existing_roots = storage.list_source_roots().unwrap_or_default();
    if existing_roots.is_empty() {
        for source in &loaded.config.library_source_roots {
            let _ = storage.upsert_source_root(&source.path);
        }
    }
    let _ = storage.ensure_default_ignore_rules();
    let _ = storage.reconcile_source_root_availability();

    runtime.roots = storage
        .list_source_roots()
        .map_or_else(|_| Vec::new(), map_roots_from_storage);
    runtime
}

fn sync_roots_to_config(database_file: &Path, config_file: &Path) {
    let Ok(mut config) = load_from_path(config_file) else {
        return;
    };
    let Ok(storage) = Storage::open(database_file) else {
        return;
    };
    let Ok(roots) = storage.list_source_roots() else {
        return;
    };
    config.library_source_roots = roots
        .into_iter()
        .map(|root| librapix_config::LibrarySourceRoot {
            path: root.normalized_path,
        })
        .collect();
    let _ = save_to_path(config_file, &config);
}

fn normalized_input_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let cwd = std::env::current_dir().ok()?;
    Some(lexical_normalize_path(&PathBuf::from(trimmed), &cwd))
}

fn display_name_for_root(root: &LibraryRootView) -> String {
    root.display_name
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            root.normalized_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| root.normalized_path.display().to_string())
        })
}

fn open_add_library_dialog(app: &mut Librapix) {
    app.library_dialog_mode = LibraryDialogMode::Add;
    app.library_dialog_open = true;
    app.library_stats_dialog_open = false;
    app.library_dialog_manual_path_open = false;
    app.library_dialog_path_input.clear();
    app.library_dialog_display_name_input.clear();
    app.library_dialog_tag_input.clear();
    app.library_dialog_tags.clear();
    app.library_dialog_editing_tag = None;
}

fn open_edit_library_dialog(app: &mut Librapix, root_id: i64) {
    app.state.apply(AppMessage::SetSelectedRoot);
    app.state.set_selected_root(Some(root_id));
    app.library_dialog_mode = LibraryDialogMode::Edit(root_id);
    app.library_dialog_open = true;
    app.library_stats_dialog_open = false;
    app.library_dialog_manual_path_open = false;

    if let Some(root) = app.state.library_roots.iter().find(|r| r.id == root_id) {
        app.library_dialog_path_input = root.normalized_path.display().to_string();
        app.library_dialog_display_name_input = root.display_name.clone().unwrap_or_default();
    }

    app.library_dialog_tags = with_storage(&app.runtime, |storage| {
        storage.list_source_root_tags(root_id).map(|rows| {
            rows.into_iter()
                .map(|row| (row.tag_name, row.tag_kind))
                .collect::<Vec<_>>()
        })
    })
    .unwrap_or_default();
    app.library_dialog_tag_input.clear();
    app.library_dialog_editing_tag = None;
}

fn open_library_statistics_dialog(app: &mut Librapix, root_id: i64) {
    app.state.apply(AppMessage::SetSelectedRoot);
    app.state.set_selected_root(Some(root_id));
    app.library_stats_root_id = Some(root_id);
    app.library_stats_record = with_storage(&app.runtime, |storage| {
        storage.get_source_root_statistics(root_id)
    })
    .ok()
    .flatten();
    app.library_stats_dialog_open = true;
}

fn add_library_dialog_tag(app: &mut Librapix, kind: TagKind) {
    let tag = app.library_dialog_tag_input.trim().to_owned();
    if tag.is_empty() {
        return;
    }
    if app
        .library_dialog_tags
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case(&tag))
    {
        app.library_dialog_tag_input.clear();
        return;
    }
    app.library_dialog_tags.push((tag, kind));
    app.library_dialog_editing_tag = None;
    app.library_dialog_tag_input.clear();
}

fn start_library_dialog_tag_edit(app: &mut Librapix, tag_name: &str) {
    if let Some((name, _)) = app
        .library_dialog_tags
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(tag_name))
    {
        app.library_dialog_tag_input = name.clone();
        app.library_dialog_editing_tag = Some(name.clone());
    }
}

fn apply_library_dialog_tag_edit(app: &mut Librapix) {
    let Some(original) = app.library_dialog_editing_tag.clone() else {
        return;
    };
    let edited = app.library_dialog_tag_input.trim().to_owned();
    if edited.is_empty() {
        return;
    }
    if app.library_dialog_tags.iter().any(|(name, _)| {
        !name.eq_ignore_ascii_case(&original) && name.eq_ignore_ascii_case(&edited)
    }) {
        return;
    }
    if let Some((name, _)) = app
        .library_dialog_tags
        .iter_mut()
        .find(|(name, _)| name.eq_ignore_ascii_case(&original))
    {
        *name = edited;
    }
    app.library_dialog_editing_tag = None;
    app.library_dialog_tag_input.clear();
}

fn sync_root_tags(
    storage: &mut Storage,
    root_id: i64,
    desired_tags: &[(String, TagKind)],
) -> Result<(), librapix_storage::StorageError> {
    let existing = storage.list_source_root_tags(root_id)?;

    for row in existing {
        let still_present = desired_tags
            .iter()
            .any(|(name, kind)| name.eq_ignore_ascii_case(&row.tag_name) && *kind == row.tag_kind);
        if !still_present {
            storage.remove_source_root_tag(root_id, &row.tag_name)?;
        }
    }

    for (tag_name, kind) in desired_tags {
        storage.upsert_source_root_tag(root_id, tag_name, *kind)?;
    }

    Ok(())
}

fn save_library_dialog(app: &mut Librapix, keep_open_for_add: bool) -> Option<Task<Message>> {
    let Some(path) = normalized_input_path(&app.library_dialog_path_input) else {
        app.root_status = app.i18n.text(TextKey::ErrorInvalidRootPathLabel).to_owned();
        return None;
    };
    let display_name = app.library_dialog_display_name_input.trim().to_owned();
    let desired_tags = app.library_dialog_tags.clone();
    let mode = app.library_dialog_mode;

    let root_id_result = with_storage(&app.runtime, |storage| {
        let root_id = match mode {
            LibraryDialogMode::Add => {
                storage.upsert_source_root(&path)?;
                let roots = storage.list_source_roots()?;
                let Some(root_id) = roots
                    .iter()
                    .find(|root| root.normalized_path == path)
                    .map(|root| root.id)
                else {
                    return Err(librapix_storage::StorageError::InvalidSourcePath(
                        path.clone(),
                    ));
                };
                root_id
            }
            LibraryDialogMode::Edit(root_id) => {
                storage.update_source_root_path(root_id, &path)?;
                root_id
            }
        };

        storage.update_source_root_display_name(root_id, &display_name)?;
        sync_root_tags(storage, root_id, &desired_tags)?;
        Ok(root_id)
    });

    let root_id = match root_id_result {
        Ok(id) => id,
        Err(_) => {
            app.root_status = app.i18n.text(TextKey::ErrorInvalidRootPathLabel).to_owned();
            return None;
        }
    };

    sync_roots_to_config(&app.runtime.database_file, &app.runtime.config_file);
    refresh_roots(app);
    app.state.apply(AppMessage::SetSelectedRoot);
    app.state.set_selected_root(Some(root_id));

    app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
    if matches!(mode, LibraryDialogMode::Add)
        && app
            .filter_source_root_id
            .is_some_and(|active_filter| active_filter != root_id)
    {
        // Keep "All libraries" semantics explicit when the current filter would hide the new root.
        app.filter_source_root_id = None;
        let message = app
            .i18n
            .text(TextKey::LibraryFilterAdjustedLabel)
            .to_owned();
        app.browse_status = message.clone();
        app.root_status = message;
    }

    if matches!(mode, LibraryDialogMode::Add) && keep_open_for_add {
        app.library_dialog_mode = LibraryDialogMode::Add;
        app.library_dialog_path_input.clear();
        app.library_dialog_display_name_input.clear();
        app.library_dialog_tag_input.clear();
        app.library_dialog_tags.clear();
        app.library_dialog_editing_tag = None;
    } else {
        app.library_dialog_open = false;
    }

    Some(request_reconcile(app, BackgroundWorkReason::UserOrSystem))
}

fn refresh_roots(app: &mut Librapix) {
    let roots = with_storage(&app.runtime, |storage| {
        storage.reconcile_source_root_availability()?;
        storage.list_source_roots()
    })
    .map(map_roots_from_storage)
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(roots);
    ensure_valid_library_filter(app);
    refresh_ignore_rules(app);
}

fn refresh_ignore_rules(app: &mut Librapix) {
    app.ignore_rules = with_storage(&app.runtime, |storage| storage.list_ignore_rules("global"))
        .unwrap_or_default();
}

fn add_ignore_rule(app: &mut Librapix) {
    let pattern = app.ignore_rule_input.trim();
    if pattern.is_empty() {
        return;
    }
    let _ = with_storage(&app.runtime, |storage| {
        storage.upsert_ignore_rule("global", pattern, true)
    });
    app.ignore_rule_input.clear();
    app.ignore_rule_editing_id = None;
    refresh_ignore_rules(app);
}

fn toggle_ignore_rule(app: &mut Librapix, rule_id: i64) {
    let Some(rule) = app.ignore_rules.iter().find(|rule| rule.id == rule_id) else {
        return;
    };
    let _ = with_storage(&app.runtime, |storage| {
        storage.upsert_ignore_rule(&rule.scope, &rule.pattern, !rule.is_enabled)
    });
    refresh_ignore_rules(app);
}

fn remove_ignore_rule(app: &mut Librapix, rule_id: i64) {
    let _ = with_storage(&app.runtime, |storage| {
        storage.delete_ignore_rule_by_id(rule_id)
    });
    if app.ignore_rule_editing_id == Some(rule_id) {
        app.ignore_rule_editing_id = None;
        app.ignore_rule_input.clear();
    }
    refresh_ignore_rules(app);
}

fn start_ignore_rule_edit(app: &mut Librapix, rule_id: i64) {
    if let Some(rule) = app.ignore_rules.iter().find(|rule| rule.id == rule_id) {
        app.ignore_rule_input = rule.pattern.clone();
        app.ignore_rule_editing_id = Some(rule.id);
    }
}

fn apply_ignore_rule_edit(app: &mut Librapix) {
    let Some(rule_id) = app.ignore_rule_editing_id else {
        return;
    };
    let pattern = app.ignore_rule_input.trim().to_owned();
    if pattern.is_empty() {
        return;
    }
    let Some(rule) = app.ignore_rules.iter().find(|rule| rule.id == rule_id) else {
        return;
    };
    if rule.pattern.eq_ignore_ascii_case(&pattern) {
        app.ignore_rule_editing_id = None;
        app.ignore_rule_input.clear();
        return;
    }

    let scope = rule.scope.clone();
    let is_enabled = rule.is_enabled;
    let _ = with_storage(&app.runtime, |storage| {
        storage.upsert_ignore_rule(&scope, &pattern, is_enabled)?;
        storage.delete_ignore_rule_by_id(rule_id)?;
        Ok(())
    });
    app.ignore_rule_editing_id = None;
    app.ignore_rule_input.clear();
    refresh_ignore_rules(app);
}

fn load_media_details(app: &mut Librapix) {
    app.details_editing_tag = None;
    app.details_tag_input.clear();
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_tags.clear();
        return;
    };
    let details = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    if let Some(details) = details {
        app.details_title = details
            .absolute_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| details.absolute_path.display().to_string());
        app.details_preview_path =
            resolve_thumbnail(&app.runtime.thumbnails_dir, &details, DETAIL_THUMB_SIZE);
        app.details_lines = vec![
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsKindLabel),
                details.media_kind
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsSizeLabel),
                format::format_file_size(details.file_size_bytes)
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsModifiedLabel),
                format::format_timestamp(details.modified_unix_seconds)
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsDimensionsLabel),
                format::format_dimensions(details.width_px, details.height_px)
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsPathLabel),
                details.absolute_path.display()
            ),
        ];
        refresh_details_tags(app, media_id);
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
    } else {
        app.details_lines.clear();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_tags.clear();
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
    }
}

fn attach_tag_to_selected_media(app: &mut Librapix, kind: TagKind) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let tag = app.details_tag_input.trim();
    if tag.is_empty() {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    }
    match with_storage(&app.runtime, |storage| {
        storage.attach_tag_name_to_media(media_id, tag, kind)
    }) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
            app.details_tag_input.clear();
            app.details_editing_tag = None;
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
}

fn detach_tag_from_selected_media(app: &mut Librapix, tag_name: &str) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let tag = tag_name.trim();
    if tag.is_empty() {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    }
    match with_storage(&app.runtime, |storage| {
        storage.detach_tag_name_from_media(media_id, tag)
    }) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
            if app
                .details_editing_tag
                .as_ref()
                .is_some_and(|editing| editing.eq_ignore_ascii_case(tag))
            {
                app.details_editing_tag = None;
                app.details_tag_input.clear();
            }
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
}

fn apply_details_tag_edit(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    let Some(original_name) = app.details_editing_tag.clone() else {
        return;
    };
    let edited_name = app.details_tag_input.trim().to_owned();
    if edited_name.is_empty() {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    }
    let Some(existing) = app
        .details_tags
        .iter()
        .find(|tag| tag.name.eq_ignore_ascii_case(&original_name))
    else {
        return;
    };
    if existing.name.eq_ignore_ascii_case(&edited_name) {
        app.details_editing_tag = None;
        app.details_tag_input.clear();
        return;
    }
    let kind = existing.kind;
    let result = with_storage(&app.runtime, |storage| {
        storage.detach_tag_name_from_media(media_id, &original_name)?;
        storage.attach_tag_name_to_media(media_id, &edited_name, kind)?;
        Ok(())
    });
    match result {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
            app.details_editing_tag = None;
            app.details_tag_input.clear();
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
}

fn refresh_details_tags(app: &mut Librapix, media_id: i64) {
    let media_row = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    let Some(media_row) = media_row else {
        app.details_tags.clear();
        return;
    };
    let root_tags = with_storage(&app.runtime, |storage| {
        storage.list_source_root_tags(media_row.source_root_id)
    })
    .unwrap_or_default();
    let media_tags =
        with_storage(&app.runtime, |storage| storage.list_media_tags(media_id)).unwrap_or_default();

    let inherited_names = root_tags
        .into_iter()
        .map(|tag| tag.tag_name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();

    let mut tags = media_tags
        .into_iter()
        .map(|tag| DetailsTagChip {
            inherited: inherited_names.contains(&tag.name.to_ascii_lowercase()),
            name: tag.name,
            kind: tag.kind,
        })
        .collect::<Vec<_>>();
    tags.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    app.details_tags = tags;
}

fn open_selected_path(app: &mut Librapix, containing_folder: bool) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    open_media_by_id(app, media_id, containing_folder);
}

fn open_media_by_id(app: &mut Librapix, media_id: i64, containing_folder: bool) {
    let Some(path) = resolve_media_path_for_action(app, media_id) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    let target = if containing_folder {
        path.parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| path.clone())
    } else {
        path
    };
    if !target.exists() {
        app.details_action_status = app.i18n.text(TextKey::ErrorUnavailableFileLabel).to_owned();
        return;
    }
    match open_with_system_default(&target) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::ErrorActionFailedLabel).to_owned()
        }
    }
}

fn copy_selected_path(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    copy_media_path_by_id(app, media_id);
}

fn copy_selected_file(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        return;
    };
    copy_media_file_by_id(app, media_id);
}

fn copy_media_path_by_id(app: &mut Librapix, media_id: i64) {
    let Some(path) = resolve_media_path_for_action(app, media_id) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    if !path.exists() {
        app.details_action_status = app.i18n.text(TextKey::ErrorUnavailableFileLabel).to_owned();
        return;
    }
    match copy_text_to_clipboard(&path.display().to_string()) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::ErrorActionFailedLabel).to_owned()
        }
    }
}

fn copy_media_file_by_id(app: &mut Librapix, media_id: i64) {
    let Some(path) = resolve_media_path_for_action(app, media_id) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    if !path.exists() {
        app.details_action_status = app.i18n.text(TextKey::ErrorUnavailableFileLabel).to_owned();
        return;
    }
    match copy_file_to_clipboard(&path) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::ErrorActionFailedLabel).to_owned()
        }
    }
}

fn resolve_media_path_for_action(app: &Librapix, media_id: i64) -> Option<PathBuf> {
    if let Some(cached) = app.media_cache.get(&media_id) {
        return Some(cached.absolute_path.clone());
    }
    with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten()
    .map(|row| row.absolute_path)
}

fn open_with_system_default(path: &PathBuf) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).status()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        opener::open(path).map_err(|error| std::io::Error::other(error.to_string()))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(path).status()?;
        Ok(())
    }
}

fn copy_text_to_clipboard(value: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let mut child = Command::new("clip").stdin(Stdio::piped()).spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        Ok(())
    }
}

fn copy_file_to_clipboard(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("osascript")
            .args([
                "-e",
                "on run argv",
                "-e",
                "set fileRef to POSIX file (item 1 of argv)",
                "-e",
                "set the clipboard to fileRef",
                "-e",
                "end run",
            ])
            .arg(path)
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other(
                "osascript failed to set file clipboard",
            ))
        }
    }
    #[cfg(target_os = "windows")]
    {
        copy_file_to_clipboard_windows(path)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let uri = path_to_file_uri(path);
        let payload = format!("copy\n{uri}\n");

        let mut child = Command::new("xclip")
            .args([
                "-selection",
                "clipboard",
                "-t",
                "x-special/gnome-copied-files",
            ])
            .stdin(Stdio::piped())
            .spawn()?;
        if let Some(stdin) = &mut child.stdin {
            stdin.write_all(payload.as_bytes())?;
        }
        let status = child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(std::io::Error::other(
                "xclip failed to copy file payload to clipboard",
            ))
        }
    }
}

#[cfg(target_os = "windows")]
const WINDOWS_CF_HDROP_FORMAT: u32 = 15;
#[cfg(target_os = "windows")]
const WINDOWS_CLIPBOARD_OPEN_RETRIES: usize = 8;
#[cfg(target_os = "windows")]
const WINDOWS_CLIPBOARD_RETRY_DELAY_MS: u64 = 15;

#[cfg(any(test, target_os = "windows"))]
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct WindowsDropFilesHeader {
    p_files: u32,
    x: i32,
    y: i32,
    f_nc: i32,
    f_wide: i32,
}

#[cfg(any(test, target_os = "windows"))]
fn build_windows_file_drop_payload(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Windows file-drop clipboard payload requires an absolute path",
        ));
    }

    let mut encoded_path = windows_path_to_utf16(path);
    if encoded_path.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Windows file-drop clipboard payload cannot be empty",
        ));
    }

    // CF_HDROP expects a UTF-16 multi-string list (one or more paths) terminated by a double NUL.
    encoded_path.push(0);
    encoded_path.push(0);

    let header = WindowsDropFilesHeader {
        p_files: std::mem::size_of::<WindowsDropFilesHeader>() as u32,
        x: 0,
        y: 0,
        f_nc: 0,
        f_wide: 1,
    };

    let byte_len = std::mem::size_of::<WindowsDropFilesHeader>()
        + encoded_path.len() * std::mem::size_of::<u16>();
    let mut payload = vec![0u8; byte_len];
    let header_size = std::mem::size_of::<WindowsDropFilesHeader>();

    // SAFETY: payload is allocated with sufficient size for the header and UTF-16 path data.
    unsafe {
        std::ptr::write_unaligned(
            payload.as_mut_ptr().cast::<WindowsDropFilesHeader>(),
            header,
        );
        std::ptr::copy_nonoverlapping(
            encoded_path.as_ptr().cast::<u8>(),
            payload.as_mut_ptr().add(header_size),
            encoded_path.len() * std::mem::size_of::<u16>(),
        );
    }

    Ok(payload)
}

#[cfg(any(test, target_os = "windows"))]
fn windows_path_to_utf16(path: &Path) -> Vec<u16> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;

        path.as_os_str().encode_wide().collect()
    }
    #[cfg(not(target_os = "windows"))]
    {
        path.to_string_lossy()
            .replace('/', "\\")
            .encode_utf16()
            .collect()
    }
}

#[cfg(target_os = "windows")]
fn copy_file_to_clipboard_windows(path: &Path) -> Result<(), std::io::Error> {
    use windows_sys::Win32::System::DataExchange::{EmptyClipboard, SetClipboardData};
    use windows_sys::Win32::System::Memory::{
        GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock,
    };

    let payload = build_windows_file_drop_payload(path)?;

    // SAFETY: Win32 clipboard and global-memory APIs are called with documented ownership flow:
    // GlobalAlloc -> GlobalLock/write -> GlobalUnlock -> SetClipboardData transfer ownership.
    unsafe {
        let handle = GlobalAlloc(GMEM_MOVEABLE, payload.len());
        if handle.is_null() {
            return Err(windows_clipboard_error("GlobalAlloc failed"));
        }
        let mut memory = WindowsGlobalMemory(handle);

        let locked = GlobalLock(memory.0);
        if locked.is_null() {
            return Err(windows_clipboard_error("GlobalLock failed"));
        }

        std::ptr::copy_nonoverlapping(payload.as_ptr(), locked.cast::<u8>(), payload.len());

        if GlobalUnlock(memory.0) == 0 {
            let unlock_error = windows_sys::Win32::Foundation::GetLastError();
            if unlock_error != 0 {
                return Err(windows_clipboard_error("GlobalUnlock failed"));
            }
        }

        open_windows_clipboard_with_retry()?;
        let _clipboard_guard = WindowsClipboardGuard;

        if EmptyClipboard() == 0 {
            return Err(windows_clipboard_error("EmptyClipboard failed"));
        }

        let set_result = SetClipboardData(WINDOWS_CF_HDROP_FORMAT, memory.0);
        if set_result.is_null() {
            return Err(windows_clipboard_error("SetClipboardData(CF_HDROP) failed"));
        }

        // Ownership is transferred to the clipboard after successful SetClipboardData.
        memory.release_to_clipboard();
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn open_windows_clipboard_with_retry() -> Result<(), std::io::Error> {
    use windows_sys::Win32::System::DataExchange::OpenClipboard;

    for _ in 0..WINDOWS_CLIPBOARD_OPEN_RETRIES {
        // SAFETY: Null owner handle is allowed when opening the clipboard for the current task.
        let opened = unsafe { OpenClipboard(std::ptr::null_mut()) };
        if opened != 0 {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(
            WINDOWS_CLIPBOARD_RETRY_DELAY_MS,
        ));
    }

    Err(windows_clipboard_error(
        "OpenClipboard failed after retries",
    ))
}

#[cfg(target_os = "windows")]
fn windows_clipboard_error(context: &str) -> std::io::Error {
    // SAFETY: GetLastError is thread-local and requires no additional invariants.
    let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
    if code == 0 {
        std::io::Error::other(context.to_owned())
    } else {
        std::io::Error::other(format!(
            "{context}: {}",
            std::io::Error::from_raw_os_error(code as i32)
        ))
    }
}

#[cfg(target_os = "windows")]
struct WindowsClipboardGuard;

#[cfg(target_os = "windows")]
impl Drop for WindowsClipboardGuard {
    fn drop(&mut self) {
        // SAFETY: Closing the clipboard after a successful OpenClipboard is always valid.
        unsafe {
            let _ = windows_sys::Win32::System::DataExchange::CloseClipboard();
        }
    }
}

#[cfg(target_os = "windows")]
struct WindowsGlobalMemory(windows_sys::Win32::Foundation::HGLOBAL);

#[cfg(target_os = "windows")]
impl WindowsGlobalMemory {
    fn release_to_clipboard(&mut self) {
        self.0 = std::ptr::null_mut();
    }
}

#[cfg(target_os = "windows")]
impl Drop for WindowsGlobalMemory {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: We free only allocations still owned by this guard.
            unsafe {
                let _ = windows_sys::Win32::Foundation::GlobalFree(self.0);
            }
        }
    }
}

fn shortcut_action_from_keyboard_event(event: &keyboard::Event) -> Option<KeyboardShortcutAction> {
    let keyboard::Event::KeyPressed {
        key,
        modifiers,
        repeat,
        ..
    } = event
    else {
        return None;
    };

    if *repeat || !modifiers.command() {
        return None;
    }

    let is_copy_key = match key.as_ref() {
        keyboard::Key::Character(value) => value.eq_ignore_ascii_case("c"),
        keyboard::Key::Named(key::Named::Copy) => true,
        _ => false,
    };

    if !is_copy_key {
        return None;
    }

    if modifiers.shift() {
        Some(KeyboardShortcutAction::CopyPath)
    } else {
        Some(KeyboardShortcutAction::CopyFile)
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn path_to_file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    for &byte in path.to_string_lossy().as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'_' | b'.' | b'~' | b':') {
            uri.push(char::from(byte));
        } else {
            uri.push('%');
            uri.push_str(&format!("{byte:02X}"));
        }
    }
    uri
}

fn aspect_ratio_from(width: Option<u32>, height: Option<u32>) -> f32 {
    match (width, height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => w as f32 / h as f32,
        _ => 1.5,
    }
}

fn localized_media_kind_label(i18n: Translator, media_kind: &str) -> &'static str {
    if media_kind.eq_ignore_ascii_case("video") {
        i18n.text(TextKey::MediaKindVideoLabel)
    } else if media_kind.eq_ignore_ascii_case("image") {
        i18n.text(TextKey::MediaKindImageLabel)
    } else {
        i18n.text(TextKey::MediaKindUnknownLabel)
    }
}

fn build_card_metadata_line(
    i18n: Translator,
    media_kind: &str,
    file_size_bytes: Option<u64>,
    width_px: Option<u32>,
    height_px: Option<u32>,
) -> String {
    let mut parts = vec![localized_media_kind_label(i18n, media_kind).to_owned()];
    if let Some(bytes) = file_size_bytes {
        parts.push(format::format_file_size(bytes));
    }
    if let (Some(w), Some(h)) = (width_px, height_px)
        && w > 0
        && h > 0
    {
        parts.push(format!("{w} \u{00D7} {h}"));
    }
    parts.join(" \u{00B7} ")
}

fn browse_item_from_row(
    i18n: Translator,
    thumbnails_dir: &Path,
    row: &librapix_storage::MediaReadModel,
) -> BrowseItem {
    BrowseItem {
        media_id: row.media_id,
        title: row
            .absolute_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| row.absolute_path.display().to_string()),
        thumbnail_path: resolve_thumbnail(thumbnails_dir, row, GALLERY_THUMB_SIZE),
        media_kind: row.media_kind.clone(),
        metadata_line: build_card_metadata_line(
            i18n,
            &row.media_kind,
            Some(row.file_size_bytes),
            row.width_px,
            row.height_px,
        ),
        is_group_header: false,
        line: format!("{} [{}]", row.absolute_path.display(), row.media_kind),
        aspect_ratio: aspect_ratio_from(row.width_px, row.height_px),
        group_image_count: None,
        group_video_count: None,
    }
}

fn is_filterable_tag(tag: &str) -> bool {
    let trimmed = tag.trim();
    !trimmed.is_empty() && !trimmed.starts_with("kind:")
}

fn collect_available_filter_tags(rows: &[librapix_storage::MediaReadModel]) -> Vec<String> {
    let mut tags = rows
        .iter()
        .flat_map(|row| row.tags.iter())
        .filter(|tag| is_filterable_tag(tag))
        .cloned()
        .collect::<Vec<_>>();
    tags.sort_by_key(|tag| tag.to_ascii_lowercase());
    tags.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    tags
}

fn row_matches_tag_filter(
    row: &librapix_storage::MediaReadModel,
    tag_filter: Option<&str>,
) -> bool {
    tag_filter.is_none_or(|selected| {
        row.tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(selected))
    })
}

fn projection_matches_tag_filter(item: &ProjectionMedia, tag_filter: Option<&str>) -> bool {
    tag_filter.is_none_or(|selected| {
        item.tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(selected))
    })
}

fn resolve_thumbnail(
    thumbnails_dir: &std::path::Path,
    row: &librapix_storage::MediaReadModel,
    max_edge: u32,
) -> Option<PathBuf> {
    if !row.media_kind.eq_ignore_ascii_case("image")
        && !row.media_kind.eq_ignore_ascii_case("video")
    {
        return None;
    }
    let candidate = thumbnail_path(
        thumbnails_dir,
        &row.absolute_path,
        row.file_size_bytes,
        row.modified_unix_seconds,
        max_edge,
    );
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

fn load_media_details_cached(app: &mut Librapix) {
    app.details_editing_tag = None;
    app.details_tag_input.clear();
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_tags.clear();
        return;
    };

    if let Some(cached) = app.media_cache.get(&media_id) {
        app.details_title = cached
            .absolute_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| cached.absolute_path.display().to_string());
        app.details_preview_path = cached.detail_thumbnail_path.clone();
        app.details_lines = vec![
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsKindLabel),
                cached.media_kind
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsSizeLabel),
                format::format_file_size(cached.file_size_bytes)
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsModifiedLabel),
                format::format_timestamp(cached.modified_unix_seconds)
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsDimensionsLabel),
                format::format_dimensions(cached.width_px, cached.height_px)
            ),
            format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsPathLabel),
                cached.absolute_path.display()
            ),
        ];
        refresh_details_tags(app, media_id);
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
    } else {
        load_media_details(app);
    }
}

fn rows_to_projection_media(rows: &[librapix_storage::MediaReadModel]) -> Vec<ProjectionMedia> {
    rows.iter()
        .map(|row| ProjectionMedia {
            media_id: row.media_id,
            absolute_path: row.absolute_path.display().to_string(),
            media_kind: row.media_kind.clone(),
            modified_unix_seconds: row.modified_unix_seconds,
            tags: row.tags.clone(),
        })
        .collect()
}

fn set_activity_stage(
    app: &mut Librapix,
    key: TextKey,
    detail_text: impl Into<String>,
    indeterminate: bool,
) {
    app.activity_status = app.i18n.text(key).to_owned();
    app.activity_progress.stage_text = app.activity_status.clone();
    app.activity_progress.detail_text = detail_text.into();
    app.activity_progress.indeterminate = indeterminate;
    app.activity_progress.busy = true;
    app.activity_progress
        .started_at
        .get_or_insert_with(Instant::now);
}

fn set_activity_ready(app: &mut Librapix) {
    app.activity_status = app.i18n.text(TextKey::StageReadyLabel).to_owned();
    app.activity_progress.stage_text = app.activity_status.clone();
    app.activity_progress.detail_text.clear();
    app.activity_progress.items_done = 0;
    app.activity_progress.items_total = None;
    app.activity_progress.roots_done = 0;
    app.activity_progress.roots_total = None;
    app.activity_progress.queue_depth = 0;
    app.activity_progress.indeterminate = false;
    app.activity_progress.busy = false;
    app.activity_progress.started_at = None;
    app.activity_progress.last_error = None;
}

fn projection_stage_key(app: &Librapix) -> TextKey {
    if !app.state.search_query.trim().is_empty() {
        TextKey::StageRefreshingSearchLabel
    } else if matches!(app.state.active_route, Route::Timeline) {
        TextKey::StageRefreshingTimelineLabel
    } else {
        TextKey::StageRefreshingGalleryLabel
    }
}

fn merge_work_reason(
    current: BackgroundWorkReason,
    incoming: BackgroundWorkReason,
) -> BackgroundWorkReason {
    if matches!(incoming, BackgroundWorkReason::FilesystemWatch)
        || matches!(current, BackgroundWorkReason::FilesystemWatch)
    {
        BackgroundWorkReason::FilesystemWatch
    } else {
        BackgroundWorkReason::UserOrSystem
    }
}

fn collect_visible_media_ids_for_thumbnailing(app: &Librapix) -> Vec<i64> {
    let mut seen = HashSet::new();
    let mut ordered = Vec::new();

    let append_items = |items: &[BrowseItem], ordered: &mut Vec<i64>, seen: &mut HashSet<i64>| {
        for media_id in items
            .iter()
            .filter(|item| !item.is_group_header)
            .map(|item| item.media_id)
        {
            if seen.insert(media_id) {
                ordered.push(media_id);
            }
            if ordered.len() >= VISIBLE_THUMBNAIL_QUEUE_LIMIT {
                break;
            }
        }
    };

    if !app.state.search_query.trim().is_empty() {
        append_items(&app.search_items, &mut ordered, &mut seen);
    }
    if ordered.len() < VISIBLE_THUMBNAIL_QUEUE_LIMIT {
        match app.state.active_route {
            Route::Gallery => append_items(&app.gallery_items, &mut ordered, &mut seen),
            Route::Timeline => append_items(&app.timeline_items, &mut ordered, &mut seen),
        }
    }

    ordered
}

fn ensure_valid_library_filter(app: &mut Librapix) {
    let Some(active_filter) = app.filter_source_root_id else {
        return;
    };
    if app
        .state
        .library_roots
        .iter()
        .any(|root| root.id == active_filter)
    {
        return;
    }
    app.filter_source_root_id = None;
    let message = app
        .i18n
        .text(TextKey::LibraryFilterResetUnavailableLabel)
        .to_owned();
    app.browse_status = message.clone();
    app.root_status = message;
}

fn reconcile_selected_media_after_projection(app: &mut Librapix) {
    let Some(selected_media_id) = app.state.selected_media_id else {
        return;
    };
    if app.media_cache.contains_key(&selected_media_id) {
        load_media_details_cached(app);
        return;
    }

    app.state.apply(AppMessage::SetSelectedMedia);
    app.state.set_selected_media(None);
    load_media_details_cached(app);
}

fn start_snapshot_hydrate(app: &mut Librapix) -> Task<Message> {
    app.background.snapshot_generation = app.background.snapshot_generation.saturating_add(1);
    let generation = app.background.snapshot_generation;
    set_activity_stage(app, TextKey::StageLoadingSnapshotLabel, String::new(), true);

    let input = SnapshotHydrateInput {
        generation,
        database_file: app.runtime.database_file.clone(),
    };
    Task::perform(async move { do_snapshot_hydrate(input) }, |result| {
        Message::HydrateSnapshotComplete(Box::new(result))
    })
}

fn do_snapshot_hydrate(input: SnapshotHydrateInput) -> SnapshotHydrateResult {
    let mut out = SnapshotHydrateResult {
        generation: input.generation,
        ..Default::default()
    };
    let Ok(storage) = Storage::open(&input.database_file) else {
        return out;
    };

    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();
    out.ignore_rules = storage.list_ignore_rules("global").unwrap_or_default();
    if let Some(json) = storage
        .load_projection_snapshot(PROJECTION_SNAPSHOT_KEY)
        .ok()
        .flatten()
    {
        match serde_json::from_str::<PersistedProjectionSnapshot>(&json) {
            Ok(snapshot) if snapshot.version == PROJECTION_SNAPSHOT_VERSION => {
                out.snapshot = Some(snapshot);
            }
            Ok(snapshot) => {
                out.snapshot_error = Some(format!(
                    "snapshot version mismatch: expected {}, got {}",
                    PROJECTION_SNAPSHOT_VERSION, snapshot.version
                ));
            }
            Err(error) => {
                out.snapshot_error = Some(format!("snapshot parse failed: {error}"));
            }
        }
    }
    out
}

fn apply_snapshot_hydrate_result(
    app: &mut Librapix,
    result: SnapshotHydrateResult,
) -> Task<Message> {
    if result.generation != app.background.snapshot_generation {
        return Task::none();
    }

    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);
    ensure_valid_library_filter(app);
    app.ignore_rules = result.ignore_rules;
    app.background.snapshot_loaded = true;
    app.activity_progress.last_error = result.snapshot_error;

    if let Some(snapshot) = result.snapshot {
        app.gallery_items = snapshot.gallery_items;
        app.timeline_items = snapshot.timeline_items;
        app.timeline_anchors = snapshot
            .timeline_anchors
            .into_iter()
            .map(|anchor| TimelineAnchor {
                group_index: anchor.group_index,
                label: anchor.label,
                year: anchor.year,
                month: anchor.month,
                day: anchor.day,
                item_count: anchor.item_count,
                normalized_position: anchor.normalized_position,
            })
            .collect();
        app.available_filter_tags = snapshot.available_filter_tags;
        app.state.apply(AppMessage::ReplaceGalleryPreview);
        app.state.replace_gallery_preview(
            app.gallery_items
                .iter()
                .map(|item| item.line.clone())
                .collect(),
        );
        app.state.apply(AppMessage::ReplaceTimelinePreview);
        app.state.replace_timeline_preview(
            app.timeline_items
                .iter()
                .map(|item| item.line.clone())
                .collect(),
        );
        refresh_thumbnail_states_from_items(app);
    }

    if !app.background.reconcile_in_flight && !app.background.projection_in_flight {
        set_activity_ready(app);
    }
    if app.background.startup_reconcile_queued {
        app.background.startup_reconcile_queued = false;
        return request_reconcile(app, BackgroundWorkReason::UserOrSystem);
    }
    Task::none()
}

fn request_reconcile(app: &mut Librapix, reason: BackgroundWorkReason) -> Task<Message> {
    if app.background.reconcile_in_flight {
        app.background.pending_reconcile = true;
        app.background.pending_reconcile_reason =
            merge_work_reason(app.background.pending_reconcile_reason, reason);
        return Task::none();
    }

    app.background.reconcile_in_flight = true;
    app.background.reconcile_generation = app.background.reconcile_generation.saturating_add(1);
    app.background.pending_reconcile = false;
    app.background.pending_reconcile_reason = reason;
    if matches!(reason, BackgroundWorkReason::FilesystemWatch) {
        app.background.pending_watch_paths.clear();
    }
    app.background.reconcile_has_media_changes = false;
    app.background.reconcile_force_projection = !app.background.snapshot_loaded
        || app.gallery_items.is_empty()
        || app.timeline_items.is_empty();
    app.background.deferred_thumbnail_items.clear();

    app.background.thumbnail_generation = app.background.reconcile_generation;
    app.background.thumbnail_queue.clear();
    app.background.thumbnail_queued_ids.clear();
    app.background.thumbnail_done = 0;
    app.background.thumbnail_total = 0;
    app.background.thumbnail_generated = 0;
    app.background.thumbnail_reused = 0;
    app.background.thumbnail_failed = 0;

    set_activity_stage(
        app,
        TextKey::StageCheckingLibrariesLabel,
        app.i18n.text(TextKey::StageScanningFilesLabel),
        true,
    );
    app.activity_progress.roots_total = Some(app.state.library_roots.len());
    app.activity_progress.roots_done = 0;
    app.activity_progress.items_done = 0;
    app.activity_progress.items_total = None;
    app.activity_progress.last_error = None;
    app.indexing_status = app.i18n.text(TextKey::StageScanningFilesLabel).to_owned();

    let input = ScanJobInput {
        generation: app.background.reconcile_generation,
        reason,
        database_file: app.runtime.database_file.clone(),
        min_file_size_bytes: app.min_file_size_bytes,
        visible_media_ids: collect_visible_media_ids_for_thumbnailing(app),
        i18n: app.i18n,
    };
    Task::perform(async move { do_scan_job(input) }, |result| {
        Message::ScanJobComplete(Box::new(result))
    })
}

fn do_scan_job(input: ScanJobInput) -> ScanJobResult {
    let mut out = ScanJobResult {
        generation: input.generation,
        reason: input.reason,
        ..Default::default()
    };

    let mut storage = match Storage::open(&input.database_file) {
        Ok(storage) => storage,
        Err(error) => {
            out.error = Some(error.to_string());
            out.indexing_status = input
                .i18n
                .text(TextKey::ErrorIndexingFailedLabel)
                .to_owned();
            return out;
        }
    };

    let _ = storage.reconcile_source_root_availability();
    let _ = storage.ensure_default_ignore_rules();
    out.ignore_rules = storage.list_ignore_rules("global").unwrap_or_default();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();

    let eligible_roots = storage.list_eligible_source_roots().unwrap_or_default();
    out.root_count = eligible_roots.len();
    let roots_for_scan: Vec<ScanRoot> = eligible_roots
        .iter()
        .map(|root| ScanRoot {
            source_root_id: root.id,
            normalized_path: root.normalized_path.clone(),
        })
        .collect();

    let patterns = storage
        .list_enabled_ignore_patterns("global")
        .unwrap_or_default();
    let ignore = match IgnoreEngine::new(&patterns) {
        Ok(ignore) => ignore,
        Err(error) => {
            out.error = Some(error.to_string());
            out.indexing_status = input
                .i18n
                .text(TextKey::ErrorIndexingFailedLabel)
                .to_owned();
            return out;
        }
    };

    let root_ids: Vec<i64> = roots_for_scan
        .iter()
        .map(|root| root.source_root_id)
        .collect();
    let existing = storage
        .list_existing_indexed_media_snapshots(&root_ids)
        .unwrap_or_default();
    let existing_for_indexer = existing
        .into_iter()
        .map(|entry| librapix_indexer::ExistingIndexedEntry {
            source_root_id: entry.source_root_id,
            absolute_path: entry.absolute_path,
            file_size_bytes: entry.file_size_bytes,
            modified_unix_seconds: entry.modified_unix_seconds,
            width_px: entry.width_px,
            height_px: entry.height_px,
        })
        .collect::<Vec<_>>();
    let scan_result = scan_roots(
        &roots_for_scan,
        &ignore,
        &existing_for_indexer,
        &ScanOptions {
            min_file_size_bytes: input.min_file_size_bytes,
        },
    );
    out.scanned_root_ids = scan_result.scanned_root_ids.clone();

    let writes = scan_result
        .candidates
        .iter()
        .map(|candidate| IndexedMediaWrite {
            source_root_id: candidate.source_root_id,
            absolute_path: candidate.absolute_path.clone(),
            media_kind: candidate.media_kind.as_str().to_owned(),
            file_size_bytes: candidate.file_size_bytes,
            modified_unix_seconds: candidate.modified_unix_seconds,
            width_px: candidate.width_px,
            height_px: candidate.height_px,
            metadata_status: match candidate.metadata_status {
                librapix_indexer::MetadataStatus::Ok => IndexedMetadataStatus::Ok,
                librapix_indexer::MetadataStatus::Partial => IndexedMetadataStatus::Partial,
                librapix_indexer::MetadataStatus::Unreadable => IndexedMetadataStatus::Unreadable,
            },
        })
        .collect::<Vec<_>>();

    let apply_summary =
        match storage.apply_incremental_index(&writes, &scan_result.scanned_root_ids) {
            Ok(summary) => summary,
            Err(error) => {
                out.error = Some(error.to_string());
                out.indexing_status = input
                    .i18n
                    .text(TextKey::ErrorIndexingFailedLabel)
                    .to_owned();
                return out;
            }
        };

    let _ = storage.ensure_media_kind_tags_attached();
    let _ = storage.ensure_root_tags_exist();
    let _ = storage.apply_root_auto_tags();
    let _ = storage.refresh_source_root_statistics(&scan_result.scanned_root_ids);

    let read_model_count = storage.count_indexed_media().unwrap_or_default().max(0) as usize;
    out.indexing_summary = Some(IndexingSummary {
        scanned_roots: scan_result.summary.scanned_roots,
        candidate_files: scan_result.summary.candidate_files,
        ignored_entries: scan_result.summary.ignored_entries,
        unreadable_entries: scan_result.summary.unreadable_entries,
        new_files: scan_result.summary.new_files,
        changed_files: scan_result.summary.changed_files,
        unchanged_files: scan_result.summary.unchanged_files,
        missing_marked: apply_summary.missing_marked_count,
        read_model_count,
    });
    out.has_media_changes = scan_result.summary.new_files > 0
        || scan_result.summary.changed_files > 0
        || apply_summary.missing_marked_count > 0;
    out.indexing_status = input.i18n.text(TextKey::IndexingCompletedLabel).to_owned();

    let changed_paths = scan_result
        .candidates
        .iter()
        .filter(|candidate| {
            !matches!(
                candidate.change_kind,
                librapix_indexer::ChangeKind::Unchanged
            )
        })
        .map(|candidate| candidate.absolute_path.clone())
        .collect::<Vec<_>>();
    let changed_rows = storage
        .list_media_read_models_by_paths(&changed_paths)
        .unwrap_or_default();
    out.changed_media = changed_rows
        .into_iter()
        .filter(|row| {
            row.media_kind.eq_ignore_ascii_case("image")
                || row.media_kind.eq_ignore_ascii_case("video")
        })
        .map(|row| ThumbnailWorkItem {
            generation: input.generation,
            media_id: row.media_id,
            absolute_path: row.absolute_path,
            media_kind: row.media_kind,
            file_size_bytes: row.file_size_bytes,
            modified_unix_seconds: row.modified_unix_seconds,
            attempt: 0,
        })
        .collect();

    out.visible_media = storage
        .list_media_read_models_by_ids(&input.visible_media_ids)
        .unwrap_or_default()
        .into_iter()
        .filter(|row| {
            row.media_kind.eq_ignore_ascii_case("image")
                || row.media_kind.eq_ignore_ascii_case("video")
        })
        .map(|row| ThumbnailWorkItem {
            generation: input.generation,
            media_id: row.media_id,
            absolute_path: row.absolute_path,
            media_kind: row.media_kind,
            file_size_bytes: row.file_size_bytes,
            modified_unix_seconds: row.modified_unix_seconds,
            attempt: 0,
        })
        .collect();
    out
}

fn request_projection_refresh(app: &mut Librapix, reason: BackgroundWorkReason) -> Task<Message> {
    if app.background.reconcile_in_flight {
        app.background.pending_projection_reason = if app.background.pending_projection {
            merge_work_reason(app.background.pending_projection_reason, reason)
        } else {
            reason
        };
        app.background.pending_projection = true;
        set_activity_stage(app, projection_stage_key(app), String::new(), true);
        return Task::none();
    }
    if app.background.projection_in_flight {
        app.background.pending_projection_reason = if app.background.pending_projection {
            merge_work_reason(app.background.pending_projection_reason, reason)
        } else {
            reason
        };
        app.background.pending_projection = true;
        set_activity_stage(app, projection_stage_key(app), String::new(), true);
        return Task::none();
    }
    start_projection_refresh(app, reason)
}

fn start_projection_refresh(app: &mut Librapix, reason: BackgroundWorkReason) -> Task<Message> {
    app.background.projection_in_flight = true;
    app.background.pending_projection = false;
    app.background.pending_projection_reason = reason;
    app.background.projection_generation = app.background.projection_generation.saturating_add(1);
    set_activity_stage(app, projection_stage_key(app), String::new(), true);

    let input = ProjectionJobInput {
        generation: app.background.projection_generation,
        reason,
        database_file: app.runtime.database_file.clone(),
        thumbnails_dir: app.runtime.thumbnails_dir.clone(),
        filter_media_kind: app.filter_media_kind.clone(),
        filter_extension: app.filter_extension.clone(),
        filter_tag: app.filter_tag.clone(),
        filter_source_root_id: app.filter_source_root_id,
        search_query: app.state.search_query.clone(),
        active_route: app.state.active_route,
        i18n: app.i18n,
    };
    Task::perform(async move { do_projection_job(input) }, |result| {
        Message::ProjectionJobComplete(Box::new(result))
    })
}

fn do_projection_job(input: ProjectionJobInput) -> ProjectionJobResult {
    let mut out = ProjectionJobResult {
        generation: input.generation,
        reason: input.reason,
        ..Default::default()
    };

    let Ok(storage) = Storage::open(&input.database_file) else {
        return out;
    };
    let _ = storage.reconcile_source_root_availability();
    let _ = storage.ensure_default_ignore_rules();
    out.ignore_rules = storage.list_ignore_rules("global").unwrap_or_default();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();

    let all_rows = storage
        .list_all_media_read_models_filtered(input.filter_source_root_id)
        .unwrap_or_default();
    out.available_filter_tags = collect_available_filter_tags(&all_rows);
    let active_tag_filter = input.filter_tag.as_ref().filter(|selected| {
        out.available_filter_tags
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(selected))
    });

    let row_lookup = all_rows
        .iter()
        .map(|row| (row.media_id, row))
        .collect::<HashMap<_, _>>();
    let media = rows_to_projection_media(&all_rows);

    let gallery_query = GalleryQuery {
        media_kind: input.filter_media_kind.clone(),
        extension: input.filter_extension.clone(),
        tag: active_tag_filter.cloned(),
        sort: GallerySort::ModifiedDesc,
        limit: all_rows.len(),
        offset: 0,
    };
    out.gallery_items = project_gallery(&media, &gallery_query)
        .into_iter()
        .map(|item| {
            row_lookup.get(&item.media_id).copied().map_or_else(
                || {
                    let path = PathBuf::from(item.absolute_path);
                    BrowseItem {
                        media_id: item.media_id,
                        title: path
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.display().to_string()),
                        thumbnail_path: None,
                        media_kind: item.media_kind.clone(),
                        metadata_line: build_card_metadata_line(
                            input.i18n,
                            &item.media_kind,
                            None,
                            None,
                            None,
                        ),
                        is_group_header: false,
                        line: format!("{} [{}]", path.display(), item.media_kind),
                        aspect_ratio: 1.5,
                        group_image_count: None,
                        group_video_count: None,
                    }
                },
                |row| browse_item_from_row(input.i18n, &input.thumbnails_dir, row),
            )
        })
        .collect();
    out.gallery_preview_lines = out
        .gallery_items
        .iter()
        .map(|item| item.line.clone())
        .collect();

    let filtered_media = media
        .into_iter()
        .filter(|item| {
            input
                .filter_media_kind
                .as_ref()
                .is_none_or(|kind| item.media_kind.eq_ignore_ascii_case(kind))
        })
        .filter(|item| {
            input.filter_extension.as_ref().is_none_or(|ext| {
                item.absolute_path
                    .rsplit('.')
                    .next()
                    .is_some_and(|value| value.eq_ignore_ascii_case(ext))
            })
        })
        .filter(|item| {
            projection_matches_tag_filter(item, active_tag_filter.map(|tag| tag.as_str()))
        })
        .collect::<Vec<_>>();

    let buckets = project_timeline(&filtered_media, TimelineGranularity::Day);
    out.timeline_anchors = build_timeline_anchors(&buckets);
    out.timeline_items = buckets
        .into_iter()
        .flat_map(|bucket| {
            let image_count = bucket
                .items
                .iter()
                .filter(|item| item.media_kind.eq_ignore_ascii_case("image"))
                .count();
            let video_count = bucket
                .items
                .iter()
                .filter(|item| item.media_kind.eq_ignore_ascii_case("video"))
                .count();
            let mut rows = vec![BrowseItem {
                media_id: 0,
                title: bucket.label.clone(),
                thumbnail_path: None,
                media_kind: String::new(),
                metadata_line: String::new(),
                is_group_header: true,
                line: bucket.label.clone(),
                aspect_ratio: 1.5,
                group_image_count: Some(image_count),
                group_video_count: Some(video_count),
            }];
            rows.extend(bucket.items.into_iter().map(|item| {
                row_lookup.get(&item.media_id).copied().map_or_else(
                    || BrowseItem {
                        media_id: item.media_id,
                        title: PathBuf::from(&item.absolute_path)
                            .file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or(item.absolute_path.clone()),
                        thumbnail_path: None,
                        media_kind: item.media_kind.clone(),
                        metadata_line: build_card_metadata_line(
                            input.i18n,
                            &item.media_kind,
                            None,
                            None,
                            None,
                        ),
                        is_group_header: false,
                        line: format!("{} [{}]", item.absolute_path, item.media_kind),
                        aspect_ratio: 1.5,
                        group_image_count: None,
                        group_video_count: None,
                    },
                    |row| {
                        let mut browse =
                            browse_item_from_row(input.i18n, &input.thumbnails_dir, row);
                        browse.line = format!("{} [{}]", item.absolute_path, item.media_kind);
                        browse
                    },
                )
            }));
            rows
        })
        .collect();
    out.timeline_preview_lines = out
        .timeline_items
        .iter()
        .map(|item| item.line.clone())
        .collect();

    if !input.search_query.trim().is_empty() {
        let docs = all_rows
            .iter()
            .map(|row| SearchDocument {
                media_id: row.media_id,
                absolute_path: row.absolute_path.display().to_string(),
                file_name: row
                    .absolute_path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default(),
                media_kind: row.media_kind.clone(),
                tags: row.tags.clone(),
            })
            .collect::<Vec<_>>();
        let strategy = FuzzySearchStrategy::default();
        let hits = strategy.search(
            &docs,
            &SearchQuery {
                text: input.search_query.clone(),
                limit: all_rows.len(),
            },
        );
        out.search_items = hits
            .into_iter()
            .filter_map(|hit| row_lookup.get(&hit.media_id).copied())
            .filter(|row| {
                input
                    .filter_media_kind
                    .as_ref()
                    .is_none_or(|kind| row.media_kind.eq_ignore_ascii_case(kind))
            })
            .filter(|row| {
                input.filter_extension.as_ref().is_none_or(|ext| {
                    row.absolute_path
                        .extension()
                        .and_then(|value| value.to_str())
                        .is_some_and(|value| value.eq_ignore_ascii_case(ext))
                })
            })
            .filter(|row| row_matches_tag_filter(row, active_tag_filter.map(|tag| tag.as_str())))
            .map(|row| browse_item_from_row(input.i18n, &input.thumbnails_dir, row))
            .collect::<Vec<_>>();
        out.search_preview_lines = out
            .search_items
            .iter()
            .map(|item| item.line.clone())
            .collect();
    }

    let projected_ids = out
        .gallery_items
        .iter()
        .chain(out.timeline_items.iter())
        .chain(out.search_items.iter())
        .filter(|item| !item.is_group_header)
        .map(|item| item.media_id)
        .collect::<HashSet<_>>();
    out.media_cache = projected_ids
        .iter()
        .filter_map(|media_id| row_lookup.get(media_id).copied())
        .map(|row| {
            (
                row.media_id,
                CachedDetails {
                    absolute_path: row.absolute_path.clone(),
                    media_kind: row.media_kind.clone(),
                    file_size_bytes: row.file_size_bytes,
                    modified_unix_seconds: row.modified_unix_seconds,
                    width_px: row.width_px,
                    height_px: row.height_px,
                    detail_thumbnail_path: resolve_thumbnail(
                        &input.thumbnails_dir,
                        row,
                        DETAIL_THUMB_SIZE,
                    ),
                },
            )
        })
        .collect();

    out.browse_status = if input.search_query.trim().is_empty() {
        if matches!(input.active_route, Route::Timeline) {
            input.i18n.text(TextKey::TimelineCompletedLabel).to_owned()
        } else {
            input.i18n.text(TextKey::GalleryCompletedLabel).to_owned()
        }
    } else {
        input.i18n.text(TextKey::SearchCompletedLabel).to_owned()
    };

    if input.filter_media_kind.is_none()
        && input.filter_extension.is_none()
        && input.filter_source_root_id.is_none()
        && active_tag_filter.is_none()
        && input.search_query.trim().is_empty()
    {
        let snapshot = PersistedProjectionSnapshot {
            version: PROJECTION_SNAPSHOT_VERSION,
            gallery_items: out.gallery_items.clone(),
            timeline_items: out.timeline_items.clone(),
            timeline_anchors: out
                .timeline_anchors
                .iter()
                .map(|anchor| PersistedTimelineAnchor {
                    group_index: anchor.group_index,
                    label: anchor.label.clone(),
                    year: anchor.year,
                    month: anchor.month,
                    day: anchor.day,
                    item_count: anchor.item_count,
                    normalized_position: anchor.normalized_position,
                })
                .collect(),
            available_filter_tags: out.available_filter_tags.clone(),
            updated_unix_seconds: chrono::Utc::now().timestamp(),
        };
        out.snapshot_payload = serde_json::to_string(&snapshot).ok();
    }

    out
}

fn request_post_projection_thumbnail_work(app: &mut Librapix) -> Task<Message> {
    let mut items = Vec::new();
    items.extend(app.background.deferred_thumbnail_items.clone());
    app.background.deferred_thumbnail_items.clear();
    items.extend(visible_thumbnail_candidates(app));
    enqueue_thumbnail_items(app, items);
    app.background.thumbnail_total = app
        .background
        .thumbnail_total
        .max(app.background.thumbnail_done + app.background.thumbnail_queue.len());
    run_next_thumbnail_batch_if_idle(app)
}

fn visible_thumbnail_candidates(app: &Librapix) -> Vec<ThumbnailWorkItem> {
    let items = if !app.state.search_query.trim().is_empty() {
        &app.search_items
    } else if matches!(app.state.active_route, Route::Timeline) {
        &app.timeline_items
    } else {
        &app.gallery_items
    };

    items
        .iter()
        .filter(|item| !item.is_group_header)
        .take(VISIBLE_THUMBNAIL_QUEUE_LIMIT)
        .filter_map(|item| {
            app.media_cache
                .get(&item.media_id)
                .map(|details| (item, details))
        })
        .filter(|(_, details)| {
            details.media_kind.eq_ignore_ascii_case("image")
                || details.media_kind.eq_ignore_ascii_case("video")
        })
        .map(|(item, details)| ThumbnailWorkItem {
            generation: app.background.thumbnail_generation,
            media_id: item.media_id,
            absolute_path: details.absolute_path.clone(),
            media_kind: details.media_kind.clone(),
            file_size_bytes: details.file_size_bytes,
            modified_unix_seconds: details.modified_unix_seconds,
            attempt: 0,
        })
        .collect()
}

fn queue_selected_media_thumbnail(app: &mut Librapix) -> Task<Message> {
    let Some(media_id) = app.state.selected_media_id else {
        return Task::none();
    };
    let details = if let Some(cached) = app.media_cache.get(&media_id) {
        cached.clone()
    } else {
        let Some(row) = with_storage(&app.runtime, |storage| {
            storage.get_media_read_model_by_id(media_id)
        })
        .ok()
        .flatten() else {
            return Task::none();
        };
        let details = CachedDetails {
            absolute_path: row.absolute_path.clone(),
            media_kind: row.media_kind.clone(),
            file_size_bytes: row.file_size_bytes,
            modified_unix_seconds: row.modified_unix_seconds,
            width_px: row.width_px,
            height_px: row.height_px,
            detail_thumbnail_path: resolve_thumbnail(
                &app.runtime.thumbnails_dir,
                &row,
                DETAIL_THUMB_SIZE,
            ),
        };
        app.media_cache.insert(media_id, details.clone());
        details
    };
    let item = ThumbnailWorkItem {
        generation: app.background.thumbnail_generation,
        media_id,
        absolute_path: details.absolute_path,
        media_kind: details.media_kind,
        file_size_bytes: details.file_size_bytes,
        modified_unix_seconds: details.modified_unix_seconds,
        attempt: 0,
    };
    enqueue_thumbnail_items(app, vec![item]);
    run_next_thumbnail_batch_if_idle(app)
}

fn enqueue_thumbnail_items(app: &mut Librapix, items: Vec<ThumbnailWorkItem>) {
    for item in items {
        if item.generation != app.background.thumbnail_generation {
            continue;
        }
        if app.background.thumbnail_queued_ids.contains(&item.media_id) {
            continue;
        }
        if let Some(ThumbnailState::Ready(_)) = app.thumbnail_states.get(&item.media_id) {
            continue;
        }
        if let Some(ThumbnailState::Generating) = app.thumbnail_states.get(&item.media_id) {
            continue;
        }
        if matches!(
            app.thumbnail_states.get(&item.media_id),
            Some(ThumbnailState::Failed {
                retryable: false,
                ..
            })
        ) {
            continue;
        }
        app.background.thumbnail_queued_ids.insert(item.media_id);
        app.background.thumbnail_queue.push_back(item.clone());
        app.thumbnail_states
            .insert(item.media_id, ThumbnailState::Queued);
    }
    app.activity_progress.queue_depth = app.background.thumbnail_queue.len();
}

fn run_next_thumbnail_batch_if_idle(app: &mut Librapix) -> Task<Message> {
    if app.background.thumbnail_in_flight || app.background.thumbnail_queue.is_empty() {
        return Task::none();
    }

    app.background.thumbnail_in_flight = true;
    let mut batch = Vec::new();
    for _ in 0..THUMBNAIL_BATCH_SIZE {
        let Some(item) = app.background.thumbnail_queue.pop_front() else {
            break;
        };
        app.thumbnail_states
            .insert(item.media_id, ThumbnailState::Generating);
        batch.push(item);
    }

    app.activity_progress.items_total =
        Some(app.background.thumbnail_total.max(
            app.background.thumbnail_done + batch.len() + app.background.thumbnail_queue.len(),
        ));
    app.activity_progress.items_done = app.background.thumbnail_done;
    app.activity_progress.queue_depth = app.background.thumbnail_queue.len() + batch.len();
    set_activity_stage(
        app,
        TextKey::StageGeneratingThumbnailsLabel,
        String::new(),
        false,
    );

    let input = ThumbnailBatchInput {
        generation: app.background.thumbnail_generation,
        thumbnails_dir: app.runtime.thumbnails_dir.clone(),
        items: batch,
    };
    Task::perform(async move { do_thumbnail_batch(input) }, |result| {
        Message::ThumbnailBatchComplete(Box::new(result))
    })
}

fn do_thumbnail_batch(input: ThumbnailBatchInput) -> ThumbnailBatchResult {
    let mut out = ThumbnailBatchResult {
        generation: input.generation,
        ..Default::default()
    };

    for item in input.items {
        let outcome = if item.media_kind.eq_ignore_ascii_case("image") {
            ensure_image_thumbnail(
                &input.thumbnails_dir,
                &item.absolute_path,
                item.file_size_bytes,
                item.modified_unix_seconds,
                GALLERY_THUMB_SIZE,
            )
        } else if item.media_kind.eq_ignore_ascii_case("video") {
            ensure_video_thumbnail(
                &input.thumbnails_dir,
                &item.absolute_path,
                item.file_size_bytes,
                item.modified_unix_seconds,
                GALLERY_THUMB_SIZE,
            )
        } else {
            continue;
        };

        match outcome {
            Ok(thumbnail) => {
                if thumbnail.generated {
                    out.generated += 1;
                } else {
                    out.reused += 1;
                }
                out.outcomes.push(ThumbnailWorkOutcome {
                    media_id: item.media_id,
                    state: ThumbnailState::Ready(thumbnail.thumbnail_path.clone()),
                    thumbnail_path: Some(thumbnail.thumbnail_path),
                });
            }
            Err(error) => {
                let retryable = thumbnail_error_retryable(&error);
                out.failed += 1;
                out.outcomes.push(ThumbnailWorkOutcome {
                    media_id: item.media_id,
                    state: ThumbnailState::Failed {
                        retryable,
                        last_error: error.to_string(),
                    },
                    thumbnail_path: None,
                });
                if retryable && item.attempt.saturating_add(1) < THUMBNAIL_MAX_RETRY_ATTEMPTS {
                    out.retry_items.push(ThumbnailWorkItem {
                        generation: item.generation,
                        media_id: item.media_id,
                        absolute_path: item.absolute_path,
                        media_kind: item.media_kind,
                        file_size_bytes: item.file_size_bytes,
                        modified_unix_seconds: item.modified_unix_seconds,
                        attempt: item.attempt.saturating_add(1),
                    });
                }
            }
        }
    }

    out
}

fn thumbnail_error_retryable(error: &librapix_thumbnails::ThumbnailError) -> bool {
    match error {
        librapix_thumbnails::ThumbnailError::Io(io) => !io
            .to_string()
            .to_ascii_lowercase()
            .contains("ffmpeg may not be installed"),
        librapix_thumbnails::ThumbnailError::Image(_) => true,
    }
}

fn item_retry_generation_is_current(app: &Librapix, item: &ThumbnailWorkItem) -> bool {
    item.generation == app.background.thumbnail_generation
}

fn thumbnail_retry_delay_ms(attempt: u8) -> u64 {
    let exponent = u32::from(attempt.saturating_sub(1)).min(4);
    THUMBNAIL_RETRY_DELAY_MS.saturating_mul(2u64.saturating_pow(exponent))
}

fn on_filesystem_changed(app: &mut Librapix, paths: Vec<PathBuf>) -> Task<Message> {
    if paths.is_empty() {
        return Task::none();
    }
    app.background.pending_watch_paths.extend(paths);
    app.activity_progress.detail_text = format!(
        "{}: {}",
        app.i18n.text(TextKey::ProgressQueueLabel),
        app.background.pending_watch_paths.len()
    );
    request_reconcile(app, BackgroundWorkReason::FilesystemWatch)
}

fn apply_scan_job_result(app: &mut Librapix, result: ScanJobResult) -> Task<Message> {
    if result.generation != app.background.reconcile_generation {
        return Task::none();
    }
    let reason = result.reason;

    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);
    ensure_valid_library_filter(app);
    app.ignore_rules = result.ignore_rules;
    app.indexing_status = result.indexing_status;
    app.activity_progress.roots_total = Some(result.root_count);
    app.activity_progress.roots_done = result.scanned_root_ids.len();
    if let Some(summary) = result.indexing_summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
    }

    if let Some(error) = result.error {
        app.activity_progress.last_error = Some(error);
    }
    app.background.reconcile_has_media_changes = result.has_media_changes;
    let mut deferred_thumbnail_items = Vec::new();
    let mut deferred_ids = HashSet::new();
    for item in result
        .changed_media
        .into_iter()
        .chain(result.visible_media.into_iter())
    {
        if deferred_ids.insert(item.media_id) {
            deferred_thumbnail_items.push(item);
        }
    }
    app.background.deferred_thumbnail_items = deferred_thumbnail_items;
    app.background.thumbnail_total = app.background.deferred_thumbnail_items.len();
    app.background.thumbnail_done = 0;
    app.background.thumbnail_generated = 0;
    app.background.thumbnail_reused = 0;
    app.background.thumbnail_failed = 0;
    if app.library_stats_dialog_open
        && let Some(root_id) = app.library_stats_root_id
    {
        app.library_stats_record = with_storage(&app.runtime, |storage| {
            storage.get_source_root_statistics(root_id)
        })
        .ok()
        .flatten();
    }

    if app.background.reconcile_has_media_changes
        || app.background.reconcile_force_projection
        || app.background.pending_projection
    {
        let projection_reason = if app.background.pending_projection {
            merge_work_reason(app.background.pending_projection_reason, reason)
        } else {
            reason
        };
        app.background.reconcile_in_flight = false;
        app.background.pending_projection = false;
        return request_projection_refresh(app, projection_reason);
    }

    let thumbnail_task = request_post_projection_thumbnail_work(app);
    if app.background.thumbnail_in_flight {
        return thumbnail_task;
    }

    app.background.reconcile_in_flight = false;
    finalize_background_flow(app)
}

fn apply_projection_job_result(app: &mut Librapix, result: ProjectionJobResult) -> Task<Message> {
    if result.generation != app.background.projection_generation {
        return Task::none();
    }
    app.background.projection_in_flight = false;

    let previous_media_ids = app.media_cache.keys().copied().collect::<HashSet<_>>();
    let announcement = if matches!(result.reason, BackgroundWorkReason::FilesystemWatch) {
        build_new_media_announcement(app.i18n, &previous_media_ids, &result.media_cache)
    } else {
        None
    };

    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);
    ensure_valid_library_filter(app);
    app.ignore_rules = result.ignore_rules;

    app.state.apply(AppMessage::ReplaceSearchPreview);
    app.state
        .replace_search_preview(result.search_preview_lines);
    app.search_items = result.search_items;

    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state
        .replace_gallery_preview(result.gallery_preview_lines);
    app.gallery_items = result.gallery_items;

    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state
        .replace_timeline_preview(result.timeline_preview_lines);
    app.timeline_items = result.timeline_items;
    app.timeline_anchors = result.timeline_anchors;
    sync_timeline_scrub_selection(app, app.timeline_scrub_value);

    app.media_cache = result.media_cache;
    refresh_thumbnail_states_from_items(app);
    reconcile_selected_media_after_projection(app);
    app.available_filter_tags = result.available_filter_tags;
    if app.filter_tag.as_ref().is_some_and(|tag| {
        !app.available_filter_tags
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(tag))
    }) {
        app.filter_tag = None;
    }
    app.browse_status = result.browse_status;

    if let Some(mut announcement) = announcement {
        if announcement.preview_path.is_none()
            && let Some(path) = app
                .media_cache
                .get(&announcement.media_id)
                .and_then(|details| details.detail_thumbnail_path.as_ref())
        {
            announcement.preview_path = Some(path.clone());
        }
        app.new_media_announcement = Some(announcement);
    }

    if let Some(payload) = result.snapshot_payload {
        let _ = with_storage(&app.runtime, |storage| {
            storage.upsert_projection_snapshot(PROJECTION_SNAPSHOT_KEY, &payload)
        });
    }
    if app.library_stats_dialog_open
        && let Some(root_id) = app.library_stats_root_id
    {
        app.library_stats_record = with_storage(&app.runtime, |storage| {
            storage.get_source_root_statistics(root_id)
        })
        .ok()
        .flatten();
    }

    let thumbnail_task = request_post_projection_thumbnail_work(app);
    if app.background.thumbnail_in_flight {
        return thumbnail_task;
    }

    if app.background.reconcile_in_flight {
        app.background.reconcile_in_flight = false;
    }
    finalize_background_flow(app)
}

fn apply_thumbnail_batch_result(app: &mut Librapix, result: ThumbnailBatchResult) -> Task<Message> {
    if result.generation != app.background.thumbnail_generation {
        // A superseded batch completed after a newer generation was requested.
        // Release the in-flight lock and continue the current generation queue.
        app.background.thumbnail_in_flight = false;
        app.activity_progress.queue_depth = app.background.thumbnail_queue.len();
        let next_batch = run_next_thumbnail_batch_if_idle(app);
        if app.background.thumbnail_in_flight {
            return next_batch;
        }
        return finalize_background_flow(app);
    }

    app.background.thumbnail_in_flight = false;
    app.background.thumbnail_generated += result.generated;
    app.background.thumbnail_reused += result.reused;
    app.background.thumbnail_failed += result.failed;

    for outcome in result.outcomes {
        app.background
            .thumbnail_queued_ids
            .remove(&outcome.media_id);
        if let ThumbnailState::Failed { last_error, .. } = &outcome.state {
            app.activity_progress.last_error = Some(last_error.clone());
        }
        app.thumbnail_states
            .insert(outcome.media_id, outcome.state.clone());
        if let Some(path) = outcome.thumbnail_path {
            patch_thumbnail_path(&mut app.gallery_items, outcome.media_id, &path);
            patch_thumbnail_path(&mut app.timeline_items, outcome.media_id, &path);
            patch_thumbnail_path(&mut app.search_items, outcome.media_id, &path);
            if let Some(details) = app.media_cache.get_mut(&outcome.media_id) {
                details.detail_thumbnail_path = Some(path.clone());
            }
            if let Some(announcement) = app.new_media_announcement.as_mut()
                && announcement.media_id == outcome.media_id
            {
                announcement.preview_path = Some(path.clone());
            }
            if app.state.selected_media_id == Some(outcome.media_id) {
                app.details_preview_path = Some(path);
            }
        }
        app.background.thumbnail_done = app.background.thumbnail_done.saturating_add(1);
    }

    let retry_tasks = result
        .retry_items
        .into_iter()
        .map(|item| {
            let delay_ms = thumbnail_retry_delay_ms(item.attempt);
            Task::perform(
                async move {
                    std::thread::sleep(Duration::from_millis(delay_ms));
                    item
                },
                Message::ThumbnailRetryEnqueue,
            )
        })
        .collect::<Vec<_>>();

    app.thumbnail_status = format!(
        "{}: {}={}, {}={}, {}={}",
        app.i18n.text(TextKey::ThumbnailStatusLabel),
        app.i18n.text(TextKey::ThumbnailGeneratedLabel),
        app.background.thumbnail_generated,
        app.i18n.text(TextKey::ThumbnailReusedLabel),
        app.background.thumbnail_reused,
        app.i18n.text(TextKey::ThumbnailFailedLabel),
        app.background.thumbnail_failed,
    );

    app.activity_progress.items_total = Some(
        app.background
            .thumbnail_total
            .max(app.background.thumbnail_done),
    );
    app.activity_progress.items_done = app.background.thumbnail_done;
    app.activity_progress.queue_depth = app.background.thumbnail_queue.len();

    let mut tasks = Vec::new();
    let next_batch = run_next_thumbnail_batch_if_idle(app);
    if app.background.thumbnail_in_flight {
        tasks.push(next_batch);
    } else {
        if app.background.reconcile_in_flight {
            app.background.reconcile_in_flight = false;
        }
        if !app.background.projection_in_flight {
            tasks.push(finalize_background_flow(app));
        }
    }

    tasks.extend(retry_tasks);
    if tasks.is_empty() {
        Task::none()
    } else {
        Task::batch(tasks)
    }
}

fn patch_thumbnail_path(items: &mut [BrowseItem], media_id: i64, path: &Path) {
    for item in items.iter_mut().filter(|item| item.media_id == media_id) {
        item.thumbnail_path = Some(path.to_path_buf());
    }
}

fn refresh_thumbnail_states_from_items(app: &mut Librapix) {
    let mut seen_ids = HashSet::new();
    for item in app
        .gallery_items
        .iter()
        .chain(app.timeline_items.iter())
        .chain(app.search_items.iter())
        .filter(|item| !item.is_group_header)
    {
        seen_ids.insert(item.media_id);
        if matches!(
            app.thumbnail_states.get(&item.media_id),
            Some(ThumbnailState::Queued) | Some(ThumbnailState::Generating)
        ) {
            continue;
        }
        if let Some(path) = &item.thumbnail_path {
            app.thumbnail_states
                .insert(item.media_id, ThumbnailState::Ready(path.clone()));
        } else {
            app.thumbnail_states
                .insert(item.media_id, ThumbnailState::Missing);
        }
    }
    app.thumbnail_states.retain(|media_id, state| {
        seen_ids.contains(media_id)
            || matches!(state, ThumbnailState::Queued | ThumbnailState::Generating)
    });
}

fn finalize_background_flow(app: &mut Librapix) -> Task<Message> {
    if app.background.pending_reconcile {
        let reason = app.background.pending_reconcile_reason;
        app.background.pending_reconcile = false;
        return request_reconcile(app, reason);
    }
    if app.background.pending_projection {
        let reason = app.background.pending_projection_reason;
        app.background.pending_projection = false;
        return request_projection_refresh(app, reason);
    }
    if !app.background.reconcile_in_flight
        && !app.background.thumbnail_in_flight
        && !app.background.projection_in_flight
        && app.background.thumbnail_queue.is_empty()
    {
        set_activity_ready(app);
    }
    Task::none()
}

fn refresh_diagnostics(app: &mut Librapix) {
    let mut lines = Vec::new();

    let (indexed_count, roots_total, roots_eligible) = with_storage(&app.runtime, |storage| {
        let indexed = storage.count_indexed_media().unwrap_or(-1);
        let total = storage.list_source_roots().map(|r| r.len()).unwrap_or(0);
        let eligible = storage
            .list_eligible_source_roots()
            .map(|r| r.len())
            .unwrap_or(0);
        Ok::<_, librapix_storage::StorageError>((indexed, total, eligible))
    })
    .unwrap_or((-1, 0, 0));

    lines.push(format!(
        "roots: {} total, {} eligible",
        roots_total, roots_eligible
    ));
    lines.push(format!("indexed media: {}", indexed_count));
    lines.push(format!("gallery items: {}", app.gallery_items.len()));
    lines.push(format!("timeline items: {}", app.timeline_items.len()));
    lines.push(format!(
        "available tags: {}",
        app.available_filter_tags.len()
    ));
    lines.push(format!("timeline anchors: {}", app.timeline_anchors.len()));
    lines.push(format!(
        "timeline scrub: value={:.3}, active={:?}, dragging={}",
        app.timeline_scrub_value, app.timeline_scrub_anchor_index, app.timeline_scrubbing
    ));
    lines.push(format!(
        "filter: root={:?}, kind={:?}, ext={:?}, tag={:?}",
        app.filter_source_root_id,
        app.filter_media_kind.as_deref().unwrap_or("all"),
        app.filter_extension.as_deref().unwrap_or("all"),
        app.filter_tag.as_deref().unwrap_or("all")
    ));
    lines.push(format!("min file size: {} bytes", app.min_file_size_bytes));
    lines.push(format!(
        "updates: {}",
        update_check_status_label(&app.update_check_state)
    ));
    lines.push(format!("browse status: {}", app.browse_status));
    lines.push(format!(
        "activity: stage='{}', busy={}, indeterminate={}, queue={}",
        app.activity_progress.stage_text,
        app.activity_progress.busy,
        app.activity_progress.indeterminate,
        app.activity_progress.queue_depth
    ));
    lines.push(format!(
        "work generations: snapshot={}, reconcile={}, projection={}, thumbnail={}",
        app.background.snapshot_generation,
        app.background.reconcile_generation,
        app.background.projection_generation,
        app.background.thumbnail_generation
    ));
    lines.push(format!(
        "work in-flight: reconcile={}, projection={}, thumbnail={}",
        app.background.reconcile_in_flight,
        app.background.projection_in_flight,
        app.background.thumbnail_in_flight
    ));
    lines.push(format!(
        "watch pending paths: {}",
        app.background.pending_watch_paths.len()
    ));
    lines.push(format!(
        "thumbnails: done={}, total={}, generated={}, reused={}, failed={}",
        app.background.thumbnail_done,
        app.background.thumbnail_total,
        app.background.thumbnail_generated,
        app.background.thumbnail_reused,
        app.background.thumbnail_failed
    ));

    app.diagnostics_lines = lines;
}

fn build_new_media_announcement(
    i18n: Translator,
    previous_media_ids: &HashSet<i64>,
    current_media_cache: &HashMap<i64, CachedDetails>,
) -> Option<NewMediaAnnouncement> {
    let mut new_items = current_media_cache
        .iter()
        .filter(|(media_id, _)| !previous_media_ids.contains(media_id))
        .map(|(media_id, details)| (*media_id, details))
        .collect::<Vec<_>>();

    if new_items.is_empty() {
        return None;
    }

    new_items.sort_by(|(left_id, left), (right_id, right)| {
        right
            .modified_unix_seconds
            .unwrap_or(i64::MIN)
            .cmp(&left.modified_unix_seconds.unwrap_or(i64::MIN))
            .then_with(|| right_id.cmp(left_id))
    });

    let (media_id, details) = new_items[0];
    Some(NewMediaAnnouncement {
        media_id,
        title: details
            .absolute_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| details.absolute_path.display().to_string()),
        metadata_line: build_card_metadata_line(
            i18n,
            &details.media_kind,
            Some(details.file_size_bytes),
            details.width_px,
            details.height_px,
        ),
        preview_path: details.detail_thumbnail_path.clone(),
        media_kind: details.media_kind.clone(),
        file_size_bytes: details.file_size_bytes,
        modified_unix_seconds: details.modified_unix_seconds,
        width_px: details.width_px,
        height_px: details.height_px,
        absolute_path: details.absolute_path.clone(),
        additional_count: new_items.len().saturating_sub(1),
    })
}

fn with_storage<T>(
    runtime: &RuntimeContext,
    action: impl FnOnce(&mut Storage) -> Result<T, librapix_storage::StorageError>,
) -> Result<T, librapix_storage::StorageError> {
    let mut storage = Storage::open(&runtime.database_file)?;
    action(&mut storage)
}

fn map_roots_from_storage(roots: Vec<librapix_storage::SourceRootRecord>) -> Vec<LibraryRootView> {
    roots
        .into_iter()
        .map(|root| LibraryRootView {
            id: root.id,
            normalized_path: root.normalized_path,
            lifecycle: match root.lifecycle {
                SourceRootLifecycle::Active => RootLifecycle::Active,
                SourceRootLifecycle::Unavailable => RootLifecycle::Unavailable,
                SourceRootLifecycle::Deactivated => RootLifecycle::Deactivated,
            },
            display_name: root.display_name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(
        group_index: usize,
        year: Option<i32>,
        label: &str,
        normalized_position: f32,
    ) -> TimelineAnchor {
        TimelineAnchor {
            group_index,
            label: label.to_owned(),
            year,
            month: None,
            day: None,
            item_count: 10,
            normalized_position,
        }
    }

    fn key_pressed_event(
        key: keyboard::Key,
        modifiers: keyboard::Modifiers,
        repeat: bool,
    ) -> keyboard::Event {
        keyboard::Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: key::Physical::Unidentified(key::NativeCode::Unidentified),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat,
        }
    }

    #[test]
    fn scrub_value_to_anchor_index_maps_across_anchor_span() {
        let anchors = vec![
            anchor(0, Some(2026), "2026-03-01", 0.0),
            anchor(1, Some(2025), "2025-12-01", 0.35),
            anchor(2, Some(2024), "2024-08-01", 0.7),
            anchor(3, None, "unknown", 1.0),
        ];

        assert_eq!(scrub_value_to_anchor_index(&anchors, 0.01), Some(0));
        assert_eq!(scrub_value_to_anchor_index(&anchors, 0.20), Some(1));
        assert_eq!(scrub_value_to_anchor_index(&anchors, 0.55), Some(2));
        assert_eq!(scrub_value_to_anchor_index(&anchors, 0.97), Some(3));
    }

    #[test]
    fn year_markers_deduplicate_by_year_and_use_anchor_positions() {
        let anchors = vec![
            anchor(0, Some(2026), "2026-03-01", 0.0),
            anchor(1, Some(2026), "2026-02-01", 0.1),
            anchor(2, Some(2025), "2025-12-01", 0.4),
            anchor(3, Some(2025), "2025-08-01", 0.55),
            anchor(4, None, "unknown", 1.0),
        ];

        let markers = timeline_year_markers(&anchors);
        assert_eq!(
            markers,
            vec![
                TimelineYearMarker {
                    label: "2026".to_owned(),
                    group_index: 0,
                    normalized_position: 0.0,
                },
                TimelineYearMarker {
                    label: "2025".to_owned(),
                    group_index: 2,
                    normalized_position: 0.4,
                },
            ]
        );
    }

    #[test]
    fn shortcut_mapping_uses_command_c_and_shift_variant() {
        let file_copy = shortcut_action_from_keyboard_event(&key_pressed_event(
            keyboard::Key::Character("c".into()),
            keyboard::Modifiers::COMMAND,
            false,
        ));
        assert_eq!(file_copy, Some(KeyboardShortcutAction::CopyFile));

        let path_copy = shortcut_action_from_keyboard_event(&key_pressed_event(
            keyboard::Key::Character("c".into()),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::SHIFT,
            false,
        ));
        assert_eq!(path_copy, Some(KeyboardShortcutAction::CopyPath));

        let ignored_repeat = shortcut_action_from_keyboard_event(&key_pressed_event(
            keyboard::Key::Character("c".into()),
            keyboard::Modifiers::COMMAND,
            true,
        ));
        assert_eq!(ignored_repeat, None);
    }

    #[test]
    fn collect_available_filter_tags_skips_kind_tags_and_deduplicates() {
        let rows = vec![
            librapix_storage::MediaReadModel {
                media_id: 1,
                source_root_id: 10,
                absolute_path: PathBuf::from("/tmp/a.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 100,
                modified_unix_seconds: Some(10),
                width_px: Some(10),
                height_px: Some(10),
                metadata_status: librapix_storage::IndexedMetadataStatus::Ok,
                tags: vec![
                    "kind:image".to_owned(),
                    "Boss".to_owned(),
                    "campaign".to_owned(),
                ],
            },
            librapix_storage::MediaReadModel {
                media_id: 2,
                source_root_id: 10,
                absolute_path: PathBuf::from("/tmp/b.mp4"),
                media_kind: "video".to_owned(),
                file_size_bytes: 100,
                modified_unix_seconds: Some(20),
                width_px: None,
                height_px: None,
                metadata_status: librapix_storage::IndexedMetadataStatus::Ok,
                tags: vec!["kind:video".to_owned(), "boss".to_owned()],
            },
        ];

        assert_eq!(
            collect_available_filter_tags(&rows),
            vec!["Boss".to_owned(), "campaign".to_owned()]
        );
    }

    #[test]
    fn build_new_media_announcement_uses_latest_new_item() {
        let mut previous = std::collections::HashSet::new();
        previous.insert(10);

        let mut cache = std::collections::HashMap::new();
        cache.insert(
            10,
            CachedDetails {
                absolute_path: PathBuf::from("/tmp/old.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 100,
                modified_unix_seconds: Some(100),
                width_px: Some(10),
                height_px: Some(10),
                detail_thumbnail_path: None,
            },
        );
        cache.insert(
            20,
            CachedDetails {
                absolute_path: PathBuf::from("/tmp/newer.mp4"),
                media_kind: "video".to_owned(),
                file_size_bytes: 200,
                modified_unix_seconds: Some(300),
                width_px: Some(1920),
                height_px: Some(1080),
                detail_thumbnail_path: None,
            },
        );
        cache.insert(
            21,
            CachedDetails {
                absolute_path: PathBuf::from("/tmp/new.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 150,
                modified_unix_seconds: Some(200),
                width_px: Some(800),
                height_px: Some(600),
                detail_thumbnail_path: None,
            },
        );

        let announcement =
            build_new_media_announcement(Translator::new(Locale::EnUs), &previous, &cache)
                .expect("announcement should be generated");
        assert_eq!(announcement.media_id, 20);
        assert_eq!(announcement.additional_count, 1);
        assert!(announcement.metadata_line.contains("Video"));
    }

    #[test]
    fn windows_file_drop_payload_uses_dropfiles_header_and_double_nul() {
        let path = PathBuf::from("/tmp/librapix/clip.png");
        let payload = build_windows_file_drop_payload(&path).expect("payload should be generated");
        let header_size = std::mem::size_of::<WindowsDropFilesHeader>();
        assert!(payload.len() > header_size);

        let header =
            unsafe { std::ptr::read_unaligned(payload.as_ptr().cast::<WindowsDropFilesHeader>()) };
        assert_eq!(header.p_files as usize, header_size);
        assert_eq!(header.f_wide, 1);

        let tail = &payload[header_size..];
        assert_eq!(tail.len() % std::mem::size_of::<u16>(), 0);

        let units = tail
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        assert!(units.ends_with(&[0, 0]));

        let mut expected = windows_path_to_utf16(&path);
        expected.push(0);
        expected.push(0);
        assert_eq!(units, expected);
    }

    #[test]
    fn windows_file_drop_payload_rejects_relative_paths() {
        let err = build_windows_file_drop_payload(Path::new("relative/clip.png"))
            .expect_err("relative path should be rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn projection_loading_key_prefers_search_over_route() {
        assert_eq!(
            projection_loading_key("dragon", Route::Gallery),
            TextKey::LoadingSearchLabel
        );
        assert_eq!(
            projection_loading_key("dragon", Route::Timeline),
            TextKey::LoadingSearchLabel
        );
        assert_eq!(
            projection_loading_key("", Route::Timeline),
            TextKey::LoadingTimelineLabel
        );
        assert_eq!(
            projection_loading_key("", Route::Gallery),
            TextKey::LoadingGalleryLabel
        );
    }

    #[test]
    fn ensure_valid_library_filter_resets_missing_root() {
        let mut app = Librapix::default();
        app.state.replace_library_roots(vec![LibraryRootView {
            id: 7,
            normalized_path: PathBuf::from("/tmp/librapix-root-7"),
            lifecycle: RootLifecycle::Active,
            display_name: Some("Root 7".to_owned()),
        }]);
        app.filter_source_root_id = Some(999);

        ensure_valid_library_filter(&mut app);

        assert_eq!(app.filter_source_root_id, None);
        assert!(app.root_status.contains("Showing all libraries"));
    }

    #[test]
    fn collect_visible_media_ids_prefers_search_then_route_and_dedupes() {
        let mut app = Librapix::default();
        app.state.search_query = "boss".to_owned();
        app.state.active_route = Route::Gallery;

        app.search_items = vec![
            BrowseItem {
                media_id: 10,
                title: "a".to_owned(),
                thumbnail_path: None,
                media_kind: "image".to_owned(),
                metadata_line: String::new(),
                is_group_header: false,
                line: String::new(),
                aspect_ratio: 1.0,
                group_image_count: None,
                group_video_count: None,
            },
            BrowseItem {
                media_id: 11,
                title: "b".to_owned(),
                thumbnail_path: None,
                media_kind: "image".to_owned(),
                metadata_line: String::new(),
                is_group_header: false,
                line: String::new(),
                aspect_ratio: 1.0,
                group_image_count: None,
                group_video_count: None,
            },
        ];
        app.gallery_items = vec![
            BrowseItem {
                media_id: 11,
                title: "dup".to_owned(),
                thumbnail_path: None,
                media_kind: "image".to_owned(),
                metadata_line: String::new(),
                is_group_header: false,
                line: String::new(),
                aspect_ratio: 1.0,
                group_image_count: None,
                group_video_count: None,
            },
            BrowseItem {
                media_id: 12,
                title: "c".to_owned(),
                thumbnail_path: None,
                media_kind: "video".to_owned(),
                metadata_line: String::new(),
                is_group_header: false,
                line: String::new(),
                aspect_ratio: 1.0,
                group_image_count: None,
                group_video_count: None,
            },
        ];

        let visible = collect_visible_media_ids_for_thumbnailing(&app);
        assert_eq!(visible, vec![10, 11, 12]);
    }

    #[test]
    fn thumbnail_retry_delay_uses_bounded_backoff() {
        assert_eq!(thumbnail_retry_delay_ms(1), 1_200);
        assert_eq!(thumbnail_retry_delay_ms(2), 2_400);
        assert_eq!(thumbnail_retry_delay_ms(3), 4_800);
        assert_eq!(thumbnail_retry_delay_ms(6), 19_200);
    }

    #[test]
    fn thumbnail_presentation_uses_loading_state_for_retryable_failures() {
        let mut app = Librapix::default();
        app.thumbnail_states.insert(
            11,
            ThumbnailState::Failed {
                retryable: true,
                last_error: "still being written".to_owned(),
            },
        );

        let presentation = resolve_thumbnail_presentation(&app, 11, None);
        assert_eq!(presentation.state, ThumbnailPresentationState::Loading);
        assert_eq!(presentation.resolved_path, None);
    }

    #[test]
    fn thumbnail_presentation_uses_unavailable_state_for_terminal_failures() {
        let mut app = Librapix::default();
        app.thumbnail_states.insert(
            12,
            ThumbnailState::Failed {
                retryable: false,
                last_error: "unsupported".to_owned(),
            },
        );

        let presentation = resolve_thumbnail_presentation(&app, 12, None);
        assert_eq!(presentation.state, ThumbnailPresentationState::Unavailable);
        assert_eq!(presentation.resolved_path, None);
    }

    #[test]
    fn thumbnail_batch_completion_updates_announcement_preview_path() {
        let mut app = Librapix::default();
        app.background.thumbnail_generation = 3;
        app.background.thumbnail_total = 1;
        app.new_media_announcement = Some(NewMediaAnnouncement {
            media_id: 42,
            title: "shot.png".to_owned(),
            metadata_line: "Image".to_owned(),
            preview_path: None,
            media_kind: "image".to_owned(),
            file_size_bytes: 123,
            modified_unix_seconds: Some(10),
            width_px: Some(10),
            height_px: Some(10),
            absolute_path: PathBuf::from("/tmp/shot.png"),
            additional_count: 0,
        });

        let preview_path = PathBuf::from("/tmp/thumb-42.png");
        let result = ThumbnailBatchResult {
            generation: 3,
            outcomes: vec![ThumbnailWorkOutcome {
                media_id: 42,
                state: ThumbnailState::Ready(preview_path.clone()),
                thumbnail_path: Some(preview_path.clone()),
            }],
            reused: 1,
            ..Default::default()
        };

        let _ = apply_thumbnail_batch_result(&mut app, result);

        assert_eq!(
            app.new_media_announcement
                .as_ref()
                .and_then(|announcement| announcement.preview_path.clone()),
            Some(preview_path)
        );
    }

    #[test]
    fn projection_pending_reason_prefers_filesystem_watch() {
        let mut app = Librapix::default();
        app.background.reconcile_in_flight = true;

        let _ = request_projection_refresh(&mut app, BackgroundWorkReason::UserOrSystem);
        assert_eq!(
            app.background.pending_projection_reason,
            BackgroundWorkReason::UserOrSystem
        );

        let _ = request_projection_refresh(&mut app, BackgroundWorkReason::FilesystemWatch);
        assert_eq!(
            app.background.pending_projection_reason,
            BackgroundWorkReason::FilesystemWatch
        );
    }

    #[test]
    fn scan_result_starts_projection_and_clears_reconcile_handoff() {
        let mut app = Librapix::default();
        app.background.reconcile_generation = 5;
        app.background.reconcile_in_flight = true;
        app.background.pending_projection = true;
        app.background.pending_projection_reason = BackgroundWorkReason::UserOrSystem;

        let result = ScanJobResult {
            generation: 5,
            reason: BackgroundWorkReason::FilesystemWatch,
            indexing_status: "done".to_owned(),
            has_media_changes: true,
            ..Default::default()
        };

        let _ = apply_scan_job_result(&mut app, result);

        assert!(!app.background.reconcile_in_flight);
        assert!(app.background.projection_in_flight);
        assert!(!app.background.pending_projection);
        assert_eq!(
            app.background.pending_projection_reason,
            BackgroundWorkReason::FilesystemWatch
        );
    }

    #[test]
    fn stale_thumbnail_completion_releases_lock_and_resumes_queue() {
        let mut app = Librapix::default();
        let media_id = 42;
        app.background.thumbnail_generation = 9;
        app.background.thumbnail_in_flight = true;
        app.background.thumbnail_total = 1;
        app.background.thumbnail_queue.push_back(ThumbnailWorkItem {
            generation: 9,
            media_id,
            absolute_path: PathBuf::from("/tmp/current.png"),
            media_kind: "image".to_owned(),
            file_size_bytes: 10,
            modified_unix_seconds: Some(123),
            attempt: 0,
        });
        app.background.thumbnail_queued_ids.insert(media_id);
        app.thumbnail_states
            .insert(media_id, ThumbnailState::Queued);

        let stale = ThumbnailBatchResult {
            generation: 8,
            ..Default::default()
        };

        let _ = apply_thumbnail_batch_result(&mut app, stale);

        assert!(app.background.thumbnail_in_flight);
        assert!(app.background.thumbnail_queue.is_empty());
        assert!(matches!(
            app.thumbnail_states.get(&media_id),
            Some(ThumbnailState::Generating)
        ));
    }

    #[test]
    fn finalize_background_flow_keeps_busy_while_reconcile_running() {
        let mut app = Librapix::default();
        set_activity_stage(
            &mut app,
            TextKey::StageRefreshingGalleryLabel,
            String::new(),
            true,
        );
        app.background.reconcile_in_flight = true;

        let _ = finalize_background_flow(&mut app);

        assert!(app.activity_progress.busy);
        assert_eq!(
            app.activity_status,
            app.i18n.text(TextKey::StageRefreshingGalleryLabel)
        );
    }

    #[test]
    fn finalize_background_flow_sets_ready_when_no_work_remains() {
        let mut app = Librapix::default();
        set_activity_stage(
            &mut app,
            TextKey::StageRefreshingGalleryLabel,
            String::new(),
            true,
        );

        let _ = finalize_background_flow(&mut app);

        assert!(!app.activity_progress.busy);
        assert_eq!(app.activity_status, app.i18n.text(TextKey::StageReadyLabel));
    }
}
