#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod assets;
mod format;
mod startup_log;
mod ui;

use chrono::Local;
use iced::keyboard;
use iced::keyboard::key;
use iced::time;
use iced::widget::image::FilterMethod;
use iced::widget::{
    Id, Space, button, column, container, image, mouse_area, operation, progress_bar, responsive,
    row, scrollable, stack, svg, text, text_input, tooltip, vertical_slider,
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
    CatalogMediaRecord, DerivedArtifactKind, DerivedArtifactRecord, DerivedArtifactStatus,
    IndexedMediaWrite, IndexedMetadataStatus, SourceRootLifecycle, SourceRootStatisticsRecord,
    Storage, StorageOpenMetrics, TagKind,
};
use librapix_thumbnails::{
    ThumbnailCancellation, ThumbnailError, VideoThumbnailErrorKind, VideoThumbnailOptions,
    ensure_image_thumbnail, ensure_video_thumbnail_with_options, thumbnail_path,
};
use librapix_video_tools::paths::default_output_file_path;
use librapix_video_tools::{
    CropPosition, Effect as ShortEffect, FfmpegArgs as ShortFfmpegArgs, GenerationStage, Preset,
    ShortGenerationOptions, ShortGenerationRequest, ShortGenerationResult, prepare_generation,
    run_generation,
};
use notify::{EventKind, RecursiveMode, Watcher};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};
use ui::*;

fn main() -> iced::Result {
    let _ = startup_log::init_process_logging();
    startup_log::log_info("app.launch.start", "startup requested");
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
    OpenMakeShortDialog,
    CloseMakeShortDialog,
    MakeShortOutputPathChanged(String),
    MakeShortBrowseOutputPath,
    MakeShortToggleEffect(ShortEffect),
    MakeShortSetCropPosition(CropPosition),
    MakeShortSetAddFade(bool),
    MakeShortSpeedChanged(String),
    MakeShortCrfChanged(String),
    MakeShortSetPreset(Preset),
    RunMakeShort,
    MakeShortPrepared(Result<PreparedShortJob, String>),
    MakeShortGenerated(Result<ShortGenerationResult, String>),
    OpenGeneratedShortFile,
    OpenGeneratedShortFolder,
    IgnoreRuleInputChanged(String),
    IgnoreRuleAdd,
    IgnoreRuleToggleEnabled(i64),
    IgnoreRuleRemove(i64),
    IgnoreRuleStartEdit(i64),
    IgnoreRuleApplyEdit,
    IgnoreRuleCancelEdit,
    StartupRestore,
    StartupReconcileKickoff,
    FilesystemChanged,
    SetFilterMediaKind(Option<String>),
    SetFilterExtension(Option<String>),
    SetFilterTag(Option<String>),
    MinFileSizeInputChanged(String),
    ApplyMinFileSize,
    TimelineScrubChanged(f32),
    TimelineScrubReleased,
    JumpToTimelineAnchor(usize),
    MediaViewportChanged {
        absolute_y: f32,
        max_y: f32,
        viewport_height: f32,
    },
    KeyboardEvent(keyboard::Event),
    HydrateSnapshotComplete(Box<SnapshotHydrateResult>),
    SnapshotApplyTick,
    ViewportSettleTick,
    ScanJobComplete(Box<ScanJobResult>),
    ProjectionJobComplete(Box<ProjectionJobResult>),
    ThumbnailBatchComplete(Box<ThumbnailBatchResult>),
    StartupGalleryContinuationKickoff,
    AutomationTick,
    DeferredThumbnailCatchupKickoff,
    NewMediaPreviewTick,
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
    ShortsOutputDirInputChanged(String),
    ShortsOutputDirBrowse,
    SaveShortsOutputDirSetting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum IntervalMessageKind {
    UpdateCheckTick,
    StartupReconcileKickoff,
    SnapshotApplyTick,
    ViewportSettleTick,
    StartupGalleryContinuationKickoff,
    AutomationTick,
    DeferredThumbnailCatchupKickoff,
    NewMediaPreviewTick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BackgroundWorkReason {
    #[default]
    UserOrSystem,
    FilesystemWatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ProjectionRefreshPolicy {
    #[default]
    Full,
    CurrentSurface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ThumbnailWorkMode {
    #[default]
    StartupPriority,
    BackgroundCatchUp,
}

impl ThumbnailWorkMode {
    fn as_str(self) -> &'static str {
        match self {
            ThumbnailWorkMode::StartupPriority => "startup_priority",
            ThumbnailWorkMode::BackgroundCatchUp => "background_catchup",
        }
    }
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

#[derive(Debug, Clone, Default)]
struct StartupFlowMetrics {
    snapshot_hydrate_started_at: Option<Instant>,
    snapshot_apply_started_at: Option<Instant>,
    reconcile_started_at: Option<Instant>,
    projection_started_at: Option<Instant>,
    startup_thumbnail_started_at: Option<Instant>,
    deferred_thumbnail_started_at: Option<Instant>,
    first_usable_gallery_recorded: bool,
    startup_ready_recorded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActivityIndicatorMode {
    Idle,
    Determinate { total: usize, done: usize },
    Indeterminate,
}

#[derive(Debug, Clone)]
struct SnapshotHydrateInput {
    generation: u64,
    database_file: PathBuf,
    configured_library_roots: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
struct SnapshotHydrateResult {
    generation: u64,
    roots: Vec<LibraryRootView>,
    ignore_rules: Vec<librapix_storage::IgnoreRuleRecord>,
    snapshot: Option<PersistedProjectionSnapshot>,
    snapshot_bytes: usize,
    snapshot_version: Option<u32>,
    snapshot_error: Option<String>,
}

#[derive(Debug, Clone)]
struct PendingSnapshotApply {
    generation: u64,
    gallery_total: usize,
    gallery_total_items: usize,
    gallery_loaded: usize,
    gallery_iter: std::vec::IntoIter<BrowseItem>,
    available_filter_tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct ScanJobInput {
    generation: u64,
    reason: BackgroundWorkReason,
    database_file: PathBuf,
    min_file_size_bytes: u64,
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
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct ProjectionJobInput {
    generation: u64,
    reason: BackgroundWorkReason,
    policy: ProjectionRefreshPolicy,
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
    refreshed_gallery: bool,
    refreshed_timeline: bool,
    gallery_preview_lines: Vec<String>,
    timeline_preview_lines: Vec<String>,
    search_preview_lines: Vec<String>,
    media_cache: HashMap<i64, CachedDetails>,
    available_filter_tags: Vec<String>,
    ignore_rules: Vec<librapix_storage::IgnoreRuleRecord>,
    browse_status: String,
    snapshot_payload: Option<String>,
    thumbnail_candidates: Vec<ThumbnailWorkItem>,
    error: Option<String>,
}

#[derive(Debug, Clone)]
struct ThumbnailWorkItem {
    generation: u64,
    media_id: i64,
    absolute_path: PathBuf,
    media_kind: String,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone)]
struct ThumbnailWorkOutcome {
    media_id: i64,
    thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ThumbnailBatchInput {
    generation: u64,
    batch_id: u64,
    mode: ThumbnailWorkMode,
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    cancellation: ThumbnailCancellation,
    items: Vec<ThumbnailWorkItem>,
}

#[derive(Debug, Clone, Default)]
struct ThumbnailBatchResult {
    generation: u64,
    batch_id: u64,
    mode: ThumbnailWorkMode,
    outcomes: Vec<ThumbnailWorkOutcome>,
    completed_media_ids: Vec<i64>,
    failures: Vec<ThumbnailFailureEvent>,
    attempted: usize,
    image_items: usize,
    video_items: usize,
    cancelled: bool,
    worker_elapsed: Duration,
    worker_finished_at: Option<Instant>,
    dispatched_to_ui_at: Option<Instant>,
    generated: usize,
    reused_exact: usize,
    reused_fallback: usize,
    failed: usize,
    errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThumbnailFailureClass {
    ImageDecode,
    ImageIo,
    VideoFfmpegNotFound,
    VideoSpawnFailed,
    VideoTimedOut,
    VideoExitNonZero,
    VideoMissingOutput,
    Cancelled,
    Unknown,
}

impl ThumbnailFailureClass {
    fn as_str(self) -> &'static str {
        match self {
            ThumbnailFailureClass::ImageDecode => "image_decode",
            ThumbnailFailureClass::ImageIo => "image_io",
            ThumbnailFailureClass::VideoFfmpegNotFound => "video_ffmpeg_not_found",
            ThumbnailFailureClass::VideoSpawnFailed => "video_spawn_failed",
            ThumbnailFailureClass::VideoTimedOut => "video_timed_out",
            ThumbnailFailureClass::VideoExitNonZero => "video_exit_non_zero",
            ThumbnailFailureClass::VideoMissingOutput => "video_missing_output",
            ThumbnailFailureClass::Cancelled => "cancelled",
            ThumbnailFailureClass::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
struct ThumbnailFailureEvent {
    media_id: i64,
    media_kind: String,
    failure_class: ThumbnailFailureClass,
    detail: String,
    command_line: Option<String>,
    ffmpeg_path: Option<PathBuf>,
    exit_code: Option<i32>,
    stderr_summary: Option<String>,
    timeout_ms: Option<u128>,
    hard_failure: bool,
    disable_video_for_session: bool,
}

#[derive(Debug, Clone)]
struct ThumbnailRetryState {
    attempts: u32,
    next_retry_at: Instant,
    failure_class: ThumbnailFailureClass,
    last_error: String,
}

#[derive(Debug, Clone, Default)]
struct ArtifactValidationSummary {
    accepted: usize,
    rejected_missing_path: usize,
    rejected_missing_file: usize,
}

#[derive(Debug, Clone, Default)]
struct ThumbnailLookupSummary {
    requested_media: usize,
    priority_media: usize,
    exact_catalog_reused: usize,
    exact_deterministic_reused: usize,
    fallback_catalog_reused: usize,
    fallback_deterministic_reused: usize,
    priority_placeholder: usize,
    scheduled_generation: usize,
    rejected_gallery_missing_path: usize,
    rejected_gallery_missing_file: usize,
    rejected_detail_missing_path: usize,
    rejected_detail_missing_file: usize,
}

#[derive(Debug, Clone, Default)]
struct ProjectionThumbnailLookup {
    resolved_paths: HashMap<i64, PathBuf>,
    reusable_media_ids: HashSet<i64>,
    summary: ThumbnailLookupSummary,
}

struct ProjectionThumbnailLookupInput<'a> {
    generation: u64,
    all_rows: &'a [CatalogMediaRecord],
    row_lookup: &'a HashMap<i64, &'a CatalogMediaRecord>,
    gallery_artifacts: &'a [DerivedArtifactRecord],
    detail_artifacts: &'a [DerivedArtifactRecord],
    thumbnails_dir: &'a Path,
    active_route: Route,
    search_query: &'a str,
    gallery_items: &'a [BrowseItem],
    timeline_items: &'a [BrowseItem],
    search_items: &'a [BrowseItem],
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AutomationAction {
    OpenGallery,
    OpenTimeline,
    SetFilterMediaKind(Option<String>),
    SelectFirstVisible,
}

impl AutomationAction {
    fn label(&self) -> String {
        match self {
            Self::OpenGallery => "gallery".to_owned(),
            Self::OpenTimeline => "timeline".to_owned(),
            Self::SetFilterMediaKind(Some(kind)) => format!("filter_kind:{kind}"),
            Self::SetFilterMediaKind(None) => "filter_kind:none".to_owned(),
            Self::SelectFirstVisible => "select_first".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AutomationStep {
    wait_ms: u64,
    action: AutomationAction,
}

#[derive(Debug, Clone)]
struct AutomationRunner {
    steps: VecDeque<AutomationStep>,
    due_at: Option<Instant>,
    poll_interval: Duration,
}

impl AutomationRunner {
    fn from_env() -> Option<Self> {
        let raw = std::env::var("LIBRAPIX_AUTOMATION_SCRIPT").ok()?;
        let steps = parse_automation_script(&raw);
        if steps.is_empty() {
            startup_log::log_warn(
                "automation.script.ignored",
                "reason=empty_or_invalid env=LIBRAPIX_AUTOMATION_SCRIPT",
            );
            return None;
        }

        startup_log::log_info(
            "automation.script.loaded",
            &format!("steps={} script={raw}", steps.len()),
        );
        Some(Self {
            steps: steps.into(),
            due_at: None,
            poll_interval: Duration::from_millis(100),
        })
    }
}

#[derive(Debug, Clone, Default)]
struct BackgroundCoordinator {
    snapshot_generation: u64,
    reconcile_generation: u64,
    projection_generation: u64,
    thumbnail_generation: u64,
    snapshot_apply: Option<PendingSnapshotApply>,
    snapshot_loaded: bool,
    startup_reconcile_queued: bool,
    startup_reconcile_due_at: Option<Instant>,
    startup_ready: bool,
    startup_deferred_gallery_refresh: bool,
    startup_deferred_timeline_refresh: bool,
    startup_gallery_continuation_due_at: Option<Instant>,
    reconcile_in_flight: bool,
    projection_in_flight: bool,
    thumbnail_in_flight: bool,
    pending_reconcile: bool,
    pending_reconcile_reason: BackgroundWorkReason,
    pending_projection: bool,
    pending_projection_reason: BackgroundWorkReason,
    deferred_thumbnail_due_at: Option<Instant>,
    automation: Option<AutomationRunner>,
    thumbnail_cancel_generation: Arc<AtomicU64>,
    thumbnail_batch_id: u64,
    thumbnail_queue: VecDeque<ThumbnailWorkItem>,
    thumbnail_queued_ids: HashSet<i64>,
    deferred_thumbnail_queue: VecDeque<ThumbnailWorkItem>,
    thumbnail_done: usize,
    thumbnail_total: usize,
    thumbnail_generated: usize,
    thumbnail_reused_exact: usize,
    thumbnail_reused_fallback: usize,
    thumbnail_failed: usize,
    thumbnail_mode: ThumbnailWorkMode,
    thumbnail_retry_state: HashMap<i64, ThumbnailRetryState>,
    video_thumbnails_disabled_reason: Option<String>,
    video_thumbnails_disabled_ffmpeg: Option<PathBuf>,
    thumbnail_result_window_started_at: Option<Instant>,
    thumbnail_result_window_batches: usize,
    thumbnail_result_window_outcomes: usize,
    thumbnail_result_window_failures: usize,
    thumbnail_refresh_requests_while_active: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedProjectionSnapshot {
    version: u32,
    gallery_items: Vec<BrowseItem>,
    gallery_total_items: usize,
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

#[derive(Debug, Clone)]
struct CachedGalleryLayout {
    generation: u64,
    width_key: u32,
    item_count: usize,
    first_media_id: Option<i64>,
    last_media_id: Option<i64>,
    rows: Arc<[JustifiedRowLayout]>,
}

#[derive(Debug, Clone)]
struct CachedTimelineSectionLayout {
    header_index: usize,
    media_start: usize,
    row_layouts: Arc<[JustifiedRowLayout]>,
    section_height: f32,
}

#[derive(Debug, Clone)]
struct CachedTimelineLayout {
    generation: u64,
    width_key: u32,
    item_count: usize,
    first_media_id: Option<i64>,
    last_media_id: Option<i64>,
    total_items: usize,
    total_groups: usize,
    total_rows: usize,
    sections: Arc<[CachedTimelineSectionLayout]>,
}

#[derive(Debug, Default)]
struct MediaLayoutCache {
    gallery: Option<Arc<CachedGalleryLayout>>,
    timeline: Option<Arc<CachedTimelineLayout>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaSurfaceKind {
    Gallery,
    Timeline,
}

impl MediaSurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::Gallery => "gallery",
            Self::Timeline => "timeline",
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct DragSurfacePreviewState {
    frozen_width_key: Option<u32>,
    last_measured_width_key: Option<u32>,
    width_change_count: usize,
    suppressed_rebuilds: usize,
    anomaly_logged: bool,
}

#[derive(Debug, Clone, Copy, Default)]
struct DragLayoutPreviewState {
    gallery: DragSurfacePreviewState,
    timeline: DragSurfacePreviewState,
}

impl DragLayoutPreviewState {
    fn surface(self, kind: MediaSurfaceKind) -> DragSurfacePreviewState {
        match kind {
            MediaSurfaceKind::Gallery => self.gallery,
            MediaSurfaceKind::Timeline => self.timeline,
        }
    }

    fn surface_mut(&mut self, kind: MediaSurfaceKind) -> &mut DragSurfacePreviewState {
        match kind {
            MediaSurfaceKind::Gallery => &mut self.gallery,
            MediaSurfaceKind::Timeline => &mut self.timeline,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ViewportDragState {
    active: bool,
    started_at: Option<Instant>,
    last_event_at: Option<Instant>,
    last_logged_at: Option<Instant>,
    update_count: usize,
    applied_updates: usize,
    coalesced_updates: usize,
    latest_replacements: usize,
    max_y_preview_skips: usize,
    max_step_delta_px: f32,
    last_applied_at: Option<Instant>,
    pending_viewport: Option<ViewportSnapshot>,
    frozen_max_y: Option<f32>,
    candidate_started_at: Option<Instant>,
    candidate_last_event_at: Option<Instant>,
    candidate_event_count: usize,
    candidate_origin_y: f32,
    candidate_origin_height: f32,
    mode: ViewportDragMode,
}

#[derive(Debug, Clone, Copy)]
struct ViewportSnapshot {
    absolute_y: f32,
    max_y: f32,
    viewport_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ViewportDragMode {
    #[default]
    LivePreview,
    SettleFirstPreview,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubLatestReleaseResponse {
    tag_name: String,
    html_url: String,
    prerelease: bool,
    draft: bool,
}

#[derive(Debug, Clone)]
struct MakeShortDialogState {
    open: bool,
    input_file: Option<PathBuf>,
    output_file_input: String,
    effects: Vec<ShortEffect>,
    crop_position: CropPosition,
    add_fade: bool,
    speed_input: String,
    crf_input: String,
    preset: Preset,
    validation_error: Option<String>,
    run_state: MakeShortRunState,
}

impl Default for MakeShortDialogState {
    fn default() -> Self {
        Self {
            open: false,
            input_file: None,
            output_file_input: String::new(),
            effects: vec![ShortEffect::Enhanced],
            crop_position: CropPosition::Center,
            add_fade: false,
            speed_input: "1.0".to_owned(),
            crf_input: "18".to_owned(),
            preset: Preset::Medium,
            validation_error: None,
            run_state: MakeShortRunState::Idle,
        }
    }
}

#[derive(Debug, Clone, Default)]
enum MakeShortRunState {
    #[default]
    Idle,
    Running {
        stage: GenerationStage,
        status: String,
    },
    Success {
        output_file: PathBuf,
    },
    Failed {
        summary: String,
        details: String,
    },
}

#[derive(Debug, Clone)]
struct PreparedShortJob {
    prepared: ShortFfmpegArgs,
}

struct Librapix {
    state: AppState,
    i18n: Translator,
    theme_preference: ThemePreference,
    runtime: RuntimeContext,
    startup_log_path: Option<PathBuf>,
    thumbnail_status: String,
    details_tag_input: String,
    details_lines: Vec<String>,
    details_action_status: String,
    details_preview_path: Option<PathBuf>,
    details_title: String,
    details_tags: Vec<DetailsTagChip>,
    details_editing_tag: Option<String>,
    details_loaded_media_ids: HashSet<i64>,
    make_short_dialog: MakeShortDialogState,
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
    shorts_output_dir_input: String,
    media_cache: HashMap<i64, CachedDetails>,
    background: BackgroundCoordinator,
    diagnostics_lines: Vec<String>,
    diagnostics_events: Vec<String>,
    media_scroll_absolute_y: f32,
    media_scroll_max_y: f32,
    media_viewport_height: f32,
    timeline_scrub_value: f32,
    timeline_scrubbing: bool,
    timeline_scrub_anchor_index: Option<usize>,
    timeline_scroll_max_y: f32,
    browse_layout_generation: u64,
    layout_cache: RefCell<MediaLayoutCache>,
    drag_layout_preview: RefCell<DragLayoutPreviewState>,
    viewport_drag: ViewportDragState,
    last_viewport_drag_settled_at: Option<Instant>,
    new_media_announcement: Option<NewMediaAnnouncement>,
    new_media_preview_loading_phase: usize,
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
    startup_metrics: StartupFlowMetrics,
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
const GALLERY_THUMB_VARIANT: &str = "gallery-400";
const DETAIL_THUMB_VARIANT: &str = "detail-800";
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
const STARTUP_RECONCILE_DELAY_MS: u64 = 550;
const STARTUP_RECONCILE_TICK_INTERVAL: Duration = Duration::from_millis(120);
const STARTUP_GALLERY_CONTINUATION_DELAY_MS: u64 = 350;
const STARTUP_GALLERY_CONTINUATION_TICK_INTERVAL: Duration = Duration::from_millis(120);
const SNAPSHOT_APPLY_TICK_INTERVAL: Duration = Duration::from_millis(12);
const DEFERRED_THUMBNAIL_DELAY_MS: u64 = 800;
const DEFERRED_THUMBNAIL_TICK_INTERVAL: Duration = Duration::from_millis(140);
const SNAPSHOT_APPLY_CHUNK_SIZE: usize = 240;
const STARTUP_SNAPSHOT_GALLERY_LIMIT: usize = 160;
const STARTUP_IMAGE_THUMBNAIL_BATCH_SIZE: usize = 6;
const BACKGROUND_IMAGE_THUMBNAIL_BATCH_SIZE: usize = 3;
const MAX_VIDEO_THUMBNAILS_PER_BATCH: usize = 1;
const STARTUP_THUMBNAIL_PRIORITY_LIMIT: usize = 96;
const STARTUP_CACHE_WARM_LIMIT: usize = 160;
const THUMBNAIL_RESULT_LOG_WINDOW: Duration = Duration::from_secs(1);
const IMAGE_THUMBNAIL_RETRY_COOLDOWN: Duration = Duration::from_secs(15 * 60);
const VIDEO_THUMBNAIL_RETRY_COOLDOWN: Duration = Duration::from_secs(10 * 60);
const VIDEO_SESSION_DISABLE_COOLDOWN: Duration = Duration::from_secs(30 * 60);
const PROJECTION_SNAPSHOT_KEY: &str = "default";
const PROJECTION_SNAPSHOT_VERSION: u32 = 2;
const AUTO_UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MANUAL_UPDATE_CHECK_COOLDOWN: Duration = Duration::from_secs(5 * 60);
const MEDIA_VIRTUAL_OVERSCAN_PX: f32 = 1200.0;
const MEDIA_VIRTUAL_OVERSCAN_DRAG_PX: f32 = 400.0;
const TIMELINE_GROUP_HEADER_HEIGHT: f32 = 40.0;
const AUTOMATION_TICK_INTERVAL: Duration = Duration::from_millis(100);
const NEW_MEDIA_PREVIEW_TICK_INTERVAL: Duration = Duration::from_millis(180);
const VIEWPORT_SETTLE_TICK_INTERVAL: Duration = Duration::from_millis(16);
const VIEWPORT_SETTLE_DELAY: Duration = Duration::from_millis(420);
const VIEWPORT_SETTLE_DELAY_LARGE_JUMP: Duration = Duration::from_millis(560);
const VIEWPORT_ACTIVE_LOG_INTERVAL: Duration = Duration::from_millis(200);
const VIEWPORT_DRAG_APPLY_INTERVAL: Duration = Duration::from_millis(16);
const VIEWPORT_DRAG_THRASH_WINDOW: Duration = Duration::from_millis(700);
const VIEWPORT_DRAG_ACTIVATION_WINDOW: Duration = Duration::from_millis(220);
const VIEWPORT_DRAG_ACTIVATION_EVENTS: usize = 3;
const VIEWPORT_DRAG_FAST_ACTIVATION_EVENTS: usize = 2;
const VIEWPORT_DRAG_ACTIVATION_DELTA_PX: f32 = 120.0;
const VIEWPORT_DRAG_FAST_ACTIVATION_DELTA_PX: f32 = 280.0;
const VIEWPORT_DRAG_SETTLE_FIRST_DELTA_PX: f32 = 1_200.0;
const VIEWPORT_DRAG_PREVIEW_IDLE_APPLY_DELAY: Duration = Duration::from_millis(140);
const VIEWPORT_DRAG_HEIGHT_STABILITY_PX: f32 = 4.0;
const VIEWPORT_SETTLE_LARGE_JUMP_THRESHOLD_PX: f32 = 1_500.0;
static LARGE_SURFACE_RENDER_LOGS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[derive(Debug, Clone)]
struct RuntimeContext {
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    config_file: PathBuf,
    configured_library_roots: Vec<PathBuf>,
    default_shorts_output_dir: Option<PathBuf>,
}

impl Default for Librapix {
    fn default() -> Self {
        let bootstrap = bootstrap_runtime();
        let mut app = Self {
            state: AppState::default(),
            i18n: Translator::new(bootstrap.locale),
            theme_preference: bootstrap.theme_preference,
            runtime: RuntimeContext {
                database_file: bootstrap.database_file,
                thumbnails_dir: bootstrap.thumbnails_dir,
                config_file: bootstrap.config_file,
                configured_library_roots: bootstrap.configured_library_roots,
                default_shorts_output_dir: bootstrap.default_shorts_output_dir.clone(),
            },
            startup_log_path: startup_log::active_log_path(),
            thumbnail_status: String::new(),
            details_tag_input: String::new(),
            details_lines: Vec::new(),
            details_action_status: String::new(),
            details_preview_path: None,
            details_title: String::new(),
            details_tags: Vec::new(),
            details_editing_tag: None,
            details_loaded_media_ids: HashSet::new(),
            make_short_dialog: MakeShortDialogState::default(),
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
            shorts_output_dir_input: bootstrap
                .default_shorts_output_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            media_cache: HashMap::new(),
            background: BackgroundCoordinator::default(),
            diagnostics_lines: Vec::new(),
            diagnostics_events: Vec::new(),
            media_scroll_absolute_y: 0.0,
            media_scroll_max_y: 0.0,
            media_viewport_height: 0.0,
            timeline_scrub_value: 0.0,
            timeline_scrubbing: false,
            timeline_scrub_anchor_index: None,
            timeline_scroll_max_y: 0.0,
            browse_layout_generation: 0,
            layout_cache: RefCell::new(MediaLayoutCache::default()),
            drag_layout_preview: RefCell::new(DragLayoutPreviewState::default()),
            viewport_drag: ViewportDragState::default(),
            last_viewport_drag_settled_at: None,
            new_media_announcement: None,
            new_media_preview_loading_phase: 0,
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
            startup_metrics: StartupFlowMetrics::default(),
        };
        app.background.automation = AutomationRunner::from_env();
        refresh_ignore_rules(&mut app);
        set_activity_ready(&mut app);
        app.background.startup_ready = true;
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
    let update_tick_subscription = interval_subscription(
        true,
        UPDATE_CHECK_TICK_INTERVAL,
        IntervalMessageKind::UpdateCheckTick,
    );
    let startup_reconcile_subscription = interval_subscription(
        app.background.startup_reconcile_due_at.is_some(),
        STARTUP_RECONCILE_TICK_INTERVAL,
        IntervalMessageKind::StartupReconcileKickoff,
    );
    let startup_gallery_continuation_subscription = interval_subscription(
        app.background.startup_gallery_continuation_due_at.is_some(),
        STARTUP_GALLERY_CONTINUATION_TICK_INTERVAL,
        IntervalMessageKind::StartupGalleryContinuationKickoff,
    );
    let snapshot_apply_subscription = interval_subscription(
        app.background.snapshot_apply.is_some(),
        SNAPSHOT_APPLY_TICK_INTERVAL,
        IntervalMessageKind::SnapshotApplyTick,
    );
    let viewport_settle_subscription = interval_subscription(
        app.viewport_drag.active,
        VIEWPORT_SETTLE_TICK_INTERVAL,
        IntervalMessageKind::ViewportSettleTick,
    );
    let automation_subscription = interval_subscription(
        app.background
            .automation
            .as_ref()
            .is_some_and(|runner| runner.due_at.is_some()),
        AUTOMATION_TICK_INTERVAL,
        IntervalMessageKind::AutomationTick,
    );
    let new_media_preview_subscription = interval_subscription(
        app.new_media_announcement
            .as_ref()
            .is_some_and(|announcement| announcement.preview_path.is_none()),
        NEW_MEDIA_PREVIEW_TICK_INTERVAL,
        IntervalMessageKind::NewMediaPreviewTick,
    );
    let deferred_thumbnail_subscription = interval_subscription(
        app.background.deferred_thumbnail_due_at.is_some(),
        DEFERRED_THUMBNAIL_TICK_INTERVAL,
        IntervalMessageKind::DeferredThumbnailCatchupKickoff,
    );

    let roots = app
        .state
        .library_roots
        .iter()
        .filter(|root| matches!(root.lifecycle, RootLifecycle::Active))
        .map(|root| root.normalized_path.clone())
        .collect::<Vec<_>>();

    if roots.is_empty() {
        Subscription::batch(vec![
            keyboard_subscription,
            update_tick_subscription,
            startup_reconcile_subscription,
            startup_gallery_continuation_subscription,
            snapshot_apply_subscription,
            viewport_settle_subscription,
            automation_subscription,
            new_media_preview_subscription,
            deferred_thumbnail_subscription,
        ])
    } else {
        Subscription::batch(vec![
            keyboard_subscription,
            update_tick_subscription,
            startup_reconcile_subscription,
            startup_gallery_continuation_subscription,
            snapshot_apply_subscription,
            viewport_settle_subscription,
            automation_subscription,
            new_media_preview_subscription,
            deferred_thumbnail_subscription,
            Subscription::run_with(WatchSubscriptionConfig { roots }, watch_filesystem),
        ])
    }
}

fn interval_subscription(
    enabled: bool,
    interval: Duration,
    message: IntervalMessageKind,
) -> Subscription<Message> {
    if enabled {
        time::every(interval).with(message).map(interval_message)
    } else {
        Subscription::none()
    }
}

fn interval_message((message, _instant): (IntervalMessageKind, Instant)) -> Message {
    match message {
        IntervalMessageKind::UpdateCheckTick => Message::UpdateCheckTick,
        IntervalMessageKind::StartupReconcileKickoff => Message::StartupReconcileKickoff,
        IntervalMessageKind::StartupGalleryContinuationKickoff => {
            Message::StartupGalleryContinuationKickoff
        }
        IntervalMessageKind::SnapshotApplyTick => Message::SnapshotApplyTick,
        IntervalMessageKind::ViewportSettleTick => Message::ViewportSettleTick,
        IntervalMessageKind::AutomationTick => Message::AutomationTick,
        IntervalMessageKind::DeferredThumbnailCatchupKickoff => {
            Message::DeferredThumbnailCatchupKickoff
        }
        IntervalMessageKind::NewMediaPreviewTick => Message::NewMediaPreviewTick,
    }
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
            if watcher.watch(root, RecursiveMode::Recursive).is_err() {}
        }

        let mut last_signal = Instant::now();
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
                    if should_trigger && last_signal.elapsed().as_millis() > 500 {
                        last_signal = Instant::now();
                        let _ = output.send(Message::FilesystemChanged).await;
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
        Message::OpenMakeShortDialog => "OpenMakeShortDialog".into(),
        Message::CloseMakeShortDialog => "CloseMakeShortDialog".into(),
        Message::MakeShortOutputPathChanged(v) => {
            format!("MakeShortOutputPathChanged({})", v.len())
        }
        Message::MakeShortBrowseOutputPath => "MakeShortBrowseOutputPath".into(),
        Message::MakeShortToggleEffect(effect) => {
            format!("MakeShortToggleEffect({})", effect.as_str())
        }
        Message::MakeShortSetCropPosition(_) => "MakeShortSetCropPosition".into(),
        Message::MakeShortSetAddFade(value) => format!("MakeShortSetAddFade({value})"),
        Message::MakeShortSpeedChanged(v) => format!("MakeShortSpeedChanged({})", v.len()),
        Message::MakeShortCrfChanged(v) => format!("MakeShortCrfChanged({})", v.len()),
        Message::MakeShortSetPreset(_) => "MakeShortSetPreset".into(),
        Message::RunMakeShort => "RunMakeShort".into(),
        Message::MakeShortPrepared(result) => {
            if result.is_ok() {
                "MakeShortPrepared(ok)".into()
            } else {
                "MakeShortPrepared(err)".into()
            }
        }
        Message::MakeShortGenerated(result) => {
            if result.is_ok() {
                "MakeShortGenerated(ok)".into()
            } else {
                "MakeShortGenerated(err)".into()
            }
        }
        Message::OpenGeneratedShortFile => "OpenGeneratedShortFile".into(),
        Message::OpenGeneratedShortFolder => "OpenGeneratedShortFolder".into(),
        Message::IgnoreRuleInputChanged(v) => format!("IgnoreRuleInputChanged({})", v.len()),
        Message::IgnoreRuleAdd => "IgnoreRuleAdd".into(),
        Message::IgnoreRuleToggleEnabled(id) => format!("IgnoreRuleToggleEnabled({id})"),
        Message::IgnoreRuleRemove(id) => format!("IgnoreRuleRemove({id})"),
        Message::IgnoreRuleStartEdit(id) => format!("IgnoreRuleStartEdit({id})"),
        Message::IgnoreRuleApplyEdit => "IgnoreRuleApplyEdit".into(),
        Message::IgnoreRuleCancelEdit => "IgnoreRuleCancelEdit".into(),
        Message::StartupRestore => "StartupRestore".into(),
        Message::StartupReconcileKickoff => "StartupReconcileKickoff".into(),
        Message::FilesystemChanged => "FilesystemChanged".into(),
        Message::SetFilterMediaKind(k) => format!("SetFilterMediaKind({:?})", k.as_deref()),
        Message::SetFilterExtension(e) => format!("SetFilterExtension({:?})", e.as_deref()),
        Message::SetFilterTag(tag) => format!("SetFilterTag({:?})", tag.as_deref()),
        Message::MinFileSizeInputChanged(v) => format!("MinFileSizeInputChanged({})", v.len()),
        Message::ApplyMinFileSize => "ApplyMinFileSize".into(),
        Message::TimelineScrubChanged(value) => format!("TimelineScrubChanged({value:.3})"),
        Message::TimelineScrubReleased => "TimelineScrubReleased".into(),
        Message::JumpToTimelineAnchor(index) => format!("JumpToTimelineAnchor({index})"),
        Message::MediaViewportChanged {
            absolute_y,
            max_y,
            viewport_height,
        } => {
            format!("MediaViewportChanged({absolute_y:.1}/{max_y:.1}/{viewport_height:.1})")
        }
        Message::KeyboardEvent(_) => "KeyboardEvent".into(),
        Message::HydrateSnapshotComplete(_) => "HydrateSnapshotComplete".into(),
        Message::SnapshotApplyTick => "SnapshotApplyTick".into(),
        Message::ViewportSettleTick => "ViewportSettleTick".into(),
        Message::ScanJobComplete(_) => "ScanJobComplete".into(),
        Message::ProjectionJobComplete(_) => "ProjectionJobComplete".into(),
        Message::ThumbnailBatchComplete(_) => "ThumbnailBatchComplete".into(),
        Message::StartupGalleryContinuationKickoff => "StartupGalleryContinuationKickoff".into(),
        Message::AutomationTick => "AutomationTick".into(),
        Message::DeferredThumbnailCatchupKickoff => "DeferredThumbnailCatchupKickoff".into(),
        Message::NewMediaPreviewTick => "NewMediaPreviewTick".into(),
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
        Message::ShortsOutputDirInputChanged(v) => {
            format!("ShortsOutputDirInputChanged({})", v.len())
        }
        Message::ShortsOutputDirBrowse => "ShortsOutputDirBrowse".into(),
        Message::SaveShortsOutputDirSetting => "SaveShortsOutputDirSetting".into(),
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

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Gallery => "gallery",
        Route::Timeline => "timeline",
    }
}

fn format_optional_str(value: Option<&str>) -> &str {
    value.unwrap_or("none")
}

fn filter_state_summary(app: &Librapix) -> String {
    format!(
        "kind={} extension={} tag={} root_id={} search_len={}",
        format_optional_str(app.filter_media_kind.as_deref()),
        format_optional_str(app.filter_extension.as_deref()),
        format_optional_str(app.filter_tag.as_deref()),
        app.filter_source_root_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        app.state.search_query.trim().len(),
    )
}

fn route_item_count(app: &Librapix, route: Route) -> usize {
    match route {
        Route::Gallery => app.gallery_items.len(),
        Route::Timeline => app.timeline_items.len(),
    }
}

fn interaction_slow_threshold_ms(duration: Duration) -> Option<u128> {
    let elapsed = duration.as_millis();
    if elapsed >= 500 {
        Some(500)
    } else if elapsed >= 250 {
        Some(250)
    } else if elapsed >= 100 {
        Some(100)
    } else if elapsed >= 50 {
        Some(50)
    } else {
        None
    }
}

fn log_interaction_duration(event: &str, duration: Duration, detail: &str) {
    let detail = if detail.trim().is_empty() {
        format!("elapsed_ms={}", duration.as_millis())
    } else {
        format!("{detail} elapsed_ms={}", duration.as_millis())
    };
    startup_log::log_info(event, &detail);
    if let Some(threshold_ms) = interaction_slow_threshold_ms(duration) {
        startup_log::log_warn(
            &format!("{event}.slow"),
            &format!("threshold_ms={threshold_ms} {detail}"),
        );
    }
}

fn selected_media_context(app: &Librapix, media_id: i64) -> String {
    let cache = app.media_cache.get(&media_id);
    let browse_thumbnail = browse_thumbnail_path(app, media_id);
    let path = cache
        .map(|details| details.absolute_path.display().to_string())
        .unwrap_or_default();
    let kind = cache
        .map(|details| details.media_kind.as_str())
        .unwrap_or("unknown");
    format!(
        "media_id={} kind={} path={} cached={} detail_thumb_cached={} browse_thumb_present={} first_open={}",
        media_id,
        kind,
        path,
        cache.is_some(),
        cache
            .and_then(|details| details.detail_thumbnail_path.as_ref())
            .is_some(),
        browse_thumbnail.is_some(),
        !app.details_loaded_media_ids.contains(&media_id),
    )
}

fn selected_media_is_video(app: &Librapix) -> bool {
    let Some(media_id) = app.state.selected_media_id else {
        return false;
    };
    app.media_cache
        .get(&media_id)
        .is_some_and(|details| details.media_kind.eq_ignore_ascii_case("video"))
}

fn close_all_dialogs(app: &mut Librapix) {
    app.filter_dialog_open = false;
    app.settings_open = false;
    app.about_open = false;
    app.library_dialog_open = false;
    app.library_stats_dialog_open = false;
    app.make_short_dialog.open = false;
    app.new_media_announcement = None;
    app.new_media_preview_loading_phase = 0;
}

fn update(app: &mut Librapix, message: Message) -> Task<Message> {
    log_diagnostic_event(app, &message_event_label(&message));

    match message {
        Message::OpenGallery => {
            let started_at = Instant::now();
            let from_route = app.state.active_route;
            startup_log::log_info(
                "interaction.route_switch.request.start",
                &format!(
                    "from={} to=gallery reason=user_tab_click from_items={} to_items={} startup_deferred_gallery={} startup_deferred_timeline={} {}",
                    route_name(from_route),
                    route_item_count(app, from_route),
                    route_item_count(app, Route::Gallery),
                    app.background.startup_deferred_gallery_refresh,
                    app.background.startup_deferred_timeline_refresh,
                    filter_state_summary(app),
                ),
            );
            app.state.apply(AppMessage::OpenGallery);
            app.timeline_scrubbing = false;
            if app.background.startup_deferred_gallery_refresh {
                startup_log::log_info(
                    "interaction.route_switch.request.end",
                    &format!(
                        "from={} to=gallery deferred_projection=true startup_deferred_gallery={} startup_deferred_timeline={} {} elapsed_ms={}",
                        route_name(from_route),
                        app.background.startup_deferred_gallery_refresh,
                        app.background.startup_deferred_timeline_refresh,
                        filter_state_summary(app),
                        started_at.elapsed().as_millis(),
                    ),
                );
                return request_projection_refresh_with_context(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    "route_switch",
                );
            }
            log_interaction_duration(
                "interaction.route_switch.request.end",
                started_at.elapsed(),
                &format!(
                    "from={} to=gallery deferred_projection=false startup_deferred_gallery={} startup_deferred_timeline={} {}",
                    route_name(from_route),
                    app.background.startup_deferred_gallery_refresh,
                    app.background.startup_deferred_timeline_refresh,
                    filter_state_summary(app),
                ),
            );
        }
        Message::OpenTimeline => {
            let started_at = Instant::now();
            let from_route = app.state.active_route;
            startup_log::log_info(
                "interaction.route_switch.request.start",
                &format!(
                    "from={} to=timeline reason=user_tab_click from_items={} to_items={} startup_deferred_gallery={} startup_deferred_timeline={} {}",
                    route_name(from_route),
                    route_item_count(app, from_route),
                    route_item_count(app, Route::Timeline),
                    app.background.startup_deferred_gallery_refresh,
                    app.background.startup_deferred_timeline_refresh,
                    filter_state_summary(app),
                ),
            );
            app.state.apply(AppMessage::OpenTimeline);
            app.timeline_scrubbing = false;
            sync_timeline_scrub_selection(app, app.timeline_scrub_value);
            if app.background.startup_deferred_timeline_refresh {
                startup_log::log_info(
                    "interaction.route_switch.request.end",
                    &format!(
                        "from={} to=timeline deferred_projection=true startup_deferred_gallery={} startup_deferred_timeline={} {} elapsed_ms={}",
                        route_name(from_route),
                        app.background.startup_deferred_gallery_refresh,
                        app.background.startup_deferred_timeline_refresh,
                        filter_state_summary(app),
                        started_at.elapsed().as_millis(),
                    ),
                );
                return request_projection_refresh_with_context(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    "route_switch",
                );
            }
            log_interaction_duration(
                "interaction.route_switch.request.end",
                started_at.elapsed(),
                &format!(
                    "from={} to=timeline deferred_projection=false startup_deferred_gallery={} startup_deferred_timeline={} {}",
                    route_name(from_route),
                    app.background.startup_deferred_gallery_refresh,
                    app.background.startup_deferred_timeline_refresh,
                    filter_state_summary(app),
                ),
            );
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
            startup_log::log_info(
                "interaction.filter_change.request.start",
                &format!(
                    "trigger=search previous_route={} {}",
                    route_name(app.state.active_route),
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "search_query",
            );
        }
        Message::RunTimelineProjection => {
            startup_log::log_info(
                "interaction.route_switch.request.start",
                &format!(
                    "from={} to=timeline reason=explicit_projection_button from_items={} to_items={} {}",
                    route_name(app.state.active_route),
                    route_item_count(app, app.state.active_route),
                    route_item_count(app, Route::Timeline),
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "route_switch",
            );
        }
        Message::RunGalleryProjection => {
            startup_log::log_info(
                "interaction.route_switch.request.start",
                &format!(
                    "from={} to=gallery reason=explicit_projection_button from_items={} to_items={} {}",
                    route_name(app.state.active_route),
                    route_item_count(app, app.state.active_route),
                    route_item_count(app, Route::Gallery),
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "route_switch",
            );
        }
        Message::SelectMedia(media_id) => {
            let started_at = Instant::now();
            startup_log::log_info(
                "interaction.media_select.request.start",
                &selected_media_context(app, media_id),
            );
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
                startup_log::log_info(
                    "interaction.media_select.request.end",
                    &format!(
                        "{} double_click=true",
                        selected_media_context(app, media_id)
                    ),
                );
                open_selected_path(app, false);
            } else {
                app.state.apply(AppMessage::SetSelectedMedia);
                app.state.set_selected_media(Some(media_id));
                load_media_details_cached(app);
                log_interaction_duration(
                    "interaction.media_select.request.end",
                    started_at.elapsed(),
                    &format!(
                        "{} double_click=false details_status={} details_lines={} details_tags={}",
                        selected_media_context(app, media_id),
                        app.details_action_status,
                        app.details_lines.len(),
                        app.details_tags.len(),
                    ),
                );
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
        Message::OpenMakeShortDialog => {
            open_make_short_dialog(app);
        }
        Message::CloseMakeShortDialog => {
            app.make_short_dialog.open = false;
            app.make_short_dialog.run_state = MakeShortRunState::Idle;
            app.make_short_dialog.validation_error = None;
        }
        Message::MakeShortOutputPathChanged(value) => {
            app.make_short_dialog.output_file_input = value;
            app.make_short_dialog.validation_error = None;
        }
        Message::MakeShortBrowseOutputPath => {
            if let Some(path) = rfd::FileDialog::new()
                .set_title(app.i18n.text(TextKey::MakeShortOutputPathLabel))
                .save_file()
            {
                app.make_short_dialog.output_file_input = path.display().to_string();
                app.make_short_dialog.validation_error = None;
            }
        }
        Message::MakeShortToggleEffect(effect) => {
            toggle_make_short_effect(app, effect);
        }
        Message::MakeShortSetCropPosition(position) => {
            app.make_short_dialog.crop_position = position;
        }
        Message::MakeShortSetAddFade(value) => {
            app.make_short_dialog.add_fade = value;
        }
        Message::MakeShortSpeedChanged(value) => {
            app.make_short_dialog.speed_input = value;
            app.make_short_dialog.validation_error = None;
        }
        Message::MakeShortCrfChanged(value) => {
            app.make_short_dialog.crf_input = value;
            app.make_short_dialog.validation_error = None;
        }
        Message::MakeShortSetPreset(preset) => {
            app.make_short_dialog.preset = preset;
        }
        Message::RunMakeShort => {
            let Some(request) = build_short_request_from_dialog(app) else {
                return Task::none();
            };
            app.make_short_dialog.run_state = MakeShortRunState::Running {
                stage: GenerationStage::Preparing,
                status: app.i18n.text(TextKey::MakeShortStagePreparing).to_owned(),
            };
            return Task::perform(
                async move { do_prepare_short(request) },
                Message::MakeShortPrepared,
            );
        }
        Message::MakeShortPrepared(result) => match result {
            Ok(job) => {
                app.make_short_dialog.run_state = MakeShortRunState::Running {
                    stage: GenerationStage::Generating,
                    status: app.i18n.text(TextKey::MakeShortStageGenerating).to_owned(),
                };
                return Task::perform(
                    async move { do_generate_short(job) },
                    Message::MakeShortGenerated,
                );
            }
            Err(error) => {
                app.make_short_dialog.run_state = MakeShortRunState::Failed {
                    summary: app.i18n.text(TextKey::MakeShortFailureLabel).to_owned(),
                    details: error,
                };
            }
        },
        Message::MakeShortGenerated(result) => match result {
            Ok(output) => {
                app.make_short_dialog.run_state = MakeShortRunState::Running {
                    stage: GenerationStage::Finalizing,
                    status: app.i18n.text(TextKey::MakeShortStageFinalizing).to_owned(),
                };
                app.make_short_dialog.run_state = MakeShortRunState::Success {
                    output_file: output.output_file,
                };
            }
            Err(error) => {
                app.make_short_dialog.run_state = MakeShortRunState::Failed {
                    summary: app.i18n.text(TextKey::MakeShortFailureLabel).to_owned(),
                    details: error,
                };
            }
        },
        Message::OpenGeneratedShortFile => {
            if let MakeShortRunState::Success { output_file } = &app.make_short_dialog.run_state {
                let _ = open_with_system_default(output_file);
            }
        }
        Message::OpenGeneratedShortFolder => {
            if let MakeShortRunState::Success { output_file } = &app.make_short_dialog.run_state {
                let target = output_file
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| output_file.clone());
                let _ = open_with_system_default(&target);
            }
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

            app.background.startup_ready = false;
            app.background.startup_deferred_gallery_refresh = false;
            app.background.startup_deferred_timeline_refresh = false;
            app.background.startup_gallery_continuation_due_at = None;
            app.background.startup_reconcile_queued = false;
            tasks.push(start_snapshot_hydrate(app));

            if !matches!(app.update_check_state, UpdateCheckState::Checking) {
                tasks.push(start_update_check(app, UpdateCheckTrigger::Startup));
            }

            if tasks.is_empty() {
                return Task::none();
            }
            return Task::batch(tasks);
        }
        Message::StartupReconcileKickoff => {
            let Some(due_at) = app.background.startup_reconcile_due_at else {
                return Task::none();
            };
            if Instant::now() < due_at {
                return Task::none();
            }
            app.background.startup_reconcile_due_at = None;
            if app.state.library_roots.is_empty() {
                mark_startup_ready(app);
                return Task::none();
            }
            return request_reconcile(app, BackgroundWorkReason::UserOrSystem);
        }
        Message::StartupGalleryContinuationKickoff => {
            return start_startup_gallery_continuation(app);
        }
        Message::AutomationTick => {
            if let Some(message) = maybe_execute_automation_step(app) {
                return Task::done(message);
            }
            return Task::none();
        }
        Message::NewMediaPreviewTick => {
            if app
                .new_media_announcement
                .as_ref()
                .is_some_and(|announcement| announcement.preview_path.is_none())
            {
                app.new_media_preview_loading_phase = (app.new_media_preview_loading_phase + 1) % 6;
            } else {
                app.new_media_preview_loading_phase = 0;
            }
            return Task::none();
        }
        Message::FilesystemChanged => {
            return request_reconcile(app, BackgroundWorkReason::FilesystemWatch);
        }
        Message::SetFilterMediaKind(kind) => {
            let started_at = Instant::now();
            let previous = filter_state_summary(app);
            app.filter_media_kind = kind;
            app.filter_extension = None;
            startup_log::log_info(
                "interaction.filter_change.request.start",
                &format!(
                    "trigger=media_kind previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            log_interaction_duration(
                "interaction.filter_change.request.end",
                started_at.elapsed(),
                &format!(
                    "trigger=media_kind previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "filter_change",
            );
        }
        Message::SetFilterExtension(ext) => {
            let started_at = Instant::now();
            let previous = filter_state_summary(app);
            app.filter_extension = ext;
            startup_log::log_info(
                "interaction.filter_change.request.start",
                &format!(
                    "trigger=extension previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            log_interaction_duration(
                "interaction.filter_change.request.end",
                started_at.elapsed(),
                &format!(
                    "trigger=extension previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "filter_change",
            );
        }
        Message::SetFilterTag(tag) => {
            let started_at = Instant::now();
            let previous = filter_state_summary(app);
            app.filter_tag = tag;
            startup_log::log_info(
                "interaction.filter_change.request.start",
                &format!(
                    "trigger=tag previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            log_interaction_duration(
                "interaction.filter_change.request.end",
                started_at.elapsed(),
                &format!(
                    "trigger=tag previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "filter_change",
            );
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
        Message::ShortsOutputDirInputChanged(value) => {
            app.shorts_output_dir_input = value;
        }
        Message::ShortsOutputDirBrowse => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.shorts_output_dir_input = path.display().to_string();
            }
        }
        Message::SaveShortsOutputDirSetting => {
            save_shorts_output_dir_setting(app);
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
        Message::MediaViewportChanged {
            absolute_y,
            max_y,
            viewport_height,
        } => {
            handle_media_viewport_changed(app, absolute_y, max_y, viewport_height);
        }
        Message::ViewportSettleTick => {
            settle_media_viewport_drag(app);
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
            return apply_snapshot_hydrate_result(app, *result);
        }
        Message::SnapshotApplyTick => {
            return apply_snapshot_chunk(app);
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
        Message::DeferredThumbnailCatchupKickoff => {
            return start_deferred_thumbnail_catchup(app);
        }
        Message::OpenMediaById(media_id) => {
            open_media_by_id(app, media_id, false);
            app.new_media_announcement = None;
            app.new_media_preview_loading_phase = 0;
        }
        Message::CopyMediaFileById(media_id) => {
            copy_media_file_by_id(app, media_id);
            app.new_media_announcement = None;
            app.new_media_preview_loading_phase = 0;
        }
        Message::DismissNewMediaAnnouncement => {
            app.new_media_announcement = None;
            app.new_media_preview_loading_phase = 0;
            refresh_diagnostics(app);
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
            app.shorts_output_dir_input = app
                .runtime
                .default_shorts_output_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
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
            let started_at = Instant::now();
            let previous = filter_state_summary(app);
            app.filter_source_root_id = root_id;
            startup_log::log_info(
                "interaction.filter_change.request.start",
                &format!(
                    "trigger=library previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            log_interaction_duration(
                "interaction.filter_change.request.end",
                started_at.elapsed(),
                &format!(
                    "trigger=library previous=\"{previous}\" next=\"{}\"",
                    filter_state_summary(app),
                ),
            );
            return request_projection_refresh_with_context(
                app,
                BackgroundWorkReason::UserOrSystem,
                "filter_change",
            );
        }
    }

    Task::none()
}

fn projection_loading_label(app: &Librapix) -> &'static str {
    app.i18n.text(projection_loading_key(
        &app.state.search_query,
        app.state.active_route,
    ))
}

fn projection_loading_key(search_query: &str, active_route: Route) -> TextKey {
    if !search_query.trim().is_empty() {
        TextKey::LoadingSearchLabel
    } else if matches!(active_route, Route::Timeline) {
        TextKey::LoadingTimelineLabel
    } else {
        TextKey::LoadingGalleryLabel
    }
}

fn activity_stage_key(search_query: &str, active_route: Route) -> TextKey {
    if !search_query.trim().is_empty() {
        TextKey::StageRefreshingSearchLabel
    } else if matches!(active_route, Route::Timeline) {
        TextKey::StageRefreshingTimelineLabel
    } else {
        TextKey::StageRefreshingGalleryLabel
    }
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
    if indeterminate {
        app.activity_progress.items_total = None;
    }
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

fn log_storage_open_metrics(context: &str, database_file: &Path, metrics: &StorageOpenMetrics) {
    startup_log::log_duration(
        "storage.open.complete",
        metrics.total_duration,
        &format!(
            "context={context} path={} existed={} connection_open_ms={} pragma_ms={} migration_ms={} migration_from={} migration_to={} applied_migrations={}",
            database_file.display(),
            metrics.file_exists,
            metrics.connection_open_duration.as_millis(),
            metrics.pragma_duration.as_millis(),
            metrics.migration.total_duration.as_millis(),
            metrics.migration.previous_version,
            metrics.migration.final_version,
            metrics.migration.applied.len(),
        ),
    );
    for migration in &metrics.migration.applied {
        startup_log::log_duration(
            "storage.migration.applied",
            migration.duration,
            &format!(
                "context={context} version={} name={}",
                migration.version, migration.name
            ),
        );
    }
}

fn extract_snapshot_version(payload: &str) -> Option<u32> {
    let version_key = "\"version\"";
    let start = payload.find(version_key)?;
    let rest = &payload[start + version_key.len()..];
    let colon_index = rest.find(':')?;
    let digits = rest[colon_index + 1..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn note_first_usable_gallery(app: &mut Librapix, source: &str) {
    if app.startup_metrics.first_usable_gallery_recorded || app.gallery_items.is_empty() {
        return;
    }

    app.startup_metrics.first_usable_gallery_recorded = true;
    let elapsed_ms = startup_log::elapsed_since_launch()
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    startup_log::log_info(
        "startup.first_usable_gallery",
        &format!(
            "source={source} items={} elapsed_ms={elapsed_ms}",
            app.gallery_items.len()
        ),
    );
}

fn mark_startup_ready(app: &mut Librapix) {
    app.background.startup_ready = true;
    set_activity_ready(app);
    schedule_automation_if_needed(app, "startup_ready");
    if !app.startup_metrics.startup_ready_recorded {
        app.startup_metrics.startup_ready_recorded = true;
        let elapsed_ms = startup_log::elapsed_since_launch()
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        startup_log::log_info(
            "startup.ready",
            &format!(
                "elapsed_ms={elapsed_ms} deferred_thumbnail_backlog={} gallery_items={} timeline_items={}",
                app.background.deferred_thumbnail_queue.len(),
                app.gallery_items.len(),
                app.timeline_items.len(),
            ),
        );
    }
}

fn schedule_automation_if_needed(app: &mut Librapix, reason: &'static str) {
    let Some(runner) = app.background.automation.as_mut() else {
        return;
    };
    if runner.due_at.is_some() || runner.steps.is_empty() || !app.background.startup_ready {
        return;
    }

    let wait_ms = runner
        .steps
        .front()
        .map(|step| step.wait_ms)
        .unwrap_or_default();
    runner.due_at = Some(Instant::now() + Duration::from_millis(wait_ms));
    startup_log::log_info(
        "automation.step.scheduled",
        &format!(
            "reason={reason} wait_ms={wait_ms} remaining_steps={}",
            runner.steps.len()
        ),
    );
}

fn maybe_execute_automation_step(app: &mut Librapix) -> Option<Message> {
    let due_at = app.background.automation.as_ref()?.due_at?;
    if Instant::now() < due_at {
        return None;
    }

    if !automation_can_advance(app) {
        let poll_interval = app
            .background
            .automation
            .as_ref()
            .map(|runner| runner.poll_interval)
            .unwrap_or(AUTOMATION_TICK_INTERVAL);
        if let Some(runner) = app.background.automation.as_mut() {
            runner.due_at = Some(Instant::now() + poll_interval);
        }
        startup_log::log_info(
            "automation.step.deferred",
            &format!(
                "route={} startup_ready={} reconcile_in_flight={} projection_in_flight={} thumbnail_in_flight={} pending_reconcile={} pending_projection={}",
                route_name(app.state.active_route),
                app.background.startup_ready,
                app.background.reconcile_in_flight,
                app.background.projection_in_flight,
                app.background.thumbnail_in_flight,
                app.background.pending_reconcile,
                app.background.pending_projection,
            ),
        );
        return None;
    }

    let step = {
        let runner = app.background.automation.as_mut()?;
        let step = runner.steps.pop_front()?;
        runner.due_at = runner
            .steps
            .front()
            .map(|next| Instant::now() + Duration::from_millis(next.wait_ms));
        step
    };

    startup_log::log_info(
        "automation.step.execute",
        &format!(
            "action={} remaining_steps={} route={} {}",
            step.action.label(),
            app.background
                .automation
                .as_ref()
                .map(|runner| runner.steps.len())
                .unwrap_or_default(),
            route_name(app.state.active_route),
            filter_state_summary(app),
        ),
    );

    let message = automation_step_message(app, &step.action);
    if app
        .background
        .automation
        .as_ref()
        .is_some_and(|runner| runner.steps.is_empty() && runner.due_at.is_none())
    {
        startup_log::log_info(
            "automation.script.complete",
            &format!(
                "route={} gallery_items={} timeline_items={} {}",
                route_name(app.state.active_route),
                app.gallery_items.len(),
                app.timeline_items.len(),
                filter_state_summary(app),
            ),
        );
    }
    message
}

fn automation_can_advance(app: &Librapix) -> bool {
    app.background.startup_ready
        && app.background.snapshot_apply.is_none()
        && !app.background.reconcile_in_flight
        && !app.background.projection_in_flight
        && !app.background.pending_reconcile
        && !app.background.pending_projection
}

fn automation_step_message(app: &Librapix, action: &AutomationAction) -> Option<Message> {
    match action {
        AutomationAction::OpenGallery => Some(Message::OpenGallery),
        AutomationAction::OpenTimeline => Some(Message::OpenTimeline),
        AutomationAction::SetFilterMediaKind(kind) => {
            Some(Message::SetFilterMediaKind(kind.clone()))
        }
        AutomationAction::SelectFirstVisible => current_route_items(app)
            .iter()
            .find(|item| !item.is_group_header)
            .map(|item| Message::SelectMedia(item.media_id)),
    }
}

fn current_route_items(app: &Librapix) -> &[BrowseItem] {
    match app.state.active_route {
        Route::Gallery => &app.gallery_items,
        Route::Timeline => &app.timeline_items,
    }
}

fn parse_automation_script(raw: &str) -> Vec<AutomationStep> {
    let mut steps = Vec::new();
    let mut pending_wait_ms = 0u64;

    for token in raw
        .split(';')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        if let Some(value) = token.strip_prefix("wait:") {
            if let Ok(wait_ms) = value.trim().parse::<u64>() {
                pending_wait_ms = pending_wait_ms.saturating_add(wait_ms);
            }
            continue;
        }

        let action = if token.eq_ignore_ascii_case("gallery") {
            Some(AutomationAction::OpenGallery)
        } else if token.eq_ignore_ascii_case("timeline") {
            Some(AutomationAction::OpenTimeline)
        } else if token.eq_ignore_ascii_case("select_first") {
            Some(AutomationAction::SelectFirstVisible)
        } else if let Some(value) = token.strip_prefix("filter_kind:") {
            let value = value.trim();
            Some(AutomationAction::SetFilterMediaKind(
                if value.eq_ignore_ascii_case("none") || value.is_empty() {
                    None
                } else {
                    Some(value.to_owned())
                },
            ))
        } else {
            None
        };

        if let Some(action) = action {
            steps.push(AutomationStep {
                wait_ms: pending_wait_ms,
                action,
            });
            pending_wait_ms = 0;
        }
    }

    steps
}

fn projection_refresh_policy(
    app: &Librapix,
    reason: BackgroundWorkReason,
    trigger: &'static str,
) -> ProjectionRefreshPolicy {
    if !app.background.startup_ready {
        return ProjectionRefreshPolicy::CurrentSurface;
    }
    if matches!(reason, BackgroundWorkReason::FilesystemWatch)
        || matches!(
            trigger,
            "route_switch" | "filter_change" | "search_query" | "startup_continuation"
        )
    {
        ProjectionRefreshPolicy::CurrentSurface
    } else {
        ProjectionRefreshPolicy::Full
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

    let sidebar_content = scrollable(
        column![nav_section, h_divider(), library_section]
            .spacing(SPACE_LG)
            .padding(SPACE_LG as u16),
    )
    .height(Length::Fill);
    let sidebar_status = container(
        column![
            section_heading(app.i18n.text(TextKey::StatusSectionLabel)),
            render_update_chip(app).width(Length::Fill),
            render_activity_status(app),
        ]
        .spacing(SPACE_XS),
    )
    .width(Length::Fill)
    .padding([SPACE_SM as u16, SPACE_MD as u16]);
    let sidebar = container(
        column![sidebar_content, h_divider(), sidebar_status]
            .height(Length::Fill)
            .spacing(SPACE_XS),
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
        .on_scroll(|viewport| Message::MediaViewportChanged {
            absolute_y: viewport.absolute_offset().y,
            max_y: (viewport.content_bounds().height - viewport.bounds().height).max(0.0),
            viewport_height: viewport.bounds().height.max(0.0),
        })
        .height(Length::Fill);
    let media_scrollable: Element<'_, Message> = base_media_scrollable.into();
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
    if app.make_short_dialog.open {
        overlay = stack([overlay, render_make_short_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    overlay
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
                render_justified_gallery(
                    app,
                    &app.search_items,
                    app.state.selected_media_id,
                    app.media_scroll_absolute_y,
                    app.media_viewport_height,
                ),
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
            Route::Gallery => render_justified_gallery(
                app,
                browse_items,
                app.state.selected_media_id,
                app.media_scroll_absolute_y,
                app.media_viewport_height,
            ),
            Route::Timeline => render_timeline_view(
                app,
                browse_items,
                app.state.selected_media_id,
                app.media_scroll_absolute_y,
                app.media_viewport_height,
            ),
        }
    };

    let scrollable_content: Element<'_, Message> = column![search_section, browse_content]
        .spacing(SPACE_LG)
        .into();

    (header, scrollable_content)
}

#[derive(Debug, Clone, Copy)]
struct JustifiedRowLayout {
    start: usize,
    end: usize,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct VisibleRowWindow {
    start_row: usize,
    end_row: usize,
    visible_rows: usize,
    total_rows: usize,
    top_spacer: f32,
    bottom_spacer: f32,
}

#[derive(Debug, Clone, Copy)]
struct TimelineRenderWindowMetrics {
    total_items: usize,
    total_groups: usize,
    total_rows: usize,
    visible_groups: usize,
    visible_rows: usize,
    first_visible_row: Option<usize>,
    last_visible_row: Option<usize>,
    top_spacer: f32,
    bottom_spacer: f32,
    viewport_absolute_y: f32,
    viewport_height: f32,
    overscan_px: f32,
    drag_active: bool,
}

#[derive(Debug, Clone, Copy)]
struct SurfaceRenderMetrics<'a> {
    surface: &'a str,
    total_items: usize,
    total_rows: usize,
    visible_rows: usize,
    viewport_absolute_y: f32,
    viewport_height: f32,
    overscan_px: f32,
    drag_active: bool,
    render_elapsed: Duration,
}

fn build_justified_row_layouts(
    items: &[&BrowseItem],
    available_width: f32,
) -> Vec<JustifiedRowLayout> {
    let mut layouts = Vec::new();
    let gap = GALLERY_GAP as f32;
    let mut row_start = 0;

    while row_start < items.len() {
        let mut ar_sum = 0.0f32;
        let mut row_end = row_start;

        while row_end < items.len() {
            ar_sum += items[row_end].aspect_ratio;
            row_end += 1;
            let n_gaps = (row_end - row_start).saturating_sub(1) as f32;
            let row_h = (available_width - gap * n_gaps) / ar_sum;
            if row_h <= TARGET_ROW_HEIGHT {
                break;
            }
        }

        let n_gaps = (row_end - row_start).saturating_sub(1) as f32;
        let row_height = ((available_width - gap * n_gaps) / ar_sum).clamp(100.0, MAX_ROW_HEIGHT);
        layouts.push(JustifiedRowLayout {
            start: row_start,
            end: row_end,
            height: row_height,
        });
        row_start = row_end;
    }

    layouts
}

fn compute_visible_row_window(
    layouts: &[JustifiedRowLayout],
    rows_top: f32,
    viewport_top: f32,
    viewport_bottom: f32,
    gap: f32,
) -> VisibleRowWindow {
    let mut cursor_y = rows_top;
    let mut first_visible = None;
    let mut last_visible_exclusive = 0usize;
    let mut top_spacer = 0.0f32;
    let mut trailing_spacer = 0.0f32;

    for (index, layout) in layouts.iter().enumerate() {
        let row_top = cursor_y;
        let row_bottom = row_top + layout.height;
        let gap_after = if index + 1 < layouts.len() { gap } else { 0.0 };
        let segment_height = layout.height + gap_after;
        let intersects_window = row_bottom >= viewport_top && row_top <= viewport_bottom;

        if intersects_window {
            if first_visible.is_none() {
                first_visible = Some(index);
            }
            last_visible_exclusive = index + 1;
        } else if first_visible.is_none() {
            top_spacer += segment_height;
        } else {
            trailing_spacer += segment_height;
        }

        cursor_y += segment_height;
    }

    let start_row = first_visible.unwrap_or(0);
    let visible_rows = last_visible_exclusive.saturating_sub(start_row);
    let bottom_spacer = if visible_rows == 0 {
        0.0
    } else {
        trailing_spacer
    };

    VisibleRowWindow {
        start_row,
        end_row: last_visible_exclusive,
        visible_rows,
        total_rows: layouts.len(),
        top_spacer,
        bottom_spacer,
    }
}

fn log_large_surface_render_once(metrics: SurfaceRenderMetrics<'_>) {
    if metrics.total_items < 1_000 {
        return;
    }

    if metrics.drag_active {
        return;
    }

    let signature = format!(
        "{surface}:{total_items}:{total_rows}:{visible_rows}:{:.0}:{:.0}:{:.0}",
        (metrics.viewport_absolute_y / 250.0).floor(),
        metrics.viewport_height,
        metrics.overscan_px,
        surface = metrics.surface,
        total_items = metrics.total_items,
        total_rows = metrics.total_rows,
        visible_rows = metrics.visible_rows,
    );
    let seen = LARGE_SURFACE_RENDER_LOGS.get_or_init(|| Mutex::new(HashSet::new()));
    let mut seen = seen.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if !seen.insert(signature) {
        return;
    }

    startup_log::log_info(
        "interaction.surface_render.window",
        &format!(
            "surface={surface} total_items={total_items} total_rows={total_rows} visible_rows={visible_rows} overscan_px={overscan_px:.1} drag_active={drag_active} viewport_y={viewport_absolute_y:.1} viewport_height={viewport_height:.1} render_elapsed_ms={}",
            metrics.render_elapsed.as_millis(),
            surface = metrics.surface,
            total_items = metrics.total_items,
            total_rows = metrics.total_rows,
            visible_rows = metrics.visible_rows,
            overscan_px = metrics.overscan_px,
            drag_active = metrics.drag_active,
            viewport_absolute_y = metrics.viewport_absolute_y,
            viewport_height = metrics.viewport_height,
        ),
    );

    if metrics.render_elapsed >= Duration::from_millis(16) {
        startup_log::log_warn(
            "interaction.surface_render.slow",
            &format!(
                "surface={surface} total_items={total_items} total_rows={total_rows} visible_rows={visible_rows} overscan_px={overscan_px:.1} drag_active={drag_active} viewport_y={viewport_absolute_y:.1} viewport_height={viewport_height:.1} render_elapsed_ms={}",
                metrics.render_elapsed.as_millis(),
                surface = metrics.surface,
                total_items = metrics.total_items,
                total_rows = metrics.total_rows,
                visible_rows = metrics.visible_rows,
                overscan_px = metrics.overscan_px,
                drag_active = metrics.drag_active,
                viewport_absolute_y = metrics.viewport_absolute_y,
                viewport_height = metrics.viewport_height,
            ),
        );
    }
}

fn log_timeline_window_once(metrics: TimelineRenderWindowMetrics) {
    if metrics.drag_active {
        return;
    }

    let signature = format!(
        "timeline:{total_items}:{total_groups}:{total_rows}:{visible_groups}:{visible_rows}:{}:{}:{:.0}:{:.0}:{:.0}:{:.0}:{:.0}",
        metrics
            .first_visible_row
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        metrics
            .last_visible_row
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_owned()),
        metrics.top_spacer,
        metrics.bottom_spacer,
        (metrics.viewport_absolute_y / 250.0).floor(),
        metrics.viewport_height,
        metrics.overscan_px,
        total_items = metrics.total_items,
        total_groups = metrics.total_groups,
        total_rows = metrics.total_rows,
        visible_groups = metrics.visible_groups,
        visible_rows = metrics.visible_rows,
    );
    let seen = LARGE_SURFACE_RENDER_LOGS.get_or_init(|| Mutex::new(HashSet::new()));
    let mut seen = seen.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if !seen.insert(signature) {
        return;
    }

    startup_log::log_info(
        "interaction.timeline_render.window",
        &format!(
            "surface=timeline total_items={total_items} total_groups={total_groups} total_rows={total_rows} visible_groups={visible_groups} visible_rows={visible_rows} first_visible_row={} last_visible_row={} overscan_px={overscan_px:.1} drag_active={drag_active} top_spacer={top_spacer:.1} bottom_spacer={bottom_spacer:.1} viewport_y={viewport_absolute_y:.1} viewport_height={viewport_height:.1}",
            metrics
                .first_visible_row
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            metrics
                .last_visible_row
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            total_items = metrics.total_items,
            total_groups = metrics.total_groups,
            total_rows = metrics.total_rows,
            visible_groups = metrics.visible_groups,
            visible_rows = metrics.visible_rows,
            overscan_px = metrics.overscan_px,
            drag_active = metrics.drag_active,
            top_spacer = metrics.top_spacer,
            bottom_spacer = metrics.bottom_spacer,
            viewport_absolute_y = metrics.viewport_absolute_y,
            viewport_height = metrics.viewport_height,
        ),
    );

    if metrics.visible_rows > 200 {
        startup_log::log_warn(
            "interaction.timeline_render.window.anomaly",
            &format!(
                "visible_rows={visible_rows} total_rows={total_rows} total_groups={total_groups} first_visible_row={} last_visible_row={} overscan_px={overscan_px:.1} drag_active={drag_active} top_spacer={top_spacer:.1} bottom_spacer={bottom_spacer:.1} viewport_y={viewport_absolute_y:.1} viewport_height={viewport_height:.1}",
                metrics
                    .first_visible_row
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                metrics
                    .last_visible_row
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_owned()),
                visible_rows = metrics.visible_rows,
                total_rows = metrics.total_rows,
                total_groups = metrics.total_groups,
                overscan_px = metrics.overscan_px,
                drag_active = metrics.drag_active,
                top_spacer = metrics.top_spacer,
                bottom_spacer = metrics.bottom_spacer,
                viewport_absolute_y = metrics.viewport_absolute_y,
                viewport_height = metrics.viewport_height,
            ),
        );
    }
}

fn render_justified_gallery<'a>(
    app: &'a Librapix,
    items: &'a [BrowseItem],
    selected_id: Option<i64>,
    viewport_absolute_y: f32,
    viewport_height: f32,
) -> Element<'a, Message> {
    let total_items = items.len();
    if total_items == 0 {
        return column![].into();
    }

    responsive(move |size: Size| {
        let render_started_at = Instant::now();
        let layout = gallery_layout_for_width(app, items, size.width);
        let overscan_px = media_virtual_overscan_px(app.viewport_drag.active);
        let viewport_height = if viewport_height > 0.0 {
            viewport_height
        } else {
            size.height.clamp(600.0, 900.0)
        };
        let window_top = (viewport_absolute_y - overscan_px).max(0.0);
        let window_bottom = viewport_absolute_y + viewport_height + overscan_px;
        let mut grid = column![];
        let mut pending_spacer = 0.0f32;
        let mut cursor_y = 0.0f32;
        let mut visible_rows = 0usize;

        for (index, row_layout) in layout.rows.iter().enumerate() {
            let row_top = cursor_y;
            let row_bottom = row_top + row_layout.height;
            let gap_after = if index + 1 < layout.rows.len() {
                GALLERY_GAP as f32
            } else {
                0.0
            };
            let segment_height = row_layout.height + gap_after;
            let intersects_window = row_bottom >= window_top && row_top <= window_bottom;

            if !intersects_window {
                pending_spacer += segment_height;
                cursor_y += segment_height;
                continue;
            }

            if pending_spacer > 0.0 {
                grid = grid.push(Space::new().height(Length::Fixed(pending_spacer)));
                pending_spacer = 0.0;
            }

            let mut row_widget = row![].spacing(GALLERY_GAP);
            for item in &items[row_layout.start..row_layout.end] {
                let portion = (item.aspect_ratio * 1000.0).max(1.0) as u16;
                let card =
                    render_media_card(item, selected_id == Some(item.media_id), row_layout.height);
                row_widget = row_widget.push(container(card).width(Length::FillPortion(portion)));
            }
            visible_rows = visible_rows.saturating_add(1);
            grid = grid.push(row_widget);
            if gap_after > 0.0 {
                grid = grid.push(Space::new().height(Length::Fixed(gap_after)));
            }
            cursor_y += segment_height;
        }

        if pending_spacer > 0.0 {
            grid = grid.push(Space::new().height(Length::Fixed(pending_spacer)));
        }

        log_large_surface_render_once(SurfaceRenderMetrics {
            surface: "gallery",
            total_items,
            total_rows: layout.rows.len(),
            visible_rows,
            viewport_absolute_y,
            viewport_height,
            overscan_px,
            drag_active: app.viewport_drag.active,
            render_elapsed: render_started_at.elapsed(),
        });

        grid.into()
    })
    .into()
}

fn render_media_card(item: &BrowseItem, selected: bool, height: f32) -> Element<'_, Message> {
    let thumb: Element<'_, Message> = if let Some(path) = &item.thumbnail_path {
        image(image::Handle::from_path(path))
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .content_fit(ContentFit::Cover)
            .into()
    } else {
        container(
            column![
                Space::new().height(Length::Fill),
                text(item.title.clone())
                    .size(FONT_CAPTION)
                    .color(TEXT_TERTIARY),
            ]
            .padding(SPACE_XS as u16),
        )
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(thumb_placeholder_style)
        .into()
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

fn invalidate_browse_layout_cache(app: &mut Librapix, reason: &str) {
    app.browse_layout_generation = app.browse_layout_generation.saturating_add(1);
    *app.layout_cache.borrow_mut() = MediaLayoutCache::default();
    *app.drag_layout_preview.borrow_mut() = DragLayoutPreviewState::default();
    startup_log::log_info(
        "interaction.surface_layout.invalidate",
        &format!(
            "reason={reason} generation={} gallery_items={} timeline_items={} search_items={}",
            app.browse_layout_generation,
            app.gallery_items.len(),
            app.timeline_items.len(),
            app.search_items.len(),
        ),
    );
}

fn layout_width_key(available_width: f32) -> u32 {
    available_width.max(0.0).round() as u32
}

fn media_virtual_overscan_px(active_drag: bool) -> f32 {
    if active_drag {
        MEDIA_VIRTUAL_OVERSCAN_DRAG_PX
    } else {
        MEDIA_VIRTUAL_OVERSCAN_PX
    }
}

fn cached_layout_width_key(app: &Librapix, surface: MediaSurfaceKind) -> Option<u32> {
    let cache = app.layout_cache.borrow();
    match surface {
        MediaSurfaceKind::Gallery => cache.gallery.as_ref().map(|layout| layout.width_key),
        MediaSurfaceKind::Timeline => cache.timeline.as_ref().map(|layout| layout.width_key),
    }
}

fn layout_width_for_surface(app: &Librapix, surface: MediaSurfaceKind, measured_width: f32) -> f32 {
    let measured_width_key = layout_width_key(measured_width);
    if !app.viewport_drag.active {
        return measured_width_key as f32;
    }

    let mut preview = app.drag_layout_preview.borrow_mut();
    let state = preview.surface_mut(surface);
    if state.frozen_width_key.is_none() {
        let cached_width_key = cached_layout_width_key(app, surface);
        let frozen_width_key = cached_width_key.unwrap_or(measured_width_key);
        state.frozen_width_key = Some(frozen_width_key);
        state.last_measured_width_key = Some(measured_width_key);
        startup_log::log_info(
            "interaction.surface_layout.drag_width.freeze",
            &format!(
                "surface={} frozen_width={} measured_width={} source={}",
                surface.label(),
                frozen_width_key,
                measured_width_key,
                if cached_width_key.is_some() {
                    "last_settled_layout"
                } else {
                    "current_measurement"
                },
            ),
        );
        return frozen_width_key as f32;
    }

    let frozen_width_key = state.frozen_width_key.unwrap_or(measured_width_key);
    if state.last_measured_width_key != Some(measured_width_key) {
        state.width_change_count = state.width_change_count.saturating_add(1);
        if measured_width_key != frozen_width_key {
            state.suppressed_rebuilds = state.suppressed_rebuilds.saturating_add(1);
            if state.suppressed_rebuilds >= 8 && !state.anomaly_logged {
                startup_log::log_warn(
                    "interaction.surface_layout.drag_width.anomaly",
                    &format!(
                        "surface={} frozen_width={} measured_width={} width_changes={} suppressed_rebuilds={}",
                        surface.label(),
                        frozen_width_key,
                        measured_width_key,
                        state.width_change_count,
                        state.suppressed_rebuilds,
                    ),
                );
                state.anomaly_logged = true;
            }
        }
        state.last_measured_width_key = Some(measured_width_key);
    }

    frozen_width_key as f32
}

fn gallery_layout_for_width(
    app: &Librapix,
    items: &[BrowseItem],
    measured_width: f32,
) -> Arc<CachedGalleryLayout> {
    let available_width = layout_width_for_surface(app, MediaSurfaceKind::Gallery, measured_width);
    let width_key = layout_width_key(available_width);
    let first_media_id = items.first().map(|item| item.media_id);
    let last_media_id = items.last().map(|item| item.media_id);
    if let Some(layout) = app.layout_cache.borrow().gallery.as_ref()
        && layout.generation == app.browse_layout_generation
        && layout.width_key == width_key
        && layout.item_count == items.len()
        && layout.first_media_id == first_media_id
        && layout.last_media_id == last_media_id
    {
        return Arc::clone(layout);
    }

    let started_at = Instant::now();
    let media = items
        .iter()
        .filter(|item| !item.is_group_header)
        .collect::<Vec<_>>();
    let rows = build_justified_row_layouts(&media, available_width);
    let layout = Arc::new(CachedGalleryLayout {
        generation: app.browse_layout_generation,
        width_key,
        item_count: items.len(),
        first_media_id,
        last_media_id,
        rows: Arc::<[JustifiedRowLayout]>::from(rows),
    });
    app.layout_cache.borrow_mut().gallery = Some(Arc::clone(&layout));
    log_interaction_duration(
        "interaction.surface_layout.cache_build",
        started_at.elapsed(),
        &format!(
            "surface=gallery width={} measured_width={} drag_active={} items={} rows={}",
            width_key,
            layout_width_key(measured_width),
            app.viewport_drag.active,
            media.len(),
            layout.rows.len(),
        ),
    );
    layout
}

fn timeline_layout_for_width(
    app: &Librapix,
    items: &[BrowseItem],
    measured_width: f32,
) -> Arc<CachedTimelineLayout> {
    let available_width = layout_width_for_surface(app, MediaSurfaceKind::Timeline, measured_width);
    let width_key = layout_width_key(available_width);
    let first_media_id = items.first().map(|item| item.media_id);
    let last_media_id = items.last().map(|item| item.media_id);
    if let Some(layout) = app.layout_cache.borrow().timeline.as_ref()
        && layout.generation == app.browse_layout_generation
        && layout.width_key == width_key
        && layout.item_count == items.len()
        && layout.first_media_id == first_media_id
        && layout.last_media_id == last_media_id
    {
        return Arc::clone(layout);
    }

    let started_at = Instant::now();
    let mut sections = Vec::new();
    let mut i = 0usize;
    let mut total_items = 0usize;
    let mut total_rows = 0usize;
    let mut total_groups = 0usize;

    while i < items.len() {
        if !items[i].is_group_header {
            i += 1;
            continue;
        }

        total_groups = total_groups.saturating_add(1);
        let header_index = i;
        i += 1;
        let media_start = i;
        while i < items.len() && !items[i].is_group_header {
            i += 1;
        }

        total_items = total_items.saturating_add(i.saturating_sub(media_start));
        let group_media = items[media_start..i].iter().collect::<Vec<_>>();
        let row_layouts = build_justified_row_layouts(&group_media, available_width);
        let rows_height = row_layouts
            .iter()
            .enumerate()
            .map(|(index, layout)| {
                layout.height
                    + if index + 1 < row_layouts.len() {
                        GALLERY_GAP as f32
                    } else {
                        0.0
                    }
            })
            .sum::<f32>();
        let body_spacing = if row_layouts.is_empty() {
            0.0
        } else {
            SPACE_XS as f32
        };
        total_rows = total_rows.saturating_add(row_layouts.len());
        sections.push(CachedTimelineSectionLayout {
            header_index,
            media_start,
            row_layouts: Arc::<[JustifiedRowLayout]>::from(row_layouts),
            section_height: TIMELINE_GROUP_HEADER_HEIGHT + body_spacing + rows_height,
        });
    }

    let layout = Arc::new(CachedTimelineLayout {
        generation: app.browse_layout_generation,
        width_key,
        item_count: items.len(),
        first_media_id,
        last_media_id,
        total_items,
        total_groups,
        total_rows,
        sections: Arc::<[CachedTimelineSectionLayout]>::from(sections),
    });
    app.layout_cache.borrow_mut().timeline = Some(Arc::clone(&layout));
    log_interaction_duration(
        "interaction.surface_layout.cache_build",
        started_at.elapsed(),
        &format!(
            "surface=timeline width={} measured_width={} drag_active={} items={} groups={} rows={}",
            width_key,
            layout_width_key(measured_width),
            app.viewport_drag.active,
            layout.total_items,
            layout.total_groups,
            layout.total_rows,
        ),
    );
    layout
}

fn render_timeline_view<'a>(
    app: &'a Librapix,
    items: &'a [BrowseItem],
    selected_id: Option<i64>,
    viewport_absolute_y: f32,
    viewport_height: f32,
) -> Element<'a, Message> {
    responsive(move |size: Size| {
        let render_started_at = Instant::now();
        let layout = timeline_layout_for_width(app, items, size.width);
        let overscan_px = media_virtual_overscan_px(app.viewport_drag.active);
        let viewport_height = if viewport_height > 0.0 {
            viewport_height
        } else {
            size.height.clamp(600.0, 900.0)
        };
        let window_top = (viewport_absolute_y - overscan_px).max(0.0);
        let window_bottom = viewport_absolute_y + viewport_height + overscan_px;
        let mut sections = column![];
        let mut pending_spacer = 0.0f32;
        let mut cursor_y = 0.0f32;
        let mut visible_groups = 0usize;
        let mut visible_rows = 0usize;
        let mut global_row_index = 0usize;
        let mut first_visible_row = None;
        let mut last_visible_row = None;
        let mut top_spacer_height = 0.0f32;
        let mut bottom_spacer_height = 0.0f32;

        for (section_index, section_layout) in layout.sections.iter().enumerate() {
            let header_item = &items[section_layout.header_index];
            let row_layouts = &section_layout.row_layouts;
            let section_gap = if section_index + 1 < layout.sections.len() {
                SPACE_MD as f32
            } else {
                0.0
            };
            let section_height = section_layout.section_height;
            let section_top = cursor_y;
            let section_bottom = section_top + section_height;
            let intersects_window = section_bottom >= window_top && section_top <= window_bottom;

            if !intersects_window {
                pending_spacer += section_height + section_gap;
                if visible_rows == 0 {
                    top_spacer_height += section_height + section_gap;
                } else {
                    bottom_spacer_height += section_height + section_gap;
                }
                global_row_index += row_layouts.len();
                cursor_y += section_height + section_gap;
                continue;
            }

            if pending_spacer > 0.0 {
                sections = sections.push(Space::new().height(Length::Fixed(pending_spacer)));
                pending_spacer = 0.0;
            }

            visible_groups = visible_groups.saturating_add(1);
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
            let body_spacing = if row_layouts.is_empty() {
                0.0
            } else {
                SPACE_XS as f32
            };
            let rows_top = section_top + TIMELINE_GROUP_HEADER_HEIGHT + body_spacing;
            let row_window = compute_visible_row_window(
                row_layouts,
                rows_top,
                window_top,
                window_bottom,
                GALLERY_GAP as f32,
            );
            let header_bottom = section_top + TIMELINE_GROUP_HEADER_HEIGHT;
            let header_visible = header_bottom >= window_top && section_top <= window_bottom;
            let mut section_pending_spacer = 0.0f32;

            if header_visible {
                sections = sections.push(group_header);
            } else {
                section_pending_spacer += TIMELINE_GROUP_HEADER_HEIGHT;
            }

            section_pending_spacer += body_spacing + row_window.top_spacer;

            if row_window.visible_rows > 0 {
                if first_visible_row.is_none() {
                    first_visible_row = Some(global_row_index + row_window.start_row);
                }
                last_visible_row = Some(global_row_index + row_window.end_row - 1);
            }

            if section_pending_spacer > 0.0 {
                sections =
                    sections.push(Space::new().height(Length::Fixed(section_pending_spacer)));
            }

            for (index, layout) in row_layouts
                .iter()
                .enumerate()
                .skip(row_window.start_row)
                .take(row_window.visible_rows)
            {
                let mut row_widget = row![].spacing(GALLERY_GAP);
                for item in &items[section_layout.media_start + layout.start
                    ..section_layout.media_start + layout.end]
                {
                    let portion = (item.aspect_ratio * 1000.0).max(1.0) as u16;
                    let card =
                        render_media_card(item, selected_id == Some(item.media_id), layout.height);
                    row_widget =
                        row_widget.push(container(card).width(Length::FillPortion(portion)));
                }
                visible_rows = visible_rows.saturating_add(1);
                sections = sections.push(row_widget);
                if index + 1 < row_window.end_row {
                    sections =
                        sections.push(Space::new().height(Length::Fixed(GALLERY_GAP as f32)));
                }
            }

            let trailing_spacer = row_window.bottom_spacer + section_gap;
            if trailing_spacer > 0.0 {
                sections = sections.push(Space::new().height(Length::Fixed(trailing_spacer)));
            }
            if visible_rows == 0 {
                top_spacer_height += section_pending_spacer + trailing_spacer;
            } else {
                bottom_spacer_height += trailing_spacer;
            }
            global_row_index += row_layouts.len();
            cursor_y += section_height + section_gap;
        }

        if pending_spacer > 0.0 {
            sections = sections.push(Space::new().height(Length::Fixed(pending_spacer)));
        }

        log_large_surface_render_once(SurfaceRenderMetrics {
            surface: "timeline",
            total_items: layout.total_items,
            total_rows: layout.total_rows,
            visible_rows,
            viewport_absolute_y,
            viewport_height,
            overscan_px,
            drag_active: app.viewport_drag.active,
            render_elapsed: render_started_at.elapsed(),
        });
        log_timeline_window_once(TimelineRenderWindowMetrics {
            total_items: layout.total_items,
            total_groups: layout.total_groups,
            total_rows: layout.total_rows,
            visible_groups,
            visible_rows,
            first_visible_row,
            last_visible_row,
            top_spacer: top_spacer_height,
            bottom_spacer: bottom_spacer_height,
            viewport_absolute_y,
            viewport_height,
            overscan_px,
            drag_active: app.viewport_drag.active,
        });

        sections.into()
    })
    .into()
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

fn render_activity_status(app: &Librapix) -> Element<'_, Message> {
    let progress = &app.activity_progress;
    let indicator_mode = activity_indicator_mode(progress);
    let progress_line = match indicator_mode {
        ActivityIndicatorMode::Determinate { total, done } => format!(
            "{} {} / {}",
            app.i18n.text(TextKey::ProgressItemsLabel),
            done,
            total
        ),
        ActivityIndicatorMode::Indeterminate => {
            format!("{} --", app.i18n.text(TextKey::ProgressItemsLabel))
        }
        ActivityIndicatorMode::Idle => {
            format!("{} 0 / 0", app.i18n.text(TextKey::ProgressItemsLabel))
        }
    };
    let queue_line = format!(
        "{} {}",
        app.i18n.text(TextKey::ProgressQueueLabel),
        progress.queue_depth
    );
    let roots_line = progress.roots_total.map(|total| {
        format!(
            "{} {} / {}",
            app.i18n.text(TextKey::ProgressRootsLabel),
            progress.roots_done.min(total),
            total
        )
    });

    if matches!(indicator_mode, ActivityIndicatorMode::Idle) {
        return container(
            row![
                text("\u{25CF}").size(FONT_CAPTION).color(SUCCESS_COLOR),
                text(app.activity_status.clone())
                    .size(FONT_CAPTION)
                    .color(TEXT_SECONDARY),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
        )
        .width(Length::Fill)
        .into();
    }

    let indicator: Element<'_, Message> = match indicator_mode {
        ActivityIndicatorMode::Determinate { total, done } => {
            let capped_total = total.max(1) as f32;
            let done = done.min(total) as f32;
            container(progress_bar(0.0..=capped_total, done))
                .height(Length::Fixed(4.0))
                .into()
        }
        ActivityIndicatorMode::Indeterminate => row![
            text("\u{25CF}").size(FONT_CAPTION).color(ACCENT),
            text(app.i18n.text(TextKey::ActivityWorkingLabel))
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
        ]
        .spacing(SPACE_XS)
        .align_y(iced::Alignment::Center)
        .into(),
        ActivityIndicatorMode::Idle => Space::new().into(),
    };

    let mut metrics = vec![progress_line, queue_line];
    if let Some(roots_line) = roots_line {
        metrics.push(roots_line);
    }
    let metrics_line = metrics.join(" | ");

    let mut lines = column![
        text(progress.stage_text.clone())
            .size(FONT_BODY)
            .color(TEXT_PRIMARY),
        indicator,
        text(metrics_line).size(FONT_CAPTION).color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_2XS);
    if !progress.detail_text.trim().is_empty() {
        lines = lines.push(
            text(progress.detail_text.clone())
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        );
    }
    if let Some(error) = progress.last_error.as_ref() {
        let error_line = format!("{}: {}", app.i18n.text(TextKey::ProgressErrorLabel), error);
        let (error_compact, error_truncated) = truncate_for_sidebar(&error_line, 120);
        let error_text = text(error_compact)
            .size(FONT_CAPTION)
            .color(WARNING_COLOR)
            .width(Length::Fill);
        let error_widget: Element<'_, Message> = if error_truncated {
            tooltip(
                error_text,
                container(text(error_line).size(FONT_CAPTION).color(TEXT_PRIMARY))
                    .padding([SPACE_XS as u16, SPACE_SM as u16])
                    .style(card_style),
                tooltip::Position::Top,
            )
            .into()
        } else {
            error_text.into()
        };
        lines = lines.push(error_widget);
    }

    container(lines).width(Length::Fill).into()
}

fn render_update_chip(app: &Librapix) -> iced::widget::Button<'_, Message> {
    let mut update_chip = button(
        text(app.i18n.text(update_chip_text_key(&app.update_check_state))).size(FONT_CAPTION),
    )
    .width(Length::Fill)
    .padding([SPACE_XS as u16, SPACE_MD as u16]);
    if !matches!(app.update_check_state, UpdateCheckState::Checking) {
        update_chip = update_chip.on_press(Message::UpdateChipPressed);
    }
    match &app.update_check_state {
        UpdateCheckState::UpdateAvailable { .. } => update_chip.style(primary_button_style),
        UpdateCheckState::Checking => update_chip.style(action_button_style),
        UpdateCheckState::UpToDate => update_chip.style(action_button_style),
        UpdateCheckState::Unknown | UpdateCheckState::Failed => {
            update_chip.style(subtle_button_style)
        }
    }
}

fn truncate_for_sidebar(value: &str, max_chars: usize) -> (String, bool) {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        return (format!("{truncated}..."), true);
    }
    (value.to_owned(), false)
}

fn activity_indicator_mode(progress: &ActivityProgressState) -> ActivityIndicatorMode {
    if !progress.busy {
        return ActivityIndicatorMode::Idle;
    }
    if let Some(total) = progress.items_total {
        return ActivityIndicatorMode::Determinate {
            total,
            done: progress.items_done.min(total),
        };
    }
    ActivityIndicatorMode::Indeterminate
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
        h_divider(),
        row![
            text(app.i18n.text(TextKey::DefaultShortsOutputDirLabel))
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
            text_input(
                app.i18n.text(TextKey::DefaultShortsOutputDirPlaceholder),
                &app.shorts_output_dir_input
            )
            .on_input(Message::ShortsOutputDirInputChanged)
            .style(field_input_style)
            .width(Length::Fill),
            button(
                row![
                    image(assets::icon_browse())
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(app.i18n.text(TextKey::BrowseFolderButton)).size(FONT_CAPTION),
                ]
                .spacing(SPACE_XS)
                .align_y(iced::Alignment::Center),
            )
            .on_press(Message::ShortsOutputDirBrowse)
            .style(subtle_button_style)
            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            button(
                row![
                    image(assets::icon_save())
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_CAPTION),
                ]
                .spacing(SPACE_XS)
                .align_y(iced::Alignment::Center),
            )
            .on_press(Message::SaveShortsOutputDirSetting)
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
        button(
            row![
                image(assets::icon_save())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::LibrarySaveAndAddAnotherButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
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
                button(
                    row![
                        image(assets::icon_browse())
                            .width(Length::Fixed(16.0))
                            .height(Length::Fixed(16.0))
                            .content_fit(ContentFit::Contain)
                            .filter_method(FilterMethod::Linear),
                        text(app.i18n.text(TextKey::BrowseFolderButton)).size(FONT_BODY),
                    ]
                    .spacing(SPACE_SM)
                    .align_y(iced::Alignment::Center),
                )
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
            button(
                row![
                    image(assets::icon_save())
                        .width(Length::Fixed(16.0))
                        .height(Length::Fixed(16.0))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_BODY),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center),
            )
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
    const NEW_MEDIA_PREVIEW_HEIGHT: f32 = 220.0;

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

    let preview: Element<'_, Message> = if let Some(path) = &announcement.preview_path {
        container(
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fixed(NEW_MEDIA_PREVIEW_HEIGHT))
                .content_fit(ContentFit::Contain),
        )
        .style(card_style)
        .into()
    } else {
        render_new_media_preview_loading_state(app, NEW_MEDIA_PREVIEW_HEIGHT)
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

fn render_new_media_preview_loading_state(app: &Librapix, height: f32) -> Element<'_, Message> {
    let placeholder = container(
        column![
            container(Space::new())
                .width(Length::Fill)
                .height(Length::FillPortion(5))
                .style(preview_loading_block_style),
            row![
                container(Space::new())
                    .width(Length::FillPortion(3))
                    .height(Length::Fixed(8.0))
                    .style(preview_loading_block_style),
                Space::new().width(Length::Fixed(SPACE_SM as f32)),
                container(Space::new())
                    .width(Length::FillPortion(2))
                    .height(Length::Fixed(8.0))
                    .style(preview_loading_block_style),
            ]
            .height(Length::FillPortion(1)),
        ]
        .spacing(SPACE_SM)
        .height(Length::Fill),
    )
    .padding(SPACE_MD as u16)
    .width(Length::Fill)
    .height(Length::Fixed(height))
    .style(thumb_placeholder_style);

    let loading_indicator = container(
        column![
            row![
                text(
                    app.i18n
                        .text(TextKey::NewFileAnnouncementPreparingPreviewLabel)
                )
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
                text(new_media_preview_loading_indicator(
                    app.new_media_preview_loading_phase,
                ))
                .size(FONT_CAPTION)
                .color(ACCENT),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
            text(new_media_preview_loading_pulse(
                app.new_media_preview_loading_phase,
            ))
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
        ]
        .spacing(SPACE_XS)
        .align_x(iced::Alignment::Center),
    )
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .width(Length::Fill)
    .height(Length::Fixed(height));

    stack([placeholder.into(), loading_indicator.into()])
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}

fn new_media_preview_loading_indicator(phase: usize) -> &'static str {
    match phase % 6 {
        0 => "·  ",
        1 => "·· ",
        2 => "···",
        3 => " ··",
        4 => "  ·",
        _ => " · ",
    }
}

fn new_media_preview_loading_pulse(phase: usize) -> &'static str {
    match phase % 6 {
        0 => "○ ● ○",
        1 => "○ ○ ●",
        2 => "● ○ ○",
        3 => "○ ● ○",
        4 => "○ ○ ●",
        _ => "● ○ ○",
    }
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

fn viewport_snapshot_matches(
    app: &Librapix,
    absolute_y: f32,
    max_y: f32,
    viewport_height: f32,
) -> bool {
    (app.media_scroll_absolute_y - absolute_y).abs() < 0.5
        && (app.media_scroll_max_y - max_y).abs() < 0.5
        && (app.media_viewport_height - viewport_height).abs() < 0.5
}

fn viewport_snapshot_matches_values(left: ViewportSnapshot, right: ViewportSnapshot) -> bool {
    (left.absolute_y - right.absolute_y).abs() < 0.5
        && (left.max_y - right.max_y).abs() < 0.5
        && (left.viewport_height - right.viewport_height).abs() < 0.5
}

fn viewport_route_label(route: Route) -> &'static str {
    match route {
        Route::Gallery => "gallery",
        Route::Timeline => "timeline",
    }
}

fn route_surface_kind(route: Route) -> MediaSurfaceKind {
    match route {
        Route::Gallery => MediaSurfaceKind::Gallery,
        Route::Timeline => MediaSurfaceKind::Timeline,
    }
}

fn begin_viewport_drag_candidate(
    app: &mut Librapix,
    now: Instant,
    absolute_y: f32,
    viewport_height: f32,
) {
    app.viewport_drag.candidate_started_at = Some(now);
    app.viewport_drag.candidate_last_event_at = Some(now);
    app.viewport_drag.candidate_event_count = 1;
    app.viewport_drag.candidate_origin_y = absolute_y;
    app.viewport_drag.candidate_origin_height = viewport_height;
}

fn update_viewport_drag_candidate(
    app: &mut Librapix,
    now: Instant,
    absolute_y: f32,
    viewport_height: f32,
) {
    let continue_candidate = app
        .viewport_drag
        .candidate_last_event_at
        .is_some_and(|last| now.duration_since(last) <= VIEWPORT_DRAG_ACTIVATION_WINDOW);

    if !continue_candidate {
        begin_viewport_drag_candidate(app, now, absolute_y, viewport_height);
        return;
    }

    app.viewport_drag.candidate_last_event_at = Some(now);
    app.viewport_drag.candidate_event_count =
        app.viewport_drag.candidate_event_count.saturating_add(1);
}

fn should_activate_viewport_drag(
    app: &Librapix,
    absolute_y: f32,
    viewport_height: f32,
) -> Option<&'static str> {
    let scroll_delta = (absolute_y - app.viewport_drag.candidate_origin_y).abs();
    let height_delta = (viewport_height - app.viewport_drag.candidate_origin_height).abs();
    if height_delta > VIEWPORT_DRAG_HEIGHT_STABILITY_PX {
        return None;
    }

    if app.viewport_drag.candidate_event_count >= VIEWPORT_DRAG_ACTIVATION_EVENTS
        && scroll_delta >= VIEWPORT_DRAG_ACTIVATION_DELTA_PX
    {
        return Some("sustained_burst");
    }

    if app.viewport_drag.candidate_event_count >= VIEWPORT_DRAG_FAST_ACTIVATION_EVENTS
        && scroll_delta >= VIEWPORT_DRAG_FAST_ACTIVATION_DELTA_PX
    {
        return Some("large_jump_fast_path");
    }

    None
}

fn viewport_settle_profile(drag: &ViewportDragState) -> (Duration, &'static str) {
    if drag.max_step_delta_px >= VIEWPORT_SETTLE_LARGE_JUMP_THRESHOLD_PX {
        (VIEWPORT_SETTLE_DELAY_LARGE_JUMP, "large_jump_idle_guard")
    } else {
        (VIEWPORT_SETTLE_DELAY, "default_idle_guard")
    }
}

fn drag_mode_label(mode: ViewportDragMode) -> &'static str {
    match mode {
        ViewportDragMode::LivePreview => "live_preview",
        ViewportDragMode::SettleFirstPreview => "settle_first_preview",
    }
}

fn apply_viewport_snapshot(app: &mut Librapix, snapshot: ViewportSnapshot, preview_mode: bool) {
    let (absolute_y, max_y) = if preview_mode && app.viewport_drag.active {
        let frozen_max_y = app
            .viewport_drag
            .frozen_max_y
            .unwrap_or(snapshot.max_y.max(0.0));
        let clamped_y = snapshot.absolute_y.clamp(0.0, frozen_max_y);
        (clamped_y, frozen_max_y)
    } else {
        (snapshot.absolute_y, snapshot.max_y)
    };
    app.media_scroll_absolute_y = absolute_y;
    app.media_scroll_max_y = max_y;
    app.media_viewport_height = snapshot.viewport_height;
    sync_timeline_scrub_from_viewport(app, absolute_y, max_y);
}

fn maybe_apply_active_drag_snapshot(
    app: &mut Librapix,
    now: Instant,
    force: bool,
    preview_mode: bool,
) -> bool {
    let Some(snapshot) = app.viewport_drag.pending_viewport else {
        return false;
    };
    if !force
        && app
            .viewport_drag
            .last_applied_at
            .is_some_and(|last| now.duration_since(last) < VIEWPORT_DRAG_APPLY_INTERVAL)
    {
        return false;
    }
    if !force
        && app.viewport_drag.mode == ViewportDragMode::SettleFirstPreview
        && app
            .viewport_drag
            .last_event_at
            .is_some_and(|last| now.duration_since(last) < VIEWPORT_DRAG_PREVIEW_IDLE_APPLY_DELAY)
    {
        return false;
    }
    apply_viewport_snapshot(app, snapshot, preview_mode);
    app.viewport_drag.applied_updates = app.viewport_drag.applied_updates.saturating_add(1);
    app.viewport_drag.last_applied_at = Some(now);
    app.viewport_drag.pending_viewport = None;
    true
}

fn handle_media_viewport_changed(
    app: &mut Librapix,
    absolute_y: f32,
    max_y: f32,
    viewport_height: f32,
) {
    let snapshot = ViewportSnapshot {
        absolute_y: absolute_y.max(0.0),
        max_y: max_y.max(0.0),
        viewport_height: viewport_height.max(0.0),
    };
    let absolute_y = snapshot.absolute_y;
    let max_y = snapshot.max_y;
    let viewport_height = snapshot.viewport_height;
    let now = Instant::now();

    if app.viewport_drag.active
        && (app.media_scroll_absolute_y - absolute_y).abs() < 0.5
        && (app.media_viewport_height - viewport_height).abs() < 0.5
    {
        app.viewport_drag.max_y_preview_skips =
            app.viewport_drag.max_y_preview_skips.saturating_add(1);
        app.viewport_drag.coalesced_updates = app.viewport_drag.coalesced_updates.saturating_add(1);
        app.viewport_drag.pending_viewport = Some(snapshot);
        app.viewport_drag.last_event_at = Some(now);
        return;
    }

    let duplicate_pending = app.viewport_drag.active
        && app
            .viewport_drag
            .pending_viewport
            .is_some_and(|pending| viewport_snapshot_matches_values(pending, snapshot));
    if duplicate_pending || viewport_snapshot_matches(app, absolute_y, max_y, viewport_height) {
        app.viewport_drag.coalesced_updates = app.viewport_drag.coalesced_updates.saturating_add(1);
        return;
    }

    let route = viewport_route_label(app.state.active_route);
    let step_reference_y = app
        .viewport_drag
        .pending_viewport
        .map(|pending| pending.absolute_y)
        .unwrap_or(app.media_scroll_absolute_y);
    let step_delta = (absolute_y - step_reference_y).abs();
    if app.viewport_drag.active {
        app.viewport_drag.update_count = app.viewport_drag.update_count.saturating_add(1);
        app.viewport_drag.max_step_delta_px = app.viewport_drag.max_step_delta_px.max(step_delta);
        if app.viewport_drag.mode == ViewportDragMode::LivePreview
            && app.viewport_drag.max_step_delta_px >= VIEWPORT_DRAG_SETTLE_FIRST_DELTA_PX
        {
            app.viewport_drag.mode = ViewportDragMode::SettleFirstPreview;
            startup_log::log_info(
                "interaction.viewport.drag.mode.shift",
                &format!(
                    "route={route} from=live_preview to=settle_first_preview max_step_delta={:.1} threshold={:.1} updates={}",
                    app.viewport_drag.max_step_delta_px,
                    VIEWPORT_DRAG_SETTLE_FIRST_DELTA_PX,
                    app.viewport_drag.update_count,
                ),
            );
        }
        if app.viewport_drag.pending_viewport.is_some() {
            app.viewport_drag.latest_replacements =
                app.viewport_drag.latest_replacements.saturating_add(1);
        }
        app.viewport_drag.pending_viewport = Some(snapshot);
        let deferred_updates = app
            .viewport_drag
            .update_count
            .saturating_sub(app.viewport_drag.applied_updates);
        let should_log_update = app
            .viewport_drag
            .last_logged_at
            .is_none_or(|last| now.duration_since(last) >= VIEWPORT_ACTIVE_LOG_INTERVAL);
        if should_log_update {
            startup_log::log_info(
                "interaction.viewport.drag.update",
                &format!(
                    "route={route} updates={} coalesced={} applied={} deferred={} replaced={} applied_now={} max_step_delta={:.1} viewport_y={absolute_y:.1} max_y={max_y:.1} viewport_height={viewport_height:.1} overscan_px={:.1} projection_in_flight={} thumbnail_in_flight={} pending_projection={} pending_reconcile={}",
                    app.viewport_drag.update_count,
                    app.viewport_drag.coalesced_updates,
                    app.viewport_drag.applied_updates,
                    deferred_updates,
                    app.viewport_drag.latest_replacements,
                    false,
                    app.viewport_drag.max_step_delta_px,
                    media_virtual_overscan_px(true),
                    app.background.projection_in_flight,
                    app.background.thumbnail_in_flight,
                    app.background.pending_projection,
                    app.background.pending_reconcile,
                ),
            );
            app.viewport_drag.last_logged_at = Some(now);
        }
        app.viewport_drag.last_event_at = Some(now);
        return;
    }

    update_viewport_drag_candidate(app, now, absolute_y, viewport_height);
    if let Some(activation_reason) = should_activate_viewport_drag(app, absolute_y, viewport_height)
    {
        if app
            .last_viewport_drag_settled_at
            .is_some_and(|last| now.duration_since(last) <= VIEWPORT_DRAG_THRASH_WINDOW)
        {
            startup_log::log_warn(
                "interaction.viewport.drag.lifecycle.anomaly",
                &format!(
                    "route={route} kind=rapid_reactivation idle_since_settle_ms={} candidate_updates={} candidate_scroll_delta={:.1}",
                    app.last_viewport_drag_settled_at
                        .map(|last| now.duration_since(last).as_millis())
                        .unwrap_or_default(),
                    app.viewport_drag.candidate_event_count,
                    (absolute_y - app.viewport_drag.candidate_origin_y).abs(),
                ),
            );
        }
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(now);
        app.viewport_drag.update_count = 0;
        app.viewport_drag.applied_updates = 0;
        app.viewport_drag.coalesced_updates = 0;
        app.viewport_drag.latest_replacements = 0;
        app.viewport_drag.max_y_preview_skips = 0;
        app.viewport_drag.max_step_delta_px = (absolute_y - app.viewport_drag.candidate_origin_y)
            .abs()
            .max(step_delta);
        app.viewport_drag.mode = if app.viewport_drag.max_step_delta_px
            >= VIEWPORT_DRAG_SETTLE_FIRST_DELTA_PX
            || activation_reason == "large_jump_fast_path"
        {
            ViewportDragMode::SettleFirstPreview
        } else {
            ViewportDragMode::LivePreview
        };
        app.viewport_drag.pending_viewport = Some(snapshot);
        app.viewport_drag.frozen_max_y = Some(max_y);
        app.viewport_drag.last_applied_at = None;
        app.viewport_drag.last_logged_at = None;
        app.viewport_drag.candidate_started_at = None;
        app.viewport_drag.candidate_last_event_at = None;
        startup_log::log_info(
            "interaction.viewport.drag.start",
            &format!(
                "route={route} viewport_y={absolute_y:.1} max_y={max_y:.1} viewport_height={viewport_height:.1} overscan_px={:.1} render_window_logs_deferred=true activation_reason={activation_reason} candidate_updates={} candidate_scroll_delta={:.1} projection_in_flight={} thumbnail_in_flight={} pending_projection={} pending_reconcile={}",
                media_virtual_overscan_px(true),
                app.viewport_drag.candidate_event_count,
                (absolute_y - app.viewport_drag.candidate_origin_y).abs(),
                app.background.projection_in_flight,
                app.background.thumbnail_in_flight,
                app.background.pending_projection,
                app.background.pending_reconcile,
            ),
        );
        app.viewport_drag.update_count = app.viewport_drag.update_count.saturating_add(1);
        let applied_now = if app.viewport_drag.mode == ViewportDragMode::SettleFirstPreview {
            false
        } else {
            maybe_apply_active_drag_snapshot(app, now, true, true)
        };
        let deferred_updates = app
            .viewport_drag
            .update_count
            .saturating_sub(app.viewport_drag.applied_updates);
        app.viewport_drag.last_logged_at = Some(now);
        startup_log::log_info(
            "interaction.viewport.drag.update",
            &format!(
                "route={route} updates={} coalesced={} applied={} deferred={} replaced={} applied_now={} max_step_delta={:.1} viewport_y={absolute_y:.1} max_y={max_y:.1} viewport_height={viewport_height:.1} overscan_px={:.1} projection_in_flight={} thumbnail_in_flight={} pending_projection={} pending_reconcile={}",
                app.viewport_drag.update_count,
                app.viewport_drag.coalesced_updates,
                app.viewport_drag.applied_updates,
                deferred_updates,
                app.viewport_drag.latest_replacements,
                applied_now,
                app.viewport_drag.max_step_delta_px,
                media_virtual_overscan_px(true),
                app.background.projection_in_flight,
                app.background.thumbnail_in_flight,
                app.background.pending_projection,
                app.background.pending_reconcile,
            ),
        );
        startup_log::log_info(
            "interaction.viewport.drag.mode",
            &format!(
                "route={route} mode={} activation_reason={activation_reason} candidate_scroll_delta={:.1} settle_first_threshold={:.1}",
                drag_mode_label(app.viewport_drag.mode),
                (absolute_y - app.viewport_drag.candidate_origin_y).abs(),
                VIEWPORT_DRAG_SETTLE_FIRST_DELTA_PX,
            ),
        );
        if !applied_now {
            app.viewport_drag.last_logged_at = Some(now);
        }
        app.viewport_drag.last_event_at = Some(now);
        return;
    }

    apply_viewport_snapshot(app, snapshot, false);
}

fn settle_media_viewport_drag(app: &mut Librapix) {
    if !app.viewport_drag.active {
        return;
    }

    let now = Instant::now();
    let preview_applied_latest = maybe_apply_active_drag_snapshot(app, now, false, true);

    let Some(last_event_at) = app.viewport_drag.last_event_at else {
        app.viewport_drag = ViewportDragState::default();
        return;
    };
    let idle_for = now.duration_since(last_event_at);
    let (settle_delay, settle_profile) = viewport_settle_profile(&app.viewport_drag);
    if idle_for < settle_delay {
        return;
    }

    let route = viewport_route_label(app.state.active_route);
    let surface_kind = route_surface_kind(app.state.active_route);
    startup_log::log_info(
        "interaction.viewport.settle.start",
        &format!(
            "route={route} mode={} viewport_y={:.1} max_y={:.1} viewport_height={:.1} idle_ms={} settle_delay_ms={} settle_profile={settle_profile} preview_applied_latest={}",
            drag_mode_label(app.viewport_drag.mode),
            app.media_scroll_absolute_y,
            app.media_scroll_max_y,
            app.media_viewport_height,
            idle_for.as_millis(),
            settle_delay.as_millis(),
            preview_applied_latest,
        ),
    );
    let settle_applied_latest = maybe_apply_active_drag_snapshot(app, now, true, false);
    let drag_elapsed = app
        .viewport_drag
        .started_at
        .map(|started_at| now.duration_since(started_at))
        .unwrap_or_default();
    let preview_summary = app.drag_layout_preview.borrow().surface(surface_kind);
    startup_log::log_info(
        "interaction.viewport.settle.end",
        &format!(
            "route={route} updates={} coalesced={} processed={} deferred={} replaced={} max_only_skipped={} settle_applied_latest={} max_step_delta={:.1} overscan_px={:.1} viewport_y={:.1} max_y={:.1} viewport_height={:.1} projection_in_flight={} thumbnail_in_flight={} pending_projection={} pending_reconcile={} frozen_width={} measured_width={} width_changes={} suppressed_layout_rebuilds={} idle_ms={} settle_delay_ms={} settle_profile={settle_profile} elapsed_ms={}",
            app.viewport_drag.update_count,
            app.viewport_drag.coalesced_updates,
            app.viewport_drag.applied_updates,
            app.viewport_drag
                .update_count
                .saturating_sub(app.viewport_drag.applied_updates),
            app.viewport_drag.latest_replacements,
            app.viewport_drag.max_y_preview_skips,
            settle_applied_latest,
            app.viewport_drag.max_step_delta_px,
            media_virtual_overscan_px(false),
            app.media_scroll_absolute_y,
            app.media_scroll_max_y,
            app.media_viewport_height,
            app.background.projection_in_flight,
            app.background.thumbnail_in_flight,
            app.background.pending_projection,
            app.background.pending_reconcile,
            preview_summary
                .frozen_width_key
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            preview_summary
                .last_measured_width_key
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_owned()),
            preview_summary.width_change_count,
            preview_summary.suppressed_rebuilds,
            idle_for.as_millis(),
            settle_delay.as_millis(),
            drag_elapsed.as_millis(),
        ),
    );
    *app.drag_layout_preview.borrow_mut() = DragLayoutPreviewState::default();
    app.viewport_drag = ViewportDragState::default();
    app.last_viewport_drag_settled_at = Some(now);
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
    let show_make_short = selected_media_is_video(app);
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
        let make_short = button(
            row![
                image(assets::icon_youtube())
                    .width(Length::Fixed(16.0))
                    .height(Length::Fixed(16.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsMakeShortButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::OpenMakeShortDialog)
        .width(Length::Fill)
        .style(primary_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);

        if size.width < 220.0 {
            let mut col = column![open, open_folder, copy_file, copy_path].spacing(SPACE_XS);
            if show_make_short {
                col = col.push(make_short);
            }
            col.into()
        } else if size.width < 420.0 {
            let mut col = column![
                row![open, open_folder].spacing(SPACE_XS),
                row![copy_file, copy_path].spacing(SPACE_XS),
            ]
            .spacing(SPACE_XS);
            if show_make_short {
                col = col.push(make_short);
            }
            col.into()
        } else {
            if show_make_short {
                row![open, open_folder, copy_file, copy_path, make_short]
                    .spacing(SPACE_XS)
                    .into()
            } else {
                row![open, open_folder, copy_file, copy_path]
                    .spacing(SPACE_XS)
                    .into()
            }
        }
    })
    .into()
}

fn render_make_short_dialog(app: &Librapix) -> Element<'_, Message> {
    let is_running = matches!(
        app.make_short_dialog.run_state,
        MakeShortRunState::Running { .. }
    );
    let has_smooth = make_short_has_smooth_warning(&app.make_short_dialog);

    let effect_chip = |effect: ShortEffect, help_key: TextKey| {
        let active = app.make_short_dialog.effects.contains(&effect);
        let mut btn = button(text(effect.as_str()).size(FONT_CAPTION))
            .style(filter_chip_style(active))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]);
        if !is_running {
            btn = btn.on_press(Message::MakeShortToggleEffect(effect));
        }
        row![btn, render_help_badge(app.i18n.text(help_key)),].spacing(SPACE_XS)
    };

    let crop_chip = |position: CropPosition, label: &'static str| {
        let active = app.make_short_dialog.crop_position == position;
        let mut btn = button(text(label).size(FONT_CAPTION))
            .style(filter_chip_style(active))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]);
        if !is_running {
            btn = btn.on_press(Message::MakeShortSetCropPosition(position));
        }
        btn
    };

    let preset_chip = |preset: Preset, label: &'static str| {
        let active = app.make_short_dialog.preset == preset;
        let mut btn = button(text(label).size(FONT_CAPTION))
            .style(filter_chip_style(active))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]);
        if !is_running {
            btn = btn.on_press(Message::MakeShortSetPreset(preset));
        }
        btn
    };

    let running_status: Element<'_, Message> = match &app.make_short_dialog.run_state {
        MakeShortRunState::Running { stage, status } => {
            let indicator_color = if matches!(stage, GenerationStage::Finalizing) {
                SUCCESS_COLOR
            } else {
                ACCENT
            };
            column![
                text(status).size(FONT_BODY).color(TEXT_PRIMARY),
                row![
                    text("\u{25CF}").size(FONT_CAPTION).color(indicator_color),
                    text(app.i18n.text(TextKey::ActivityWorkingLabel))
                        .size(FONT_CAPTION)
                        .color(TEXT_SECONDARY),
                ]
                .spacing(SPACE_XS)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(SPACE_XS)
            .into()
        }
        MakeShortRunState::Success { .. } => text(app.i18n.text(TextKey::MakeShortSuccessLabel))
            .size(FONT_CAPTION)
            .color(SUCCESS_COLOR)
            .into(),
        MakeShortRunState::Failed { summary, details } => column![
            text(summary).size(FONT_CAPTION).color(WARNING_COLOR),
            text(details).size(FONT_CAPTION).color(TEXT_TERTIARY),
        ]
        .spacing(SPACE_2XS)
        .into(),
        MakeShortRunState::Idle => Space::new().into(),
    };

    let warning: Element<'_, Message> = if has_smooth {
        text(app.i18n.text(TextKey::MakeShortSmoothWarning))
            .size(FONT_CAPTION)
            .color(WARNING_COLOR)
            .into()
    } else {
        Space::new().into()
    };

    let validation_error: Element<'_, Message> =
        if let Some(error) = app.make_short_dialog.validation_error.as_ref() {
            text(error).size(FONT_CAPTION).color(WARNING_COLOR).into()
        } else {
            Space::new().into()
        };

    let mut generate_button = button(
        row![
            image(assets::icon_generate())
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0))
                .content_fit(ContentFit::Contain)
                .filter_method(FilterMethod::Linear),
            text(app.i18n.text(TextKey::MakeShortRunButton)).size(FONT_BODY),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
    )
    .style(primary_button_style)
    .padding([SPACE_XS as u16, SPACE_MD as u16]);
    if !is_running {
        generate_button = generate_button.on_press(Message::RunMakeShort);
    }

    let mut actions = row![
        button(text(app.i18n.text(TextKey::MakeShortCloseButton)).size(FONT_BODY))
            .on_press(Message::CloseMakeShortDialog)
            .style(subtle_button_style)
            .padding([SPACE_XS as u16, SPACE_MD as u16]),
        generate_button
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    if matches!(
        app.make_short_dialog.run_state,
        MakeShortRunState::Success { .. }
    ) {
        actions = actions.push(
            button(text(app.i18n.text(TextKey::MakeShortOpenFileButton)).size(FONT_BODY))
                .on_press(Message::OpenGeneratedShortFile)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        );
        actions = actions.push(
            button(text(app.i18n.text(TextKey::MakeShortOpenFolderButton)).size(FONT_BODY))
                .on_press(Message::OpenGeneratedShortFolder)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        );
    }

    let fade_active = app.make_short_dialog.add_fade;
    let mut fade_toggle = button(text(if fade_active { "On" } else { "Off" }).size(FONT_CAPTION))
        .style(filter_chip_style(fade_active))
        .padding([SPACE_2XS as u16, SPACE_SM as u16]);
    if !is_running {
        fade_toggle = fade_toggle.on_press(Message::MakeShortSetAddFade(!fade_active));
    }

    let output_row = row![
        text_input(
            app.i18n.text(TextKey::MakeShortOutputPathLabel),
            &app.make_short_dialog.output_file_input
        )
        .on_input(Message::MakeShortOutputPathChanged)
        .width(Length::Fill)
        .style(field_input_style),
        button(
            row![
                image(assets::icon_browse())
                    .width(Length::Fixed(14.0))
                    .height(Length::Fixed(14.0))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::MakeShortChooseOutputButton)).size(FONT_CAPTION),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
        )
        .on_press(Message::MakeShortBrowseOutputPath)
        .style(subtle_button_style)
        .padding([SPACE_2XS as u16, SPACE_SM as u16]),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    let dialog_content = column![
        row![
            text(app.i18n.text(TextKey::MakeShortDialogTitle))
                .size(FONT_TITLE)
                .color(TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
                .on_press(Message::CloseMakeShortDialog)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .align_y(iced::Alignment::Center),
        h_divider(),
        section_heading(app.i18n.text(TextKey::MakeShortOutputPathLabel)),
        output_row,
        render_help_badge(app.i18n.text(TextKey::MakeShortHelpOutput)),
        section_heading(app.i18n.text(TextKey::MakeShortEffectsLabel)),
        row![
            effect_chip(ShortEffect::Clean, TextKey::MakeShortHelpEffectsClean),
            effect_chip(ShortEffect::Enhanced, TextKey::MakeShortHelpEffectsEnhanced),
            effect_chip(
                ShortEffect::Cinematic,
                TextKey::MakeShortHelpEffectsCinematic
            ),
        ]
        .spacing(SPACE_SM),
        row![
            effect_chip(ShortEffect::Night, TextKey::MakeShortHelpEffectsNight),
            effect_chip(ShortEffect::Scenic, TextKey::MakeShortHelpEffectsScenic),
            effect_chip(ShortEffect::Smooth, TextKey::MakeShortHelpEffectsSmooth),
        ]
        .spacing(SPACE_SM),
        warning,
        h_divider(),
        section_heading(app.i18n.text(TextKey::MakeShortCropLabel)),
        row![
            crop_chip(CropPosition::Center, "center"),
            crop_chip(CropPosition::Left, "left"),
            crop_chip(CropPosition::Right, "right"),
            render_help_badge(app.i18n.text(TextKey::MakeShortHelpCrop)),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        row![
            text(app.i18n.text(TextKey::MakeShortFadeLabel))
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
            fade_toggle,
            render_help_badge(app.i18n.text(TextKey::MakeShortHelpFade)),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        h_divider(),
        row![
            text(app.i18n.text(TextKey::MakeShortSpeedLabel))
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
            text_input("1.0", &app.make_short_dialog.speed_input)
                .on_input(Message::MakeShortSpeedChanged)
                .style(field_input_style)
                .width(Length::Fixed(80.0)),
            render_help_badge(app.i18n.text(TextKey::MakeShortHelpSpeed)),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        row![
            text(app.i18n.text(TextKey::MakeShortCrfLabel))
                .size(FONT_CAPTION)
                .color(TEXT_SECONDARY),
            text_input("18", &app.make_short_dialog.crf_input)
                .on_input(Message::MakeShortCrfChanged)
                .style(field_input_style)
                .width(Length::Fixed(80.0)),
            render_help_badge(app.i18n.text(TextKey::MakeShortHelpCrf)),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        section_heading(app.i18n.text(TextKey::MakeShortPresetLabel)),
        row![
            preset_chip(Preset::Fast, "fast"),
            preset_chip(Preset::Medium, "medium"),
            preset_chip(Preset::Slow, "slow"),
            render_help_badge(app.i18n.text(TextKey::MakeShortHelpPreset)),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        validation_error,
        running_status,
        h_divider(),
        actions,
    ]
    .spacing(SPACE_SM);

    let dialog = container(dialog_content)
        .width(Length::Fill)
        .max_width(620.0)
        .padding(SPACE_LG as u16)
        .style(modal_dialog_style);

    render_modal_overlay(dialog.into())
}

fn make_short_has_smooth_warning(state: &MakeShortDialogState) -> bool {
    state.effects.contains(&ShortEffect::Smooth)
}

fn render_help_badge<'a>(tooltip_text: &'a str) -> Element<'a, Message> {
    tooltip(
        container(text("(?)").size(FONT_CAPTION).color(TEXT_TERTIARY))
            .padding([0, SPACE_XS as u16]),
        container(text(tooltip_text).size(FONT_CAPTION).color(TEXT_PRIMARY))
            .padding([SPACE_XS as u16, SPACE_SM as u16])
            .style(card_style),
        tooltip::Position::Top,
    )
    .into()
}

fn do_prepare_short(request: ShortGenerationRequest) -> Result<PreparedShortJob, String> {
    prepare_generation(&request)
        .map(|prepared| PreparedShortJob { prepared })
        .map_err(|error| error.to_string())
}

fn do_generate_short(job: PreparedShortJob) -> Result<ShortGenerationResult, String> {
    run_generation(&job.prepared).map_err(|error| error.to_string())
}

fn toggle_make_short_effect(app: &mut Librapix, effect: ShortEffect) {
    let effects = &mut app.make_short_dialog.effects;
    if effects.contains(&effect) {
        effects.retain(|existing| *existing != effect);
        if effects.is_empty() {
            effects.push(ShortEffect::Enhanced);
        }
    } else if effect == ShortEffect::Clean {
        effects.clear();
        effects.push(ShortEffect::Clean);
    } else {
        effects.retain(|existing| *existing != ShortEffect::Clean);
        effects.push(effect);
    }
}

fn open_make_short_dialog(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        return;
    };
    let Some(path) = resolve_media_path_for_action(app, media_id) else {
        return;
    };
    let is_video = app
        .media_cache
        .get(&media_id)
        .is_some_and(|details| details.media_kind.eq_ignore_ascii_case("video"));
    if !is_video {
        return;
    }
    let default_output =
        default_output_file_path(&path, app.runtime.default_shorts_output_dir.as_deref());

    app.make_short_dialog.open = true;
    app.make_short_dialog.input_file = Some(path);
    app.make_short_dialog.output_file_input = default_output.display().to_string();
    app.make_short_dialog.validation_error = None;
    app.make_short_dialog.run_state = MakeShortRunState::Idle;
}

fn build_short_request_from_dialog(app: &mut Librapix) -> Option<ShortGenerationRequest> {
    let Some(input_file) = app.make_short_dialog.input_file.clone() else {
        app.make_short_dialog.validation_error =
            Some(app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned());
        return None;
    };
    let speed = match app.make_short_dialog.speed_input.trim().parse::<f64>() {
        Ok(value) => value,
        Err(_) => {
            app.make_short_dialog.validation_error =
                Some(app.i18n.text(TextKey::MakeShortHelpSpeed).to_owned());
            return None;
        }
    };
    let crf = match app.make_short_dialog.crf_input.trim().parse::<i32>() {
        Ok(value) => value,
        Err(_) => {
            app.make_short_dialog.validation_error =
                Some(app.i18n.text(TextKey::MakeShortHelpCrf).to_owned());
            return None;
        }
    };

    let output = if app.make_short_dialog.output_file_input.trim().is_empty() {
        default_output_file_path(
            &input_file,
            app.runtime.default_shorts_output_dir.as_deref(),
        )
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        lexical_normalize_path(
            Path::new(app.make_short_dialog.output_file_input.trim()),
            &cwd,
        )
    };

    let request = ShortGenerationRequest {
        input_file,
        output_file: output,
        options: ShortGenerationOptions {
            effects: app.make_short_dialog.effects.clone(),
            crop_position: app.make_short_dialog.crop_position,
            add_fade: app.make_short_dialog.add_fade,
            speed,
            crf,
            preset: app.make_short_dialog.preset,
        },
    };

    app.make_short_dialog.validation_error = None;
    Some(request)
}

fn save_shorts_output_dir_setting(app: &mut Librapix) {
    let trimmed = app.shorts_output_dir_input.trim();
    let new_value = if trimmed.is_empty() {
        None
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Some(lexical_normalize_path(Path::new(trimmed), &cwd))
    };

    let Ok(mut config) = load_from_path(&app.runtime.config_file) else {
        return;
    };
    config.video_tools.default_shorts_output_dir = new_value.clone();
    if save_to_path(&app.runtime.config_file, &config).is_ok() {
        app.runtime.default_shorts_output_dir = new_value;
    }
}

struct BootstrapRuntime {
    locale: Locale,
    theme_preference: ThemePreference,
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    config_file: PathBuf,
    configured_library_roots: Vec<PathBuf>,
    default_shorts_output_dir: Option<PathBuf>,
}

fn bootstrap_runtime() -> BootstrapRuntime {
    let bootstrap_started_at = Instant::now();
    let mut runtime = BootstrapRuntime {
        locale: Locale::EnUs,
        theme_preference: ThemePreference::System,
        database_file: PathBuf::from("librapix.db"),
        thumbnails_dir: PathBuf::from("thumbnails"),
        config_file: PathBuf::new(),
        configured_library_roots: Vec::new(),
        default_shorts_output_dir: None,
    };

    startup_log::log_info("bootstrap.config_load.start", "");
    let config_started_at = Instant::now();
    let loaded = match load_or_create() {
        Ok(config) => config,
        Err(error) => {
            startup_log::log_error("bootstrap.config_load.failed", &error.to_string());
            return runtime;
        }
    };
    startup_log::log_duration(
        "bootstrap.config_load.end",
        config_started_at.elapsed(),
        &format!("config_file={}", loaded.paths.config_file.display()),
    );

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
    runtime.configured_library_roots = loaded
        .config
        .library_source_roots
        .into_iter()
        .map(|source| source.path)
        .collect();
    runtime.default_shorts_output_dir = loaded.config.video_tools.default_shorts_output_dir;
    startup_log::log_duration(
        "bootstrap.complete",
        bootstrap_started_at.elapsed(),
        &format!(
            "database_file={} thumbnails_dir={} configured_roots={}",
            runtime.database_file.display(),
            runtime.thumbnails_dir.display(),
            runtime.configured_library_roots.len(),
        ),
    );
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
    set_activity_stage(
        app,
        TextKey::StageCheckingLibrariesLabel,
        app.i18n.text(TextKey::StageScanningFilesLabel),
        true,
    );

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
    let started_at = Instant::now();
    app.details_editing_tag = None;
    app.details_tag_input.clear();
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_tags.clear();
        startup_log::log_info(
            "interaction.detail_load.request.end",
            "media_id=none selected=false result=no_selection",
        );
        return;
    };
    startup_log::log_info(
        "interaction.detail_load.request.start",
        &selected_media_context(app, media_id),
    );
    startup_log::log_info(
        "interaction.detail_working.set",
        &format!(
            "owner=detail_load media_id={} route={} {}",
            media_id,
            route_name(app.state.active_route),
            filter_state_summary(app),
        ),
    );
    let storage_started_at = Instant::now();
    let details = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    log_interaction_duration(
        "interaction.detail_load.storage_lookup",
        storage_started_at.elapsed(),
        &format!("media_id={media_id} found={}", details.is_some()),
    );
    if let Some(details) = details {
        app.details_title = details
            .absolute_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| details.absolute_path.display().to_string());
        let thumbnail_lookup_started_at = Instant::now();
        startup_log::log_info(
            "interaction.detail_thumbnail.lookup.start",
            &format!(
                "media_id={} kind={} path={} size={DETAIL_THUMB_SIZE}",
                media_id,
                details.media_kind,
                details.absolute_path.display(),
            ),
        );
        let (preview_path, preview_source) = resolve_existing_detail_preview_path(
            &app.runtime.thumbnails_dir,
            &details,
            browse_thumbnail_path(app, media_id),
        );
        app.details_preview_path = preview_path;
        log_interaction_duration(
            "interaction.detail_thumbnail.lookup.end",
            thumbnail_lookup_started_at.elapsed(),
            &format!(
                "media_id={} kind={} resolved={} source={preview_source}",
                media_id,
                details.media_kind,
                app.details_preview_path.is_some(),
            ),
        );
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
        app.details_loaded_media_ids.insert(media_id);
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
    } else {
        app.details_lines.clear();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_tags.clear();
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
    }
    log_interaction_duration(
        "interaction.detail_load.request.end",
        started_at.elapsed(),
        &format!(
            "{} result={} details_lines={} details_tags={} preview_path={}",
            selected_media_context(app, media_id),
            app.details_action_status,
            app.details_lines.len(),
            app.details_tags.len(),
            app.details_preview_path.is_some(),
        ),
    );
    startup_log::log_info(
        "interaction.detail_working.clear",
        &format!(
            "owner=detail_load media_id={} route={} result={} preview_path={}",
            media_id,
            route_name(app.state.active_route),
            app.details_action_status,
            app.details_preview_path.is_some(),
        ),
    );
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
    let started_at = Instant::now();
    startup_log::log_info(
        "interaction.detail_tags.load.start",
        &format!("media_id={media_id}"),
    );
    let media_row = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    let Some(media_row) = media_row else {
        app.details_tags.clear();
        log_interaction_duration(
            "interaction.detail_tags.load.end",
            started_at.elapsed(),
            &format!("media_id={media_id} tags=0 media_row_found=false"),
        );
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
    log_interaction_duration(
        "interaction.detail_tags.load.end",
        started_at.elapsed(),
        &format!(
            "media_id={media_id} tags={} inherited_candidates={}",
            app.details_tags.len(),
            inherited_names.len(),
        ),
    );
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

fn browse_item_from_catalog_row(
    i18n: Translator,
    row: &CatalogMediaRecord,
    thumbnail_path: Option<PathBuf>,
) -> BrowseItem {
    BrowseItem {
        media_id: row.media_id,
        title: row.file_name.clone(),
        thumbnail_path,
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

fn collect_available_filter_tags(rows: &[CatalogMediaRecord]) -> Vec<String> {
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

fn catalog_row_matches_tag_filter(row: &CatalogMediaRecord, tag_filter: Option<&str>) -> bool {
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

fn resolve_existing_detail_preview_path(
    thumbnails_dir: &Path,
    row: &librapix_storage::MediaReadModel,
    browse_fallback: Option<PathBuf>,
) -> (Option<PathBuf>, &'static str) {
    let detail_path = thumbnail_path(
        thumbnails_dir,
        &row.absolute_path,
        row.file_size_bytes,
        row.modified_unix_seconds,
        DETAIL_THUMB_SIZE,
    );
    if detail_path.is_file() {
        (Some(detail_path), "detail_artifact")
    } else if let Some(path) = browse_fallback {
        (Some(path), "browse_thumbnail")
    } else {
        (None, "none")
    }
}

fn relative_artifact_path(artifact_root: &Path, artifact_path: &Path) -> Option<PathBuf> {
    artifact_path
        .strip_prefix(artifact_root)
        .map(PathBuf::from)
        .ok()
        .or_else(|| artifact_path.file_name().map(PathBuf::from))
}

fn resolve_artifact_path(artifact_root: &Path, relative_path: &Path) -> PathBuf {
    if relative_path.is_absolute() {
        relative_path.to_path_buf()
    } else {
        artifact_root.join(relative_path)
    }
}

fn projection_priority_media_ids(
    active_route: Route,
    search_query: &str,
    gallery_items: &[BrowseItem],
    timeline_items: &[BrowseItem],
    search_items: &[BrowseItem],
    limit: usize,
) -> Vec<i64> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    if !search_query.trim().is_empty() {
        collect_priority_media_ids(search_items, &mut seen, &mut ordered, limit);
    }

    if ordered.len() < limit {
        let browse_items = if matches!(active_route, Route::Timeline) {
            timeline_items
        } else {
            gallery_items
        };
        collect_priority_media_ids(browse_items, &mut seen, &mut ordered, limit);
    }

    ordered
}

fn validate_ready_artifact_map(
    artifact_root: &Path,
    artifacts: &[DerivedArtifactRecord],
    priority_media_ids: &HashSet<i64>,
) -> (
    std::collections::HashMap<i64, PathBuf>,
    ArtifactValidationSummary,
) {
    let mut paths = HashMap::new();
    let mut summary = ArtifactValidationSummary::default();

    for artifact in artifacts {
        let Some(relative_path) = artifact.relative_path.as_deref() else {
            summary.rejected_missing_path += 1;
            if priority_media_ids.contains(&artifact.media_id) {
                startup_log::log_warn(
                    "startup.thumbnail_lookup.artifact_rejected",
                    &format!(
                        "media_id={} variant={} reason=missing_relative_path",
                        artifact.media_id, artifact.artifact_variant
                    ),
                );
            }
            continue;
        };
        let resolved = resolve_artifact_path(artifact_root, relative_path);
        if !resolved.is_file() {
            summary.rejected_missing_file += 1;
            if priority_media_ids.contains(&artifact.media_id) {
                startup_log::log_warn(
                    "startup.thumbnail_lookup.artifact_rejected",
                    &format!(
                        "media_id={} variant={} reason=missing_file path={}",
                        artifact.media_id,
                        artifact.artifact_variant,
                        resolved.display()
                    ),
                );
            }
            continue;
        }
        summary.accepted += 1;
        paths.insert(artifact.media_id, resolved);
    }

    (paths, summary)
}

fn deterministic_thumbnail_file(
    thumbnails_dir: &Path,
    row: &CatalogMediaRecord,
    max_edge: u32,
) -> Option<PathBuf> {
    let path = thumbnail_path(
        thumbnails_dir,
        &row.absolute_path,
        row.file_size_bytes,
        row.modified_unix_seconds,
        max_edge,
    );
    path.is_file().then_some(path)
}

fn resolve_projection_thumbnail_lookup(
    input: ProjectionThumbnailLookupInput<'_>,
) -> ProjectionThumbnailLookup {
    let lookup_started_at = Instant::now();
    let priority_ids = projection_priority_media_ids(
        input.active_route,
        input.search_query,
        input.gallery_items,
        input.timeline_items,
        input.search_items,
        STARTUP_THUMBNAIL_PRIORITY_LIMIT,
    );
    let priority_set = priority_ids.iter().copied().collect::<HashSet<_>>();
    startup_log::log_info(
        "startup.thumbnail_lookup.start",
        &format!(
            "generation={} requested_media={} priority_media={}",
            input.generation,
            input.all_rows.len(),
            priority_ids.len(),
        ),
    );

    let gallery_validation_started_at = Instant::now();
    let (mut resolved_paths, gallery_validation) =
        validate_ready_artifact_map(input.thumbnails_dir, input.gallery_artifacts, &priority_set);
    startup_log::log_duration(
        "startup.thumbnail_lookup.validate_gallery_artifacts",
        gallery_validation_started_at.elapsed(),
        &format!(
            "generation={} accepted={} rejected_missing_path={} rejected_missing_file={}",
            input.generation,
            gallery_validation.accepted,
            gallery_validation.rejected_missing_path,
            gallery_validation.rejected_missing_file,
        ),
    );
    let mut reusable_media_ids = resolved_paths.keys().copied().collect::<HashSet<_>>();

    let deterministic_gallery_started_at = Instant::now();
    let mut exact_deterministic_reused = 0usize;
    for row in input.all_rows {
        if reusable_media_ids.contains(&row.media_id) {
            continue;
        }
        if let Some(path) =
            deterministic_thumbnail_file(input.thumbnails_dir, row, GALLERY_THUMB_SIZE)
        {
            reusable_media_ids.insert(row.media_id);
            resolved_paths.insert(row.media_id, path);
            exact_deterministic_reused += 1;
        }
    }
    startup_log::log_duration(
        "startup.thumbnail_lookup.exact_deterministic",
        deterministic_gallery_started_at.elapsed(),
        &format!(
            "generation={} reused={}",
            input.generation, exact_deterministic_reused,
        ),
    );

    let detail_validation_started_at = Instant::now();
    let (detail_paths, detail_validation) =
        validate_ready_artifact_map(input.thumbnails_dir, input.detail_artifacts, &priority_set);
    startup_log::log_duration(
        "startup.thumbnail_lookup.validate_detail_artifacts",
        detail_validation_started_at.elapsed(),
        &format!(
            "generation={} accepted={} rejected_missing_path={} rejected_missing_file={}",
            input.generation,
            detail_validation.accepted,
            detail_validation.rejected_missing_path,
            detail_validation.rejected_missing_file,
        ),
    );

    let mut fallback_catalog_reused = 0usize;
    for (media_id, path) in detail_paths {
        if reusable_media_ids.insert(media_id) {
            resolved_paths.insert(media_id, path);
            fallback_catalog_reused += 1;
        }
    }

    let deterministic_detail_started_at = Instant::now();
    let mut fallback_deterministic_reused = 0usize;
    for media_id in &priority_ids {
        if reusable_media_ids.contains(media_id) {
            continue;
        }
        let Some(row) = input.row_lookup.get(media_id).copied() else {
            continue;
        };
        if let Some(path) =
            deterministic_thumbnail_file(input.thumbnails_dir, row, DETAIL_THUMB_SIZE)
        {
            reusable_media_ids.insert(*media_id);
            resolved_paths.insert(*media_id, path);
            fallback_deterministic_reused += 1;
        }
    }
    startup_log::log_duration(
        "startup.thumbnail_lookup.fallback_deterministic",
        deterministic_detail_started_at.elapsed(),
        &format!(
            "generation={} reused={}",
            input.generation, fallback_deterministic_reused,
        ),
    );

    let scheduled_generation = input
        .all_rows
        .iter()
        .filter(|row| !reusable_media_ids.contains(&row.media_id))
        .filter(|row| {
            row.media_kind.eq_ignore_ascii_case("image")
                || row.media_kind.eq_ignore_ascii_case("video")
        })
        .count();
    let priority_placeholder = priority_ids
        .iter()
        .filter(|media_id| !reusable_media_ids.contains(media_id))
        .count();

    let summary = ThumbnailLookupSummary {
        requested_media: input.all_rows.len(),
        priority_media: priority_ids.len(),
        exact_catalog_reused: gallery_validation.accepted,
        exact_deterministic_reused,
        fallback_catalog_reused,
        fallback_deterministic_reused,
        priority_placeholder,
        scheduled_generation,
        rejected_gallery_missing_path: gallery_validation.rejected_missing_path,
        rejected_gallery_missing_file: gallery_validation.rejected_missing_file,
        rejected_detail_missing_path: detail_validation.rejected_missing_path,
        rejected_detail_missing_file: detail_validation.rejected_missing_file,
    };

    startup_log::log_info(
        "startup.thumbnail_lookup.exact_reuse",
        &format!(
            "generation={} catalog_hits={} deterministic_hits={}",
            input.generation, summary.exact_catalog_reused, summary.exact_deterministic_reused,
        ),
    );
    startup_log::log_info(
        "startup.thumbnail_lookup.fallback_reuse",
        &format!(
            "generation={} catalog_hits={} deterministic_hits={}",
            input.generation,
            summary.fallback_catalog_reused,
            summary.fallback_deterministic_reused,
        ),
    );
    startup_log::log_info(
        "startup.thumbnail_lookup.placeholder",
        &format!(
            "generation={} priority_items_without_reuse={}",
            input.generation, summary.priority_placeholder,
        ),
    );
    startup_log::log_info(
        "startup.thumbnail_lookup.scheduled_generation",
        &format!(
            "generation={} items={}",
            input.generation, summary.scheduled_generation,
        ),
    );
    startup_log::log_info(
        "startup.thumbnail_lookup.rejected_existing",
        &format!(
            "generation={} gallery_missing_path={} gallery_missing_file={} detail_missing_path={} detail_missing_file={}",
            input.generation,
            summary.rejected_gallery_missing_path,
            summary.rejected_gallery_missing_file,
            summary.rejected_detail_missing_path,
            summary.rejected_detail_missing_file,
        ),
    );
    startup_log::log_duration(
        "startup.thumbnail_lookup.end",
        lookup_started_at.elapsed(),
        &format!(
            "generation={} requested_media={} priority_media={} reused_exact={} reused_fallback={} priority_placeholder={} scheduled_generation={}",
            input.generation,
            summary.requested_media,
            summary.priority_media,
            summary.exact_catalog_reused + summary.exact_deterministic_reused,
            summary.fallback_catalog_reused + summary.fallback_deterministic_reused,
            summary.priority_placeholder,
            summary.scheduled_generation,
        ),
    );

    ProjectionThumbnailLookup {
        resolved_paths,
        reusable_media_ids,
        summary,
    }
}

fn populate_media_cache(
    storage: &Storage,
    cache: &mut std::collections::HashMap<i64, CachedDetails>,
    row_lookup: &HashMap<i64, &CatalogMediaRecord>,
    media_ids: &[i64],
    thumbnails_dir: &std::path::Path,
) {
    cache.clear();

    if media_ids.is_empty() {
        return;
    }

    let detail_artifacts = storage
        .list_ready_derived_artifacts_for_media_ids(
            media_ids,
            DerivedArtifactKind::Thumbnail,
            DETAIL_THUMB_VARIANT,
        )
        .unwrap_or_default();
    let (detail_artifact_paths, _) =
        validate_ready_artifact_map(thumbnails_dir, &detail_artifacts, &HashSet::new());

    for media_id in media_ids {
        let Some(row) = row_lookup.get(media_id).copied() else {
            continue;
        };
        cache.insert(
            row.media_id,
            CachedDetails {
                absolute_path: row.absolute_path.clone(),
                media_kind: row.media_kind.clone(),
                file_size_bytes: row.file_size_bytes,
                modified_unix_seconds: row.modified_unix_seconds,
                width_px: row.width_px,
                height_px: row.height_px,
                detail_thumbnail_path: detail_artifact_paths.get(&row.media_id).cloned(),
            },
        );
    }
}

fn collect_priority_media_ids(
    items: &[BrowseItem],
    seen: &mut HashSet<i64>,
    ordered: &mut Vec<i64>,
    limit: usize,
) {
    for item in items {
        if item.is_group_header || item.media_id <= 0 {
            continue;
        }
        if seen.insert(item.media_id) {
            ordered.push(item.media_id);
            if ordered.len() >= limit {
                break;
            }
        }
    }
}

fn startup_priority_media_ids(app: &Librapix, limit: usize) -> Vec<i64> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    if !app.state.search_query.trim().is_empty() {
        collect_priority_media_ids(&app.search_items, &mut seen, &mut ordered, limit);
    }

    if ordered.len() < limit {
        let browse_items = if matches!(app.state.active_route, Route::Timeline) {
            &app.timeline_items
        } else {
            &app.gallery_items
        };
        collect_priority_media_ids(browse_items, &mut seen, &mut ordered, limit);
    }

    ordered
}

fn load_media_details_cached(app: &mut Librapix) {
    let started_at = Instant::now();
    app.details_editing_tag = None;
    app.details_tag_input.clear();
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
        app.details_tags.clear();
        startup_log::log_info(
            "interaction.media_select.cache.end",
            "media_id=none selected=false result=no_selection",
        );
        return;
    };
    startup_log::log_info(
        "interaction.media_select.cache.start",
        &selected_media_context(app, media_id),
    );

    if let Some(cached) = app.media_cache.get(&media_id) {
        app.details_title = cached
            .absolute_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| cached.absolute_path.display().to_string());
        app.details_preview_path = cached
            .detail_thumbnail_path
            .clone()
            .or_else(|| browse_thumbnail_path(app, media_id));
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
        app.details_loaded_media_ids.insert(media_id);
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
        log_interaction_duration(
            "interaction.media_select.cache.end",
            started_at.elapsed(),
            &format!(
                "{} cache_hit=true details_lines={} details_tags={} preview_path={}",
                selected_media_context(app, media_id),
                app.details_lines.len(),
                app.details_tags.len(),
                app.details_preview_path.is_some(),
            ),
        );
    } else {
        startup_log::log_info(
            "interaction.media_select.cache.miss",
            &selected_media_context(app, media_id),
        );
        load_media_details(app);
        log_interaction_duration(
            "interaction.media_select.cache.end",
            started_at.elapsed(),
            &format!(
                "{} cache_hit=false details_lines={} details_tags={} preview_path={}",
                selected_media_context(app, media_id),
                app.details_lines.len(),
                app.details_tags.len(),
                app.details_preview_path.is_some(),
            ),
        );
    }
}

fn rows_to_projection_media(rows: &[CatalogMediaRecord]) -> Vec<ProjectionMedia> {
    rows.iter()
        .map(|row| ProjectionMedia {
            media_id: row.media_id,
            absolute_path: row.absolute_path.display().to_string(),
            media_kind: row.media_kind.clone(),
            modified_unix_seconds: row.modified_unix_seconds,
            tags: row.tags.clone(),
            timeline_day_key: row.timeline_day_key.clone(),
            timeline_month_key: row.timeline_month_key.clone(),
            timeline_year_key: row.timeline_year_key.clone(),
        })
        .collect()
}

fn browse_thumbnail_path(app: &Librapix, media_id: i64) -> Option<PathBuf> {
    app.gallery_items
        .iter()
        .chain(app.timeline_items.iter())
        .chain(app.search_items.iter())
        .find(|item| item.media_id == media_id)
        .and_then(|item| item.thumbnail_path.clone())
}

fn ensure_valid_library_filter(app: &mut Librapix) {
    if let Some(root_id) = app.filter_source_root_id
        && !app
            .state
            .library_roots
            .iter()
            .any(|root| root.id == root_id)
    {
        app.filter_source_root_id = None;
    }
}

fn reconcile_selected_media_after_projection(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        return;
    };

    if app.media_cache.contains_key(&media_id) {
        load_media_details_cached(app);
    } else {
        app.state.apply(AppMessage::SetSelectedMedia);
        app.state.set_selected_media(None);
        load_media_details_cached(app);
    }
}

fn collect_preview_lines(items: &[BrowseItem]) -> Vec<String> {
    items.iter().map(|item| item.line.clone()).collect()
}

fn thumbnail_status_text(
    i18n: Translator,
    generated: usize,
    reused: usize,
    failed: usize,
) -> String {
    format!(
        "{}: {}={generated}, {}={reused}, {}={failed}",
        i18n.text(TextKey::ThumbnailStatusLabel),
        i18n.text(TextKey::ThumbnailGeneratedLabel),
        i18n.text(TextKey::ThumbnailReusedLabel),
        i18n.text(TextKey::ThumbnailFailedLabel),
    )
}

fn snapshot_payload_from_projection(
    gallery_items: &[BrowseItem],
    available_filter_tags: &[String],
) -> Option<String> {
    let updated_unix_seconds = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .unwrap_or_default();
    let payload = PersistedProjectionSnapshot {
        version: PROJECTION_SNAPSHOT_VERSION,
        gallery_items: gallery_items
            .iter()
            .take(STARTUP_SNAPSHOT_GALLERY_LIMIT)
            .cloned()
            .collect(),
        gallery_total_items: gallery_items.len(),
        available_filter_tags: available_filter_tags.to_vec(),
        updated_unix_seconds,
    };
    serde_json::to_string(&payload).ok()
}

fn start_snapshot_hydrate(app: &mut Librapix) -> Task<Message> {
    app.background.startup_ready = false;
    app.background.snapshot_loaded = false;
    app.background.deferred_thumbnail_due_at = None;
    app.background.deferred_thumbnail_queue.clear();
    app.background.startup_deferred_gallery_refresh = false;
    app.background.startup_deferred_timeline_refresh = false;
    app.background.startup_gallery_continuation_due_at = None;
    app.background.snapshot_generation = app.background.snapshot_generation.saturating_add(1);
    let generation = app.background.snapshot_generation;
    app.startup_metrics.snapshot_hydrate_started_at = Some(Instant::now());
    app.startup_metrics.snapshot_apply_started_at = None;
    app.startup_metrics.reconcile_started_at = None;
    app.startup_metrics.projection_started_at = None;
    app.startup_metrics.startup_thumbnail_started_at = None;
    app.startup_metrics.deferred_thumbnail_started_at = None;
    app.startup_metrics.first_usable_gallery_recorded = false;
    app.startup_metrics.startup_ready_recorded = false;
    set_activity_stage(app, TextKey::StageLoadingSnapshotLabel, String::new(), true);
    app.activity_progress.items_done = 0;
    app.activity_progress.items_total = None;
    app.activity_progress.queue_depth = 0;
    app.activity_progress.last_error = None;
    startup_log::log_info(
        "startup.snapshot_hydrate.start",
        &format!("generation={generation}"),
    );

    let input = SnapshotHydrateInput {
        generation,
        database_file: app.runtime.database_file.clone(),
        configured_library_roots: app.runtime.configured_library_roots.clone(),
    };
    Task::perform(async move { do_snapshot_hydrate(input) }, |result| {
        Message::HydrateSnapshotComplete(Box::new(result))
    })
}

fn do_snapshot_hydrate(input: SnapshotHydrateInput) -> SnapshotHydrateResult {
    let hydrate_started_at = Instant::now();
    let mut out = SnapshotHydrateResult {
        generation: input.generation,
        ..Default::default()
    };
    let (storage, storage_metrics) = match Storage::open_with_metrics(&input.database_file) {
        Ok(value) => value,
        Err(error) => {
            startup_log::log_error(
                "startup.snapshot_hydrate.storage_open.failed",
                &format!(
                    "generation={} path={} error={error}",
                    input.generation,
                    input.database_file.display(),
                ),
            );
            return out;
        }
    };
    log_storage_open_metrics("snapshot_hydrate", &input.database_file, &storage_metrics);

    if storage
        .list_source_roots()
        .map(|roots| roots.is_empty())
        .unwrap_or(false)
    {
        let seed_started_at = Instant::now();
        for configured_root in &input.configured_library_roots {
            if let Err(error) = storage.upsert_source_root(configured_root) {
                startup_log::log_error(
                    "startup.snapshot_hydrate.seed_root.failed",
                    &format!(
                        "generation={} root={} error={error}",
                        input.generation,
                        configured_root.display(),
                    ),
                );
            }
        }
        startup_log::log_duration(
            "startup.snapshot_hydrate.seed_roots",
            seed_started_at.elapsed(),
            &format!(
                "generation={} configured_roots={}",
                input.generation,
                input.configured_library_roots.len(),
            ),
        );
    }

    let ignore_setup_started_at = Instant::now();
    let _ = storage.ensure_default_ignore_rules();
    let _ = storage.reconcile_source_root_availability();
    startup_log::log_duration(
        "startup.snapshot_hydrate.root_reconcile",
        ignore_setup_started_at.elapsed(),
        &format!("generation={}", input.generation),
    );

    let roots_started_at = Instant::now();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();
    startup_log::log_duration(
        "startup.snapshot_hydrate.load_roots",
        roots_started_at.elapsed(),
        &format!("generation={} roots={}", input.generation, out.roots.len()),
    );

    let ignore_rules_started_at = Instant::now();
    out.ignore_rules = storage.list_ignore_rules("global").unwrap_or_default();
    startup_log::log_duration(
        "startup.snapshot_hydrate.load_ignore_rules",
        ignore_rules_started_at.elapsed(),
        &format!(
            "generation={} ignore_rules={}",
            input.generation,
            out.ignore_rules.len(),
        ),
    );

    let snapshot_load_started_at = Instant::now();
    let snapshot_json = storage
        .load_projection_snapshot(PROJECTION_SNAPSHOT_KEY)
        .ok()
        .flatten();
    startup_log::log_duration(
        "startup.snapshot_hydrate.load_snapshot_payload",
        snapshot_load_started_at.elapsed(),
        &format!(
            "generation={} present={}",
            input.generation,
            snapshot_json.is_some(),
        ),
    );

    if let Some(json) = snapshot_json {
        out.snapshot_bytes = json.len();
        out.snapshot_version = extract_snapshot_version(&json);
        match out.snapshot_version {
            Some(PROJECTION_SNAPSHOT_VERSION) => {
                let parse_started_at = Instant::now();
                match serde_json::from_str::<PersistedProjectionSnapshot>(&json) {
                    Ok(snapshot) => {
                        startup_log::log_duration(
                            "startup.snapshot_hydrate.parse_snapshot",
                            parse_started_at.elapsed(),
                            &format!(
                                "generation={} bytes={} gallery_items={} gallery_total_items={}",
                                input.generation,
                                out.snapshot_bytes,
                                snapshot.gallery_items.len(),
                                snapshot.gallery_total_items,
                            ),
                        );
                        out.snapshot = Some(snapshot);
                    }
                    Err(error) => {
                        out.snapshot_error = Some(format!("snapshot parse failed: {error}"));
                        startup_log::log_error(
                            "startup.snapshot_hydrate.parse_snapshot.failed",
                            &format!(
                                "generation={} bytes={} error={error}",
                                input.generation, out.snapshot_bytes,
                            ),
                        );
                    }
                }
            }
            Some(version) => {
                startup_log::log_info(
                    "startup.snapshot_hydrate.snapshot_discarded",
                    &format!(
                        "generation={} bytes={} reason=version_mismatch expected={} actual={version}",
                        input.generation, out.snapshot_bytes, PROJECTION_SNAPSHOT_VERSION,
                    ),
                );
            }
            None => {
                out.snapshot_error = Some("snapshot version missing".to_owned());
                startup_log::log_error(
                    "startup.snapshot_hydrate.snapshot_discarded",
                    &format!(
                        "generation={} bytes={} reason=version_missing",
                        input.generation, out.snapshot_bytes,
                    ),
                );
            }
        }
    }

    startup_log::log_duration(
        "startup.snapshot_hydrate.complete",
        hydrate_started_at.elapsed(),
        &format!(
            "generation={} roots={} ignore_rules={} snapshot_loaded={} snapshot_bytes={}",
            input.generation,
            out.roots.len(),
            out.ignore_rules.len(),
            out.snapshot.is_some(),
            out.snapshot_bytes,
        ),
    );

    out
}

fn apply_snapshot_hydrate_result(
    app: &mut Librapix,
    result: SnapshotHydrateResult,
) -> Task<Message> {
    if result.generation != app.background.snapshot_generation {
        return Task::none();
    }

    if let Some(started_at) = app.startup_metrics.snapshot_hydrate_started_at.take() {
        startup_log::log_duration(
            "startup.snapshot_hydrate.end",
            started_at.elapsed(),
            &format!(
                "generation={} roots={} ignore_rules={} snapshot_loaded={} snapshot_bytes={} snapshot_version={:?}",
                result.generation,
                result.roots.len(),
                result.ignore_rules.len(),
                result.snapshot.is_some(),
                result.snapshot_bytes,
                result.snapshot_version,
            ),
        );
    }

    app.background.snapshot_apply = None;
    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);
    ensure_valid_library_filter(app);
    app.ignore_rules = result.ignore_rules;
    app.background.snapshot_loaded = true;
    app.activity_progress.last_error = result.snapshot_error;
    app.background.startup_reconcile_queued = !app.state.library_roots.is_empty();

    if let Some(snapshot) = result.snapshot {
        return begin_snapshot_apply(app, result.generation, snapshot);
    }

    continue_startup_after_snapshot_hydrate(app)
}

fn begin_snapshot_apply(
    app: &mut Librapix,
    generation: u64,
    snapshot: PersistedProjectionSnapshot,
) -> Task<Message> {
    let gallery_total = snapshot.gallery_items.len();
    let total_items = gallery_total;

    app.gallery_items.clear();
    app.timeline_items.clear();
    app.timeline_anchors.clear();
    app.search_items.clear();
    app.media_cache.clear();
    app.available_filter_tags.clear();
    invalidate_browse_layout_cache(app, "snapshot_apply");
    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state.replace_gallery_preview(Vec::new());
    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state.replace_timeline_preview(Vec::new());
    app.state.apply(AppMessage::ReplaceSearchPreview);
    app.state.replace_search_preview(Vec::new());

    set_activity_stage(
        app,
        TextKey::StageLoadingSnapshotLabel,
        format!(
            "Restoring {} recent gallery items ({} available in snapshot)",
            gallery_total, snapshot.gallery_total_items,
        ),
        false,
    );
    app.activity_progress.items_total = Some(total_items);
    app.activity_progress.items_done = 0;
    app.activity_progress.queue_depth = 0;
    app.startup_metrics.snapshot_apply_started_at = Some(Instant::now());
    startup_log::log_info(
        "startup.snapshot_apply.start",
        &format!(
            "generation={generation} gallery_slice_items={} gallery_total_items={}",
            gallery_total, snapshot.gallery_total_items,
        ),
    );

    app.background.snapshot_apply = Some(PendingSnapshotApply {
        generation,
        gallery_total,
        gallery_total_items: snapshot.gallery_total_items,
        gallery_loaded: 0,
        gallery_iter: snapshot.gallery_items.into_iter(),
        available_filter_tags: snapshot.available_filter_tags,
    });

    Task::done(Message::SnapshotApplyTick)
}

fn apply_snapshot_chunk(app: &mut Librapix) -> Task<Message> {
    let Some(mut pending) = app.background.snapshot_apply.take() else {
        return Task::none();
    };
    if pending.generation != app.background.snapshot_generation {
        return Task::none();
    }

    for _ in 0..SNAPSHOT_APPLY_CHUNK_SIZE {
        let Some(item) = pending.gallery_iter.next() else {
            break;
        };
        pending.gallery_loaded = pending.gallery_loaded.saturating_add(1);
        app.gallery_items.push(item);
    }

    let total = pending.gallery_total;
    let done = pending.gallery_loaded;
    app.activity_progress.items_total = Some(total);
    app.activity_progress.items_done = done;
    app.activity_progress.queue_depth = 0;

    if done < total {
        app.background.snapshot_apply = Some(pending);
        return Task::none();
    }

    app.available_filter_tags = pending.available_filter_tags;
    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state
        .replace_gallery_preview(collect_preview_lines(&app.gallery_items));
    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state
        .replace_timeline_preview(collect_preview_lines(&app.timeline_items));
    app.browse_status = app.i18n.text(TextKey::GalleryCompletedLabel).to_owned();
    note_first_usable_gallery(app, "snapshot");
    if let Some(started_at) = app.startup_metrics.snapshot_apply_started_at.take() {
        startup_log::log_duration(
            "startup.snapshot_apply.end",
            started_at.elapsed(),
            &format!(
                "generation={} gallery_slice_items={} gallery_total_items={}",
                pending.generation, pending.gallery_total, pending.gallery_total_items,
            ),
        );
    }

    continue_startup_after_snapshot_hydrate(app)
}

fn schedule_startup_reconcile(app: &mut Librapix) -> Task<Message> {
    app.background.startup_reconcile_due_at =
        Some(Instant::now() + Duration::from_millis(STARTUP_RECONCILE_DELAY_MS));
    Task::none()
}

fn startup_blocking_work_complete(app: &Librapix) -> bool {
    app.background.snapshot_apply.is_none()
        && !app.background.pending_reconcile
        && !app.background.pending_projection
        && !app.background.reconcile_in_flight
        && !app.background.projection_in_flight
}

fn should_skip_startup_projection(app: &Librapix, summary: IndexingSummary) -> bool {
    if app.background.startup_ready
        || !app.background.snapshot_loaded
        || app.gallery_items.is_empty()
        || !app.state.search_query.trim().is_empty()
        || !matches!(app.state.active_route, Route::Gallery)
        || app.filter_media_kind.is_some()
        || app.filter_extension.is_some()
        || app.filter_tag.is_some()
        || app.filter_source_root_id.is_some()
    {
        return false;
    }

    summary.new_files == 0 && summary.changed_files == 0 && summary.missing_marked == 0
}

fn schedule_startup_gallery_continuation(app: &mut Librapix, total_items: usize) {
    if total_items <= app.gallery_items.len() {
        startup_log::log_info(
            "startup.gallery_continuation.skipped",
            &format!(
                "reason=already_complete rendered_items={} total_items={}",
                app.gallery_items.len(),
                total_items,
            ),
        );
        return;
    }
    if app.background.startup_gallery_continuation_due_at.is_some() {
        startup_log::log_info(
            "startup.gallery_continuation.skipped",
            &format!(
                "reason=already_scheduled rendered_items={} total_items={}",
                app.gallery_items.len(),
                total_items,
            ),
        );
        return;
    }

    app.background.startup_deferred_gallery_refresh = true;
    app.background.startup_gallery_continuation_due_at =
        Some(Instant::now() + Duration::from_millis(STARTUP_GALLERY_CONTINUATION_DELAY_MS));
    startup_log::log_info(
        "startup.gallery_continuation.scheduled",
        &format!(
            "rendered_items={} total_items={} due_in_ms={}",
            app.gallery_items.len(),
            total_items,
            STARTUP_GALLERY_CONTINUATION_DELAY_MS,
        ),
    );
}

fn start_startup_gallery_continuation(app: &mut Librapix) -> Task<Message> {
    let Some(due_at) = app.background.startup_gallery_continuation_due_at else {
        return Task::none();
    };
    if Instant::now() < due_at {
        return Task::none();
    }

    app.background.startup_gallery_continuation_due_at = None;
    if !matches!(app.state.active_route, Route::Gallery) {
        startup_log::log_info(
            "startup.gallery_continuation.skipped",
            &format!(
                "reason=route_not_gallery route={}",
                route_name(app.state.active_route)
            ),
        );
        return Task::none();
    }
    if !app.state.search_query.trim().is_empty()
        || app.filter_media_kind.is_some()
        || app.filter_extension.is_some()
        || app.filter_tag.is_some()
        || app.filter_source_root_id.is_some()
    {
        startup_log::log_info(
            "startup.gallery_continuation.skipped",
            &format!(
                "reason=non_default_gallery route={} {}",
                route_name(app.state.active_route),
                filter_state_summary(app),
            ),
        );
        return Task::none();
    }

    startup_log::log_info(
        "startup.gallery_continuation.kickoff",
        &format!(
            "route={} rendered_items={} {}",
            route_name(app.state.active_route),
            app.gallery_items.len(),
            filter_state_summary(app),
        ),
    );
    request_projection_refresh_with_context(
        app,
        BackgroundWorkReason::UserOrSystem,
        "startup_continuation",
    )
}

fn all_background_work_idle(app: &Librapix) -> bool {
    startup_blocking_work_complete(app)
        && !app.background.thumbnail_in_flight
        && app.background.thumbnail_queue.is_empty()
        && app.background.deferred_thumbnail_queue.is_empty()
        && app.background.deferred_thumbnail_due_at.is_none()
}

fn thumbnail_work_active(app: &Librapix) -> bool {
    app.background.thumbnail_in_flight
        || !app.background.thumbnail_queue.is_empty()
        || !app.background.deferred_thumbnail_queue.is_empty()
        || app.background.deferred_thumbnail_due_at.is_some()
}

fn note_thumbnail_refresh_pressure(app: &mut Librapix, reason: &str) {
    if !thumbnail_work_active(app) {
        return;
    }

    app.background.thumbnail_refresh_requests_while_active = app
        .background
        .thumbnail_refresh_requests_while_active
        .saturating_add(1);
    startup_log::log_info(
        "startup.thumbnail.refresh_while_active",
        &format!(
            "reason={reason} count={}",
            app.background.thumbnail_refresh_requests_while_active
        ),
    );
}

fn cancel_thumbnail_work(app: &mut Librapix, reason: &str) {
    let had_work = thumbnail_work_active(app);
    if !had_work {
        return;
    }

    app.background.thumbnail_generation = app.background.thumbnail_generation.saturating_add(1);
    app.background.thumbnail_cancel_generation.store(
        app.background.thumbnail_generation,
        std::sync::atomic::Ordering::Relaxed,
    );
    app.background.thumbnail_in_flight = false;
    app.background.deferred_thumbnail_due_at = None;
    app.background.thumbnail_queue.clear();
    app.background.thumbnail_queued_ids.clear();
    app.background.deferred_thumbnail_queue.clear();
    app.background.thumbnail_done = 0;
    app.background.thumbnail_total = 0;
    app.background.thumbnail_generated = 0;
    app.background.thumbnail_reused_exact = 0;
    app.background.thumbnail_reused_fallback = 0;
    app.background.thumbnail_failed = 0;
    app.background.thumbnail_mode = ThumbnailWorkMode::StartupPriority;
    app.activity_progress.items_done = 0;
    app.activity_progress.items_total = None;
    app.activity_progress.queue_depth = 0;
    app.thumbnail_status = thumbnail_status_text(app.i18n, 0, 0, 0);
    startup_log::log_info(
        "startup.thumbnail.cancelled",
        &format!(
            "{reason} queued={} failed={} refresh_requests_while_active={}",
            app.background.thumbnail_total,
            app.background.thumbnail_failed,
            app.background.thumbnail_refresh_requests_while_active,
        ),
    );
}

fn continue_startup_after_snapshot_hydrate(app: &mut Librapix) -> Task<Message> {
    if app.background.pending_reconcile {
        let reason = app.background.pending_reconcile_reason;
        app.background.pending_reconcile = false;
        app.background.startup_reconcile_queued = false;
        return request_reconcile(app, reason);
    }

    if app.background.startup_reconcile_queued {
        app.background.startup_reconcile_queued = false;
        return schedule_startup_reconcile(app);
    }

    if app.background.pending_projection {
        let reason = app.background.pending_projection_reason;
        app.background.pending_projection = false;
        return request_projection_refresh(app, reason);
    }

    if startup_blocking_work_complete(app) {
        mark_startup_ready(app);
    }

    Task::none()
}

fn request_reconcile(app: &mut Librapix, reason: BackgroundWorkReason) -> Task<Message> {
    if app.background.snapshot_apply.is_some()
        || (!app.background.snapshot_loaded && app.background.snapshot_generation > 0)
        || app.background.reconcile_in_flight
        || app.background.projection_in_flight
    {
        startup_log::log_info(
            "startup.reconcile.request.queued",
            &format!(
                "reason={reason:?} snapshot_apply={} snapshot_loaded={} snapshot_generation={} reconcile_in_flight={} projection_in_flight={} pending_reconcile={} pending_projection={}",
                app.background.snapshot_apply.is_some(),
                app.background.snapshot_loaded,
                app.background.snapshot_generation,
                app.background.reconcile_in_flight,
                app.background.projection_in_flight,
                app.background.pending_reconcile,
                app.background.pending_projection,
            ),
        );
        app.background.pending_reconcile_reason = if app.background.pending_reconcile {
            merge_work_reason(app.background.pending_reconcile_reason, reason)
        } else {
            reason
        };
        app.background.pending_reconcile = true;
        return Task::none();
    }

    app.background.startup_reconcile_due_at = None;
    app.background.startup_reconcile_queued = false;
    app.background.reconcile_in_flight = true;
    app.background.reconcile_generation = app.background.reconcile_generation.saturating_add(1);
    app.background.pending_reconcile = false;
    app.background.pending_reconcile_reason = reason;
    app.background.pending_projection = false;
    note_thumbnail_refresh_pressure(app, "reconcile");
    cancel_thumbnail_work(app, "reason=reconcile");
    app.startup_metrics.reconcile_started_at = Some(Instant::now());

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
    app.activity_progress.queue_depth = 0;
    app.activity_progress.last_error = None;
    app.indexing_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
    startup_log::log_info(
        "startup.reconcile.start",
        &format!(
            "generation={} reason={:?} roots={}",
            app.background.reconcile_generation,
            reason,
            app.state.library_roots.len(),
        ),
    );

    let input = ScanJobInput {
        generation: app.background.reconcile_generation,
        reason,
        database_file: app.runtime.database_file.clone(),
        min_file_size_bytes: app.min_file_size_bytes,
        i18n: app.i18n,
    };
    Task::perform(async move { do_scan_job(input) }, |result| {
        Message::ScanJobComplete(Box::new(result))
    })
}

fn do_scan_job(input: ScanJobInput) -> ScanJobResult {
    let scan_started_at = Instant::now();
    let mut out = ScanJobResult {
        generation: input.generation,
        reason: input.reason,
        ..Default::default()
    };

    let (mut storage, storage_metrics) = match Storage::open_with_metrics(&input.database_file) {
        Ok(value) => value,
        Err(error) => {
            out.error = Some(error.to_string());
            out.indexing_status = input
                .i18n
                .text(TextKey::ErrorIndexingFailedLabel)
                .to_owned();
            startup_log::log_error(
                "startup.reconcile.storage_open.failed",
                &format!(
                    "generation={} path={} error={error}",
                    input.generation,
                    input.database_file.display(),
                ),
            );
            return out;
        }
    };
    log_storage_open_metrics("scan_job", &input.database_file, &storage_metrics);

    let root_reconcile_started_at = Instant::now();
    let _ = storage.reconcile_source_root_availability();
    let _ = storage.ensure_default_ignore_rules();
    startup_log::log_duration(
        "startup.reconcile.root_reconcile",
        root_reconcile_started_at.elapsed(),
        &format!("generation={}", input.generation),
    );

    let ignore_rules_started_at = Instant::now();
    out.ignore_rules = storage.list_ignore_rules("global").unwrap_or_default();
    startup_log::log_duration(
        "startup.reconcile.load_ignore_rules",
        ignore_rules_started_at.elapsed(),
        &format!(
            "generation={} ignore_rules={}",
            input.generation,
            out.ignore_rules.len(),
        ),
    );

    let roots_started_at = Instant::now();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();
    startup_log::log_duration(
        "startup.reconcile.load_roots",
        roots_started_at.elapsed(),
        &format!("generation={} roots={}", input.generation, out.roots.len()),
    );

    let eligible_roots_started_at = Instant::now();
    let eligible_roots = storage.list_eligible_source_roots().unwrap_or_default();
    out.root_count = eligible_roots.len();
    startup_log::log_duration(
        "startup.reconcile.load_eligible_roots",
        eligible_roots_started_at.elapsed(),
        &format!(
            "generation={} eligible_roots={}",
            input.generation, out.root_count
        ),
    );
    let roots_for_scan = eligible_roots
        .iter()
        .map(|root| ScanRoot {
            source_root_id: root.id,
            normalized_path: root.normalized_path.clone(),
        })
        .collect::<Vec<_>>();

    let ignore_pattern_started_at = Instant::now();
    let patterns = storage
        .list_enabled_ignore_patterns("global")
        .unwrap_or_default();
    startup_log::log_duration(
        "startup.reconcile.load_ignore_patterns",
        ignore_pattern_started_at.elapsed(),
        &format!(
            "generation={} patterns={}",
            input.generation,
            patterns.len(),
        ),
    );
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

    let root_ids = roots_for_scan
        .iter()
        .map(|root| root.source_root_id)
        .collect::<Vec<_>>();
    let existing_started_at = Instant::now();
    let existing = storage
        .list_existing_indexed_media_snapshots(&root_ids)
        .unwrap_or_default();
    startup_log::log_duration(
        "startup.reconcile.load_existing_media_snapshots",
        existing_started_at.elapsed(),
        &format!(
            "generation={} roots={} existing_entries={}",
            input.generation,
            root_ids.len(),
            existing.len(),
        ),
    );
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
    let scan_roots_started_at = Instant::now();
    let scan_result = scan_roots(
        &roots_for_scan,
        &ignore,
        &existing_for_indexer,
        &ScanOptions {
            min_file_size_bytes: input.min_file_size_bytes,
        },
    );
    startup_log::log_duration(
        "startup.reconcile.scan_roots",
        scan_roots_started_at.elapsed(),
        &format!(
            "generation={} scanned_roots={} candidates={} ignored={} unreadable={} changed={} new={} unchanged={}",
            input.generation,
            scan_result.summary.scanned_roots,
            scan_result.summary.candidate_files,
            scan_result.summary.ignored_entries,
            scan_result.summary.unreadable_entries,
            scan_result.summary.changed_files,
            scan_result.summary.new_files,
            scan_result.summary.unchanged_files,
        ),
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

    let apply_index_started_at = Instant::now();
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
    startup_log::log_duration(
        "startup.reconcile.apply_index",
        apply_index_started_at.elapsed(),
        &format!(
            "generation={} writes={} missing_marked={}",
            input.generation,
            writes.len(),
            apply_summary.missing_marked_count,
        ),
    );

    let post_index_started_at = Instant::now();
    let _ = storage.ensure_media_kind_tags_attached();
    let _ = storage.ensure_root_tags_exist();
    let _ = storage.apply_root_auto_tags();
    let _ = storage.refresh_source_root_statistics(&scan_result.scanned_root_ids);
    startup_log::log_duration(
        "startup.reconcile.post_index_maintenance",
        post_index_started_at.elapsed(),
        &format!(
            "generation={} scanned_root_ids={}",
            input.generation,
            scan_result.scanned_root_ids.len(),
        ),
    );

    let count_started_at = Instant::now();
    let read_model_count = storage.count_indexed_media().unwrap_or(-1).max(0) as usize;
    startup_log::log_duration(
        "startup.reconcile.count_indexed_media",
        count_started_at.elapsed(),
        &format!(
            "generation={} read_model_count={read_model_count}",
            input.generation,
        ),
    );
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
    out.indexing_status = input.i18n.text(TextKey::IndexingCompletedLabel).to_owned();
    startup_log::log_duration(
        "startup.reconcile.complete",
        scan_started_at.elapsed(),
        &format!(
            "generation={} roots={} read_model_count={read_model_count}",
            input.generation, out.root_count,
        ),
    );
    out
}

fn request_projection_refresh(app: &mut Librapix, reason: BackgroundWorkReason) -> Task<Message> {
    request_projection_refresh_with_context(app, reason, "system")
}

fn request_projection_refresh_with_context(
    app: &mut Librapix,
    reason: BackgroundWorkReason,
    trigger: &'static str,
) -> Task<Message> {
    if app.background.snapshot_apply.is_some()
        || (!app.background.snapshot_loaded && app.background.snapshot_generation > 0)
        || app.background.reconcile_in_flight
        || app.background.projection_in_flight
    {
        startup_log::log_info(
            "interaction.projection.request.deduped",
            &format!(
                "trigger={trigger} reason={reason:?} route={} snapshot_apply={} snapshot_loaded={} snapshot_generation={} reconcile_in_flight={} projection_in_flight={} pending_reconcile={} pending_projection={} gallery_items={} timeline_items={} {}",
                route_name(app.state.active_route),
                app.background.snapshot_apply.is_some(),
                app.background.snapshot_loaded,
                app.background.snapshot_generation,
                app.background.reconcile_in_flight,
                app.background.projection_in_flight,
                app.background.pending_reconcile,
                app.background.pending_projection,
                app.gallery_items.len(),
                app.timeline_items.len(),
                filter_state_summary(app),
            ),
        );
        startup_log::log_info(
            "startup.projection.request.queued",
            &format!(
                "reason={reason:?} snapshot_apply={} snapshot_loaded={} snapshot_generation={} reconcile_in_flight={} projection_in_flight={} pending_reconcile={} pending_projection={}",
                app.background.snapshot_apply.is_some(),
                app.background.snapshot_loaded,
                app.background.snapshot_generation,
                app.background.reconcile_in_flight,
                app.background.projection_in_flight,
                app.background.pending_reconcile,
                app.background.pending_projection,
            ),
        );
        app.background.pending_projection_reason = if app.background.pending_projection {
            merge_work_reason(app.background.pending_projection_reason, reason)
        } else {
            reason
        };
        app.background.pending_projection = true;
        return Task::none();
    }

    startup_log::log_info(
        "interaction.projection.request.accepted",
        &format!(
            "trigger={trigger} reason={reason:?} route={} gallery_items={} timeline_items={} startup_ready={} {}",
            route_name(app.state.active_route),
            app.gallery_items.len(),
            app.timeline_items.len(),
            app.background.startup_ready,
            filter_state_summary(app),
        ),
    );
    start_projection_refresh(app, reason, trigger)
}

fn start_projection_refresh(
    app: &mut Librapix,
    reason: BackgroundWorkReason,
    trigger: &'static str,
) -> Task<Message> {
    note_thumbnail_refresh_pressure(app, "projection_refresh");
    cancel_thumbnail_work(app, "reason=projection_refresh");
    let policy = projection_refresh_policy(app, reason, trigger);
    app.background.projection_in_flight = true;
    app.background.pending_projection = false;
    app.background.pending_projection_reason = reason;
    app.background.projection_generation = app.background.projection_generation.saturating_add(1);
    app.startup_metrics.projection_started_at = Some(Instant::now());
    if matches!(policy, ProjectionRefreshPolicy::CurrentSurface) {
        app.background.startup_deferred_gallery_refresh =
            !matches!(app.state.active_route, Route::Gallery);
        app.background.startup_deferred_timeline_refresh =
            !matches!(app.state.active_route, Route::Timeline);
    } else {
        app.background.startup_deferred_gallery_refresh = false;
        app.background.startup_deferred_timeline_refresh = false;
    }
    set_activity_stage(
        app,
        activity_stage_key(&app.state.search_query, app.state.active_route),
        projection_loading_label(app),
        true,
    );
    app.activity_progress.roots_done = 0;
    app.activity_progress.roots_total = None;
    app.activity_progress.items_done = 0;
    app.activity_progress.items_total = None;
    app.activity_progress.queue_depth = 0;
    app.activity_progress.last_error = None;
    app.browse_status = projection_loading_label(app).to_owned();
    startup_log::log_info(
        "startup.gallery_working.set",
        &format!(
            "owner=projection generation={} route={:?} browse_status={} startup_ready={} reason={reason:?}",
            app.background.projection_generation,
            app.state.active_route,
            app.browse_status,
            app.background.startup_ready,
        ),
    );
    startup_log::log_info(
        "interaction.route_working.set",
        &format!(
            "owner=projection trigger={trigger} route={} generation={} reason={reason:?} browse_status={} startup_ready={}",
            route_name(app.state.active_route),
            app.background.projection_generation,
            app.browse_status,
            app.background.startup_ready,
        ),
    );
    startup_log::log_info(
        "startup.projection.start",
        &format!(
            "generation={} reason={:?} policy={policy:?} route={:?} search_len={} filter_kind={:?} filter_extension={:?} filter_tag={:?} filter_root_id={:?}",
            app.background.projection_generation,
            reason,
            app.state.active_route,
            app.state.search_query.trim().len(),
            app.filter_media_kind,
            app.filter_extension,
            app.filter_tag,
            app.filter_source_root_id,
        ),
    );
    startup_log::log_info(
        "interaction.projection.start",
        &format!(
            "trigger={trigger} generation={} route={} reason={reason:?} policy={policy:?} gallery_items={} timeline_items={} search_items={} {}",
            app.background.projection_generation,
            route_name(app.state.active_route),
            app.gallery_items.len(),
            app.timeline_items.len(),
            app.search_items.len(),
            filter_state_summary(app),
        ),
    );

    let input = ProjectionJobInput {
        generation: app.background.projection_generation,
        reason,
        policy,
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
    let projection_started_at = Instant::now();
    let mut out = ProjectionJobResult {
        generation: input.generation,
        reason: input.reason,
        ..Default::default()
    };

    let (mut storage, storage_metrics) = match Storage::open_with_metrics(&input.database_file) {
        Ok(value) => value,
        Err(error) => {
            out.error = Some(error.to_string());
            startup_log::log_error(
                "startup.projection.storage_open.failed",
                &format!(
                    "generation={} path={} error={error}",
                    input.generation,
                    input.database_file.display(),
                ),
            );
            return out;
        }
    };
    log_storage_open_metrics("projection_job", &input.database_file, &storage_metrics);

    let preflight_started_at = Instant::now();
    let _ = storage.reconcile_source_root_availability();
    let _ = storage.ensure_default_ignore_rules();
    startup_log::log_duration(
        "startup.projection.preflight",
        preflight_started_at.elapsed(),
        &format!("generation={}", input.generation),
    );

    let ignore_rules_started_at = Instant::now();
    out.ignore_rules = storage.list_ignore_rules("global").unwrap_or_default();
    startup_log::log_duration(
        "startup.projection.load_ignore_rules",
        ignore_rules_started_at.elapsed(),
        &format!(
            "generation={} ignore_rules={}",
            input.generation,
            out.ignore_rules.len(),
        ),
    );

    let roots_started_at = Instant::now();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();
    startup_log::log_duration(
        "startup.projection.load_roots",
        roots_started_at.elapsed(),
        &format!("generation={} roots={}", input.generation, out.roots.len()),
    );

    let refresh_catalog_started_at = Instant::now();
    if let Err(error) = storage.refresh_catalog() {
        out.error = Some(error.to_string());
        startup_log::log_error(
            "startup.projection.refresh_catalog.failed",
            &format!("generation={} error={error}", input.generation),
        );
        return out;
    }
    startup_log::log_duration(
        "startup.projection.refresh_catalog",
        refresh_catalog_started_at.elapsed(),
        &format!("generation={}", input.generation),
    );

    let list_rows_started_at = Instant::now();
    let all_rows = match storage.list_catalog_media_filtered(input.filter_source_root_id) {
        Ok(rows) => rows,
        Err(error) => {
            out.error = Some(error.to_string());
            startup_log::log_error(
                "startup.projection.list_catalog_rows.failed",
                &format!("generation={} error={error}", input.generation),
            );
            return out;
        }
    };
    startup_log::log_duration(
        "startup.projection.list_catalog_rows",
        list_rows_started_at.elapsed(),
        &format!(
            "generation={} rows={} filter_root_id={:?}",
            input.generation,
            all_rows.len(),
            input.filter_source_root_id,
        ),
    );

    let available_tags_started_at = Instant::now();
    out.available_filter_tags = collect_available_filter_tags(&all_rows);
    startup_log::log_duration(
        "startup.projection.collect_available_tags",
        available_tags_started_at.elapsed(),
        &format!(
            "generation={} tags={}",
            input.generation,
            out.available_filter_tags.len(),
        ),
    );
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
    let refresh_gallery = matches!(input.policy, ProjectionRefreshPolicy::Full)
        || matches!(input.active_route, Route::Gallery);
    let refresh_timeline = matches!(input.policy, ProjectionRefreshPolicy::Full)
        || matches!(input.active_route, Route::Timeline);
    out.refreshed_gallery = refresh_gallery;
    out.refreshed_timeline = refresh_timeline;
    let media_ids = all_rows.iter().map(|row| row.media_id).collect::<Vec<_>>();
    let gallery_artifacts_started_at = Instant::now();
    let gallery_artifacts = match storage.list_ready_derived_artifacts_for_media_ids(
        &media_ids,
        DerivedArtifactKind::Thumbnail,
        GALLERY_THUMB_VARIANT,
    ) {
        Ok(artifacts) => artifacts,
        Err(error) => {
            out.error = Some(error.to_string());
            startup_log::log_error(
                "startup.projection.list_gallery_artifacts.failed",
                &format!("generation={} error={error}", input.generation),
            );
            return out;
        }
    };
    startup_log::log_duration(
        "startup.projection.list_gallery_artifacts",
        gallery_artifacts_started_at.elapsed(),
        &format!(
            "generation={} media_ids={} ready_artifacts={}",
            input.generation,
            media_ids.len(),
            gallery_artifacts.len(),
        ),
    );
    let detail_artifacts_started_at = Instant::now();
    let detail_artifacts = match storage.list_ready_derived_artifacts_for_media_ids(
        &media_ids,
        DerivedArtifactKind::Thumbnail,
        DETAIL_THUMB_VARIANT,
    ) {
        Ok(artifacts) => artifacts,
        Err(error) => {
            out.error = Some(error.to_string());
            startup_log::log_error(
                "startup.projection.list_detail_artifacts.failed",
                &format!("generation={} error={error}", input.generation),
            );
            return out;
        }
    };
    startup_log::log_duration(
        "startup.projection.list_detail_artifacts",
        detail_artifacts_started_at.elapsed(),
        &format!(
            "generation={} media_ids={} ready_artifacts={}",
            input.generation,
            media_ids.len(),
            detail_artifacts.len(),
        ),
    );

    if refresh_gallery {
        let gallery_started_at = Instant::now();
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
                        let original = PathBuf::from(item.absolute_path);
                        BrowseItem {
                            media_id: item.media_id,
                            title: original
                                .file_name()
                                .map(|name| name.to_string_lossy().to_string())
                                .unwrap_or_else(|| original.display().to_string()),
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
                            line: format!("{} [{}]", original.display(), item.media_kind),
                            aspect_ratio: 1.5,
                            group_image_count: None,
                            group_video_count: None,
                        }
                    },
                    |row| browse_item_from_catalog_row(input.i18n, row, None),
                )
            })
            .collect();
        out.gallery_preview_lines = collect_preview_lines(&out.gallery_items);
        startup_log::log_duration(
            "startup.projection.project_gallery",
            gallery_started_at.elapsed(),
            &format!(
                "generation={} items={}",
                input.generation,
                out.gallery_items.len(),
            ),
        );
    }

    if refresh_timeline {
        let timeline_started_at = Instant::now();
        let filtered_media = media
            .iter()
            .filter(|item| {
                input
                    .filter_media_kind
                    .as_ref()
                    .is_none_or(|kind| item.media_kind.eq_ignore_ascii_case(kind))
            })
            .filter(|item| {
                input.filter_extension.as_ref().is_none_or(|extension| {
                    item.absolute_path
                        .rsplit('.')
                        .next()
                        .is_some_and(|existing| existing.eq_ignore_ascii_case(extension))
                })
            })
            .filter(|item| {
                projection_matches_tag_filter(item, active_tag_filter.map(|tag| tag.as_str()))
            })
            .cloned()
            .collect::<Vec<_>>();

        let buckets = project_timeline(&filtered_media, TimelineGranularity::Day);
        out.timeline_anchors = build_timeline_anchors(&buckets);
        let mut timeline_items = Vec::new();
        let mut timeline_preview_lines = Vec::new();
        for bucket in buckets {
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
            timeline_preview_lines.push(format!("{} ({})", bucket.label, bucket.item_count));
            timeline_items.push(BrowseItem {
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
            });

            for item in bucket.items {
                timeline_items.push(row_lookup.get(&item.media_id).copied().map_or_else(
                    || {
                        BrowseItem {
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
                        }
                    },
                    |row| {
                        let mut browse_item = browse_item_from_catalog_row(input.i18n, row, None);
                        browse_item.line = format!("{} [{}]", item.absolute_path, item.media_kind);
                        browse_item
                    },
                ));
            }
        }
        out.timeline_items = timeline_items;
        out.timeline_preview_lines = timeline_preview_lines;
        startup_log::log_duration(
            "startup.projection.project_timeline",
            timeline_started_at.elapsed(),
            &format!(
                "generation={} items={} anchors={}",
                input.generation,
                out.timeline_items.len(),
                out.timeline_anchors.len(),
            ),
        );
    }

    if !input.search_query.trim().is_empty() {
        let search_started_at = Instant::now();
        let docs = all_rows
            .iter()
            .map(|row| SearchDocument {
                media_id: row.media_id,
                absolute_path: row.absolute_path.display().to_string(),
                file_name: row.file_name.clone(),
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
            .filter_map(|hit| row_lookup.get(&hit.media_id).copied().map(|row| (hit, row)))
            .filter(|(_, row)| {
                input
                    .filter_media_kind
                    .as_ref()
                    .is_none_or(|kind| row.media_kind.eq_ignore_ascii_case(kind))
            })
            .filter(|(_, row)| {
                input.filter_extension.as_ref().is_none_or(|extension| {
                    row.absolute_path
                        .extension()
                        .and_then(|existing| existing.to_str())
                        .is_some_and(|existing| existing.eq_ignore_ascii_case(extension))
                })
            })
            .filter(|(_, row)| {
                catalog_row_matches_tag_filter(row, active_tag_filter.map(|tag| tag.as_str()))
            })
            .map(|(_, row)| browse_item_from_catalog_row(input.i18n, row, None))
            .collect();
        out.search_preview_lines = collect_preview_lines(&out.search_items);
        startup_log::log_duration(
            "startup.projection.project_search",
            search_started_at.elapsed(),
            &format!(
                "generation={} items={} query_len={}",
                input.generation,
                out.search_items.len(),
                input.search_query.trim().len(),
            ),
        );
    }

    let cache_media_ids = if matches!(input.policy, ProjectionRefreshPolicy::Full) {
        media_ids.clone()
    } else {
        let mut ordered = Vec::new();
        let mut seen = HashSet::new();
        if !out.search_items.is_empty() {
            collect_priority_media_ids(
                &out.search_items,
                &mut seen,
                &mut ordered,
                STARTUP_CACHE_WARM_LIMIT,
            );
        }
        let browse_items = if refresh_timeline {
            &out.timeline_items
        } else {
            &out.gallery_items
        };
        if ordered.len() < STARTUP_CACHE_WARM_LIMIT {
            collect_priority_media_ids(
                browse_items,
                &mut seen,
                &mut ordered,
                STARTUP_CACHE_WARM_LIMIT,
            );
        }
        ordered
    };

    let thumbnail_lookup = resolve_projection_thumbnail_lookup(ProjectionThumbnailLookupInput {
        generation: input.generation,
        all_rows: &all_rows,
        row_lookup: &row_lookup,
        gallery_artifacts: &gallery_artifacts,
        detail_artifacts: &detail_artifacts,
        thumbnails_dir: &input.thumbnails_dir,
        active_route: input.active_route,
        search_query: &input.search_query,
        gallery_items: &out.gallery_items,
        timeline_items: &out.timeline_items,
        search_items: &out.search_items,
    });
    patch_thumbnail_paths(&mut out.gallery_items, &thumbnail_lookup.resolved_paths);
    patch_thumbnail_paths(&mut out.timeline_items, &thumbnail_lookup.resolved_paths);
    patch_thumbnail_paths(&mut out.search_items, &thumbnail_lookup.resolved_paths);

    let media_cache_started_at = Instant::now();
    populate_media_cache(
        &storage,
        &mut out.media_cache,
        &row_lookup,
        &cache_media_ids,
        &input.thumbnails_dir,
    );
    startup_log::log_duration(
        "startup.projection.populate_media_cache",
        media_cache_started_at.elapsed(),
        &format!(
            "generation={} cached_media={} requested_media={}",
            input.generation,
            out.media_cache.len(),
            cache_media_ids.len(),
        ),
    );

    let thumbnail_candidates_started_at = Instant::now();
    out.thumbnail_candidates = all_rows
        .iter()
        .filter(|row| !thumbnail_lookup.reusable_media_ids.contains(&row.media_id))
        .filter(|row| {
            row.media_kind.eq_ignore_ascii_case("image")
                || row.media_kind.eq_ignore_ascii_case("video")
        })
        .map(|row| ThumbnailWorkItem {
            generation: input.generation,
            media_id: row.media_id,
            absolute_path: row.absolute_path.clone(),
            media_kind: row.media_kind.clone(),
            file_size_bytes: row.file_size_bytes,
            modified_unix_seconds: row.modified_unix_seconds,
        })
        .collect();
    startup_log::log_duration(
        "startup.projection.collect_thumbnail_candidates",
        thumbnail_candidates_started_at.elapsed(),
        &format!(
            "generation={} candidates={} priority_placeholders={} reused_exact={} reused_fallback={}",
            input.generation,
            out.thumbnail_candidates.len(),
            thumbnail_lookup.summary.priority_placeholder,
            thumbnail_lookup.summary.exact_catalog_reused
                + thumbnail_lookup.summary.exact_deterministic_reused,
            thumbnail_lookup.summary.fallback_catalog_reused
                + thumbnail_lookup.summary.fallback_deterministic_reused,
        ),
    );

    if refresh_gallery
        && input.search_query.trim().is_empty()
        && input.filter_media_kind.is_none()
        && input.filter_extension.is_none()
        && active_tag_filter.is_none()
        && input.filter_source_root_id.is_none()
    {
        out.snapshot_payload =
            snapshot_payload_from_projection(&out.gallery_items, &out.available_filter_tags);
    }
    out.browse_status = if input.search_query.trim().is_empty() {
        if matches!(input.active_route, Route::Timeline) {
            input.i18n.text(TextKey::TimelineCompletedLabel).to_owned()
        } else {
            input.i18n.text(TextKey::GalleryCompletedLabel).to_owned()
        }
    } else {
        input.i18n.text(TextKey::SearchCompletedLabel).to_owned()
    };
    startup_log::log_duration(
        "startup.projection.complete",
        projection_started_at.elapsed(),
        &format!(
            "generation={} refreshed_gallery={} refreshed_timeline={} gallery_items={} timeline_items={} search_items={} thumbnail_candidates={}",
            input.generation,
            out.refreshed_gallery,
            out.refreshed_timeline,
            out.gallery_items.len(),
            out.timeline_items.len(),
            out.search_items.len(),
            out.thumbnail_candidates.len(),
        ),
    );

    out
}

fn apply_scan_job_result(app: &mut Librapix, result: ScanJobResult) -> Task<Message> {
    if result.generation != app.background.reconcile_generation {
        return Task::none();
    }

    let indexing_summary = result.indexing_summary;
    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);
    ensure_valid_library_filter(app);
    app.ignore_rules = result.ignore_rules;
    app.indexing_status = result.indexing_status;
    app.activity_progress.roots_total = Some(result.root_count);
    app.activity_progress.roots_done = result.scanned_root_ids.len();
    if let Some(summary) = indexing_summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
    }
    if let Some(error) = result.error {
        app.activity_progress.last_error = Some(error);
        startup_log::log_error(
            "startup.reconcile.failed",
            &format!(
                "generation={} reason={:?} error={}",
                result.generation,
                result.reason,
                app.activity_progress
                    .last_error
                    .as_deref()
                    .unwrap_or_default()
            ),
        );
        app.background.reconcile_in_flight = false;
        if let Some(started_at) = app.startup_metrics.reconcile_started_at.take() {
            startup_log::log_duration(
                "startup.reconcile.end",
                started_at.elapsed(),
                &format!(
                    "generation={} success=false roots_scanned={} read_model_count={}",
                    result.generation,
                    result.scanned_root_ids.len(),
                    app.state.indexing_summary.read_model_count,
                ),
            );
        }
        return finalize_background_flow(app);
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

    app.background.reconcile_in_flight = false;
    if let Some(started_at) = app.startup_metrics.reconcile_started_at.take() {
        startup_log::log_duration(
            "startup.reconcile.end",
            started_at.elapsed(),
            &format!(
                "generation={} success=true roots_scanned={} read_model_count={}",
                result.generation,
                result.scanned_root_ids.len(),
                app.state.indexing_summary.read_model_count,
            ),
        );
    }
    if let Some(summary) = indexing_summary
        && should_skip_startup_projection(app, summary)
    {
        let read_model_count = summary.read_model_count;
        app.background.startup_deferred_timeline_refresh = true;
        schedule_startup_gallery_continuation(app, read_model_count);
        startup_log::log_info(
            "startup.projection.skipped",
            &format!(
                "reason=unchanged_snapshot_gallery generation={} route={:?} new_files={} changed_files={} missing_marked={} gallery_items={} gallery_total={} gallery_deferred={} timeline_deferred=true",
                result.generation,
                app.state.active_route,
                summary.new_files,
                summary.changed_files,
                summary.missing_marked,
                app.gallery_items.len(),
                read_model_count,
                app.background.startup_deferred_gallery_refresh,
            ),
        );
        return finalize_background_flow(app);
    }
    request_projection_refresh(app, result.reason)
}

fn apply_projection_job_result(app: &mut Librapix, result: ProjectionJobResult) -> Task<Message> {
    let apply_started_at = Instant::now();
    let previous_gallery_items = app.gallery_items.len();
    let previous_timeline_items = app.timeline_items.len();
    let previous_search_items = app.search_items.len();
    startup_log::log_info(
        "interaction.projection.message.received",
        &format!(
            "generation={} route={} refreshed_gallery={} refreshed_timeline={} previous_gallery_items={} previous_timeline_items={} previous_search_items={} {}",
            result.generation,
            route_name(app.state.active_route),
            result.refreshed_gallery,
            result.refreshed_timeline,
            previous_gallery_items,
            previous_timeline_items,
            previous_search_items,
            filter_state_summary(app),
        ),
    );
    if result.generation != app.background.projection_generation {
        startup_log::log_info(
            "interaction.projection.message.stale",
            &format!(
                "generation={} active_generation={} route={} {}",
                result.generation,
                app.background.projection_generation,
                route_name(app.state.active_route),
                filter_state_summary(app),
            ),
        );
        return Task::none();
    }
    app.background.projection_in_flight = false;

    if let Some(error) = result.error {
        app.activity_progress.last_error = Some(error);
        if let Some(started_at) = app.startup_metrics.projection_started_at.take() {
            startup_log::log_duration(
                "startup.projection.end",
                started_at.elapsed(),
                &format!("generation={} success=false", result.generation),
            );
        }
        startup_log::log_warn(
            "interaction.projection.message.failed",
            &format!(
                "generation={} route={} {}",
                result.generation,
                route_name(app.state.active_route),
                filter_state_summary(app),
            ),
        );
        return finalize_background_flow(app);
    }

    let previous_media_ids = app
        .gallery_items
        .iter()
        .chain(app.timeline_items.iter())
        .chain(app.search_items.iter())
        .filter(|item| !item.is_group_header && item.media_id > 0)
        .map(|item| item.media_id)
        .collect::<HashSet<_>>();
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
    invalidate_browse_layout_cache(app, "projection_apply");

    if result.refreshed_gallery {
        app.state.apply(AppMessage::ReplaceGalleryPreview);
        app.state
            .replace_gallery_preview(result.gallery_preview_lines);
        app.gallery_items = result.gallery_items;
        app.background.startup_deferred_gallery_refresh = false;
        app.background.startup_gallery_continuation_due_at = None;
        note_first_usable_gallery(app, "projection");
    }

    if result.refreshed_timeline {
        app.state.apply(AppMessage::ReplaceTimelinePreview);
        app.state
            .replace_timeline_preview(result.timeline_preview_lines);
        app.timeline_items = result.timeline_items;
        app.timeline_anchors = result.timeline_anchors;
        sync_timeline_scrub_selection(app, app.timeline_scrub_value);
        app.background.startup_deferred_timeline_refresh = false;
    }

    app.media_cache = result.media_cache;
    app.available_filter_tags = result.available_filter_tags;
    if app.filter_tag.as_ref().is_some_and(|tag| {
        !app.available_filter_tags
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(tag))
    }) {
        app.filter_tag = None;
    }
    app.browse_status = result.browse_status;
    reconcile_selected_media_after_projection(app);
    log_interaction_duration(
        "interaction.projection.apply",
        apply_started_at.elapsed(),
        &format!(
            "generation={} route={} refreshed_gallery={} refreshed_timeline={} gallery_items_before={} gallery_items_after={} timeline_items_before={} timeline_items_after={} search_items_before={} search_items_after={} deferred_gallery={} deferred_timeline={} {}",
            result.generation,
            route_name(app.state.active_route),
            result.refreshed_gallery,
            result.refreshed_timeline,
            previous_gallery_items,
            app.gallery_items.len(),
            previous_timeline_items,
            app.timeline_items.len(),
            previous_search_items,
            app.search_items.len(),
            app.background.startup_deferred_gallery_refresh,
            app.background.startup_deferred_timeline_refresh,
            filter_state_summary(app),
        ),
    );

    if let Some(mut announcement) = announcement {
        if announcement.preview_path.is_none() {
            announcement.preview_path = browse_thumbnail_path(app, announcement.media_id);
        }
        app.new_media_preview_loading_phase = 0;
        app.new_media_announcement = Some(announcement);
    }

    if let Some(payload) = result.snapshot_payload {
        let _ = with_storage(&app.runtime, |storage| {
            storage.upsert_projection_snapshot(PROJECTION_SNAPSHOT_KEY, &payload)
        });
        startup_log::log_info(
            "startup.snapshot.persisted",
            &format!("generation={} bytes={}", result.generation, payload.len()),
        );
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

    let thumbnail_candidates = filter_thumbnail_candidates_for_runtime_policy(
        app,
        result.thumbnail_candidates,
        "apply_projection_job_result",
    );
    let (immediate_thumbnails, deferred_thumbnails) =
        split_startup_thumbnail_work(app, thumbnail_candidates);
    app.background.deferred_thumbnail_queue =
        deferred_thumbnails.into_iter().collect::<VecDeque<_>>();
    startup_log::log_info(
        "startup.thumbnail_schedule",
        &format!(
            "generation={} startup_priority={} deferred={} startup_ready_before_schedule={}",
            result.generation,
            immediate_thumbnails.len(),
            app.background.deferred_thumbnail_queue.len(),
            app.background.startup_ready,
        ),
    );
    if let Some(started_at) = app.startup_metrics.projection_started_at.take() {
        startup_log::log_duration(
            "startup.projection.end",
            started_at.elapsed(),
            &format!(
                "generation={} success=true gallery_items={} timeline_items={} search_items={} startup_thumbnail_queue={} deferred_thumbnail_queue={}",
                result.generation,
                app.gallery_items.len(),
                app.timeline_items.len(),
                app.search_items.len(),
                immediate_thumbnails.len(),
                app.background.deferred_thumbnail_queue.len(),
            ),
        );
    }

    if !app.background.startup_ready && startup_blocking_work_complete(app) {
        mark_startup_ready(app);
    }

    if !immediate_thumbnails.is_empty() {
        return start_thumbnail_batches(
            app,
            immediate_thumbnails,
            ThumbnailWorkMode::StartupPriority,
        );
    }

    if !app.background.deferred_thumbnail_queue.is_empty() {
        schedule_deferred_thumbnail_catchup(app);
    }

    finalize_background_flow(app)
}

fn split_startup_thumbnail_work(
    app: &Librapix,
    items: Vec<ThumbnailWorkItem>,
) -> (Vec<ThumbnailWorkItem>, Vec<ThumbnailWorkItem>) {
    let priority_ids = startup_priority_media_ids(app, STARTUP_THUMBNAIL_PRIORITY_LIMIT);
    let priority_set = priority_ids.into_iter().collect::<HashSet<_>>();
    let mut immediate = Vec::new();
    let mut deferred = Vec::new();
    for item in items {
        let is_video = item.media_kind.eq_ignore_ascii_case("video");
        let is_priority = if priority_set.is_empty() {
            immediate.len() < STARTUP_THUMBNAIL_PRIORITY_LIMIT
        } else {
            priority_set.contains(&item.media_id)
        };
        if is_priority && !is_video && immediate.len() < STARTUP_THUMBNAIL_PRIORITY_LIMIT {
            immediate.push(item);
        } else {
            deferred.push(item);
        }
    }
    (immediate, deferred)
}

fn log_thumbnail_stage_start(app: &mut Librapix, mode: ThumbnailWorkMode, total: usize) {
    app.background.thumbnail_result_window_started_at = Some(Instant::now());
    app.background.thumbnail_result_window_batches = 0;
    app.background.thumbnail_result_window_outcomes = 0;
    app.background.thumbnail_result_window_failures = 0;
    app.background.thumbnail_refresh_requests_while_active = 0;
    match mode {
        ThumbnailWorkMode::StartupPriority => {
            app.startup_metrics.startup_thumbnail_started_at = Some(Instant::now());
            startup_log::log_info(
                "startup.thumbnail_priority.start",
                &format!("items={total}"),
            );
        }
        ThumbnailWorkMode::BackgroundCatchUp => {
            app.startup_metrics.deferred_thumbnail_started_at = Some(Instant::now());
            startup_log::log_info("startup.thumbnail_catchup.start", &format!("items={total}"));
        }
    }
}

fn flush_thumbnail_result_window(app: &mut Librapix, force: bool) {
    let Some(started_at) = app.background.thumbnail_result_window_started_at else {
        return;
    };
    let elapsed = started_at.elapsed();
    if !force && elapsed < THUMBNAIL_RESULT_LOG_WINDOW {
        return;
    }
    if app.background.thumbnail_result_window_batches == 0 {
        app.background.thumbnail_result_window_started_at = Some(Instant::now());
        return;
    }

    startup_log::log_info(
        "startup.thumbnail.message_rate",
        &format!(
            "mode={} window_ms={} batches_applied={} outcomes_applied={} failures_applied={} refresh_requests_while_active={}",
            app.background.thumbnail_mode.as_str(),
            elapsed.as_millis(),
            app.background.thumbnail_result_window_batches,
            app.background.thumbnail_result_window_outcomes,
            app.background.thumbnail_result_window_failures,
            app.background.thumbnail_refresh_requests_while_active,
        ),
    );
    app.background.thumbnail_result_window_started_at = Some(Instant::now());
    app.background.thumbnail_result_window_batches = 0;
    app.background.thumbnail_result_window_outcomes = 0;
    app.background.thumbnail_result_window_failures = 0;
}

fn log_thumbnail_stage_end(app: &mut Librapix) {
    flush_thumbnail_result_window(app, true);
    match app.background.thumbnail_mode {
        ThumbnailWorkMode::StartupPriority => {
            if let Some(started_at) = app.startup_metrics.startup_thumbnail_started_at.take() {
                startup_log::log_duration(
                    "startup.thumbnail_priority.end",
                    started_at.elapsed(),
                    &format!(
                        "total={} generated={} reused_exact={} reused_fallback={} failed={} refresh_requests_while_active={}",
                        app.background.thumbnail_total,
                        app.background.thumbnail_generated,
                        app.background.thumbnail_reused_exact,
                        app.background.thumbnail_reused_fallback,
                        app.background.thumbnail_failed,
                        app.background.thumbnail_refresh_requests_while_active,
                    ),
                );
            }
        }
        ThumbnailWorkMode::BackgroundCatchUp => {
            if let Some(started_at) = app.startup_metrics.deferred_thumbnail_started_at.take() {
                startup_log::log_duration(
                    "startup.thumbnail_catchup.end",
                    started_at.elapsed(),
                    &format!(
                        "total={} generated={} reused_exact={} reused_fallback={} failed={} refresh_requests_while_active={}",
                        app.background.thumbnail_total,
                        app.background.thumbnail_generated,
                        app.background.thumbnail_reused_exact,
                        app.background.thumbnail_reused_fallback,
                        app.background.thumbnail_failed,
                        app.background.thumbnail_refresh_requests_while_active,
                    ),
                );
            }
        }
    }
}

fn filter_thumbnail_candidates_for_runtime_policy(
    app: &mut Librapix,
    items: Vec<ThumbnailWorkItem>,
    context: &str,
) -> Vec<ThumbnailWorkItem> {
    let now = Instant::now();
    app.background
        .thumbnail_retry_state
        .retain(|_, state| state.next_retry_at > now);

    let mut allowed = Vec::with_capacity(items.len());
    let mut suppressed_backoff = 0usize;
    let mut suppressed_video_disabled = 0usize;
    let mut next_retry_ms = None::<u128>;
    let mut backoff_reason = None::<String>;

    for item in items {
        if item.media_kind.eq_ignore_ascii_case("video")
            && app.background.video_thumbnails_disabled_reason.is_some()
        {
            suppressed_video_disabled += 1;
            continue;
        }
        if let Some(state) = app.background.thumbnail_retry_state.get(&item.media_id)
            && state.next_retry_at > now
        {
            suppressed_backoff += 1;
            let retry_after = state.next_retry_at.duration_since(now).as_millis();
            next_retry_ms =
                Some(next_retry_ms.map_or(retry_after, |existing| existing.min(retry_after)));
            if backoff_reason.is_none() {
                backoff_reason = Some(format!(
                    "{}:{}",
                    state.failure_class.as_str(),
                    state.last_error
                ));
            }
            continue;
        }
        allowed.push(item);
    }

    if suppressed_backoff > 0 || suppressed_video_disabled > 0 {
        startup_log::log_info(
            "startup.thumbnail.schedule_suppressed",
            &format!(
                "context={context} allowed={} suppressed_backoff={} suppressed_video_disabled={} next_retry_ms={} backoff_reason={}",
                allowed.len(),
                suppressed_backoff,
                suppressed_video_disabled,
                next_retry_ms.unwrap_or_default(),
                backoff_reason.unwrap_or_default(),
            ),
        );
    }

    allowed
}

fn prune_thumbnail_queues_for_runtime_policy(app: &mut Librapix, context: &str) {
    let now = Instant::now();
    let mut removed_from_active = 0usize;
    let mut removed_from_deferred = 0usize;
    let mut active = VecDeque::new();
    let mut deferred = VecDeque::new();

    while let Some(item) = app.background.thumbnail_queue.pop_front() {
        let suppress = (item.media_kind.eq_ignore_ascii_case("video")
            && app.background.video_thumbnails_disabled_reason.is_some())
            || app
                .background
                .thumbnail_retry_state
                .get(&item.media_id)
                .is_some_and(|state| state.next_retry_at > now);
        if suppress {
            app.background.thumbnail_queued_ids.remove(&item.media_id);
            removed_from_active += 1;
        } else {
            active.push_back(item);
        }
    }

    while let Some(item) = app.background.deferred_thumbnail_queue.pop_front() {
        let suppress = (item.media_kind.eq_ignore_ascii_case("video")
            && app.background.video_thumbnails_disabled_reason.is_some())
            || app
                .background
                .thumbnail_retry_state
                .get(&item.media_id)
                .is_some_and(|state| state.next_retry_at > now);
        if suppress {
            removed_from_deferred += 1;
        } else {
            deferred.push_back(item);
        }
    }

    app.background.thumbnail_queue = active;
    app.background.deferred_thumbnail_queue = deferred;
    if removed_from_active > 0 {
        app.background.thumbnail_total = app
            .background
            .thumbnail_total
            .saturating_sub(removed_from_active);
        app.activity_progress.items_total = Some(app.background.thumbnail_total);
    }
    if removed_from_active > 0 || removed_from_deferred > 0 {
        startup_log::log_info(
            "startup.thumbnail.queue_pruned",
            &format!(
                "context={context} removed_active={} removed_deferred={} remaining_active={} remaining_deferred={}",
                removed_from_active,
                removed_from_deferred,
                app.background.thumbnail_queue.len(),
                app.background.deferred_thumbnail_queue.len(),
            ),
        );
    }
}

fn schedule_deferred_thumbnail_catchup(app: &mut Librapix) {
    if app.background.deferred_thumbnail_queue.is_empty()
        || app.background.deferred_thumbnail_due_at.is_some()
    {
        return;
    }
    app.background.deferred_thumbnail_due_at =
        Some(Instant::now() + Duration::from_millis(DEFERRED_THUMBNAIL_DELAY_MS));
}

fn start_deferred_thumbnail_catchup(app: &mut Librapix) -> Task<Message> {
    if app.background.deferred_thumbnail_queue.is_empty() {
        app.background.deferred_thumbnail_due_at = None;
        return Task::none();
    }
    let Some(due_at) = app.background.deferred_thumbnail_due_at else {
        return Task::none();
    };
    if Instant::now() < due_at
        || app.background.snapshot_apply.is_some()
        || app.background.reconcile_in_flight
        || app.background.projection_in_flight
        || app.background.thumbnail_in_flight
        || app.background.pending_reconcile
        || app.background.pending_projection
    {
        return Task::none();
    }

    app.background.deferred_thumbnail_due_at = None;
    let items = app
        .background
        .deferred_thumbnail_queue
        .drain(..)
        .collect::<Vec<_>>();
    start_thumbnail_batches(app, items, ThumbnailWorkMode::BackgroundCatchUp)
}

fn thumbnail_batch_limits(mode: ThumbnailWorkMode) -> (usize, usize) {
    match mode {
        ThumbnailWorkMode::StartupPriority => (
            STARTUP_IMAGE_THUMBNAIL_BATCH_SIZE,
            MAX_VIDEO_THUMBNAILS_PER_BATCH,
        ),
        ThumbnailWorkMode::BackgroundCatchUp => (
            BACKGROUND_IMAGE_THUMBNAIL_BATCH_SIZE,
            MAX_VIDEO_THUMBNAILS_PER_BATCH,
        ),
    }
}

fn take_next_thumbnail_batch(app: &mut Librapix) -> Vec<ThumbnailWorkItem> {
    let (max_items, max_videos) = thumbnail_batch_limits(app.background.thumbnail_mode);
    let mut batch = Vec::new();
    let mut remaining = VecDeque::new();
    let mut video_count = 0usize;

    while let Some(item) = app.background.thumbnail_queue.pop_front() {
        let is_video = item.media_kind.eq_ignore_ascii_case("video");
        let allow = batch.len() < max_items && (!is_video || video_count < max_videos);
        if allow {
            if is_video {
                video_count += 1;
            }
            batch.push(item);
        } else {
            remaining.push_back(item);
        }
    }

    app.background.thumbnail_queue = remaining;
    batch
}

fn start_thumbnail_batches(
    app: &mut Librapix,
    items: Vec<ThumbnailWorkItem>,
    mode: ThumbnailWorkMode,
) -> Task<Message> {
    let items =
        filter_thumbnail_candidates_for_runtime_policy(app, items, "start_thumbnail_batches");
    app.background.thumbnail_generation = app.background.thumbnail_generation.saturating_add(1);
    let generation = app.background.thumbnail_generation;
    app.background
        .thumbnail_cancel_generation
        .store(generation, std::sync::atomic::Ordering::Relaxed);
    app.background.thumbnail_queue.clear();
    app.background.thumbnail_queued_ids.clear();
    app.background.thumbnail_in_flight = false;
    app.background.thumbnail_mode = mode;
    app.background.thumbnail_done = 0;
    app.background.thumbnail_generated = 0;
    app.background.thumbnail_reused_exact = 0;
    app.background.thumbnail_reused_fallback = 0;
    app.background.thumbnail_failed = 0;
    app.background.thumbnail_batch_id = 0;

    let mut queued = VecDeque::new();
    let mut queued_ids = HashSet::new();
    for mut item in items {
        if queued_ids.insert(item.media_id) {
            item.generation = generation;
            queued.push_back(item);
        }
    }
    app.background.thumbnail_total = queued.len();
    app.background.thumbnail_queue = queued;
    app.background.thumbnail_queued_ids = queued_ids;
    app.activity_progress.queue_depth = app.background.thumbnail_queue.len();
    app.thumbnail_status = thumbnail_status_text(app.i18n, 0, 0, 0);
    log_thumbnail_stage_start(app, mode, app.background.thumbnail_total);

    if app.background.thumbnail_total == 0 {
        log_thumbnail_stage_end(app);
        return finalize_background_flow(app);
    }

    set_activity_stage(
        app,
        TextKey::StageGeneratingThumbnailsLabel,
        String::new(),
        false,
    );
    app.activity_progress.items_total = Some(app.background.thumbnail_total);
    app.activity_progress.items_done = 0;

    run_next_thumbnail_batch_if_idle(app)
}

fn run_next_thumbnail_batch_if_idle(app: &mut Librapix) -> Task<Message> {
    if app.background.thumbnail_in_flight {
        return Task::none();
    }
    prune_thumbnail_queues_for_runtime_policy(app, "run_next_thumbnail_batch_if_idle");
    if app.background.thumbnail_queue.is_empty() {
        return Task::none();
    }

    app.background.thumbnail_in_flight = true;
    let batch = take_next_thumbnail_batch(app);
    app.background.thumbnail_batch_id = app.background.thumbnail_batch_id.saturating_add(1);
    let batch_id = app.background.thumbnail_batch_id;
    let image_items = batch
        .iter()
        .filter(|item| item.media_kind.eq_ignore_ascii_case("image"))
        .count();
    let video_items = batch.len().saturating_sub(image_items);

    app.activity_progress.items_total = Some(app.background.thumbnail_total);
    app.activity_progress.items_done = app.background.thumbnail_done;
    app.activity_progress.queue_depth = app.background.thumbnail_queue.len() + batch.len();
    set_activity_stage(
        app,
        TextKey::StageGeneratingThumbnailsLabel,
        String::new(),
        false,
    );

    startup_log::log_info(
        "startup.thumbnail.batch.dispatch",
        &format!(
            "generation={} batch_id={} mode={} items={} images={} videos={} queue_remaining={}",
            app.background.thumbnail_generation,
            batch_id,
            app.background.thumbnail_mode.as_str(),
            batch.len(),
            image_items,
            video_items,
            app.background.thumbnail_queue.len(),
        ),
    );

    let input = ThumbnailBatchInput {
        generation: app.background.thumbnail_generation,
        batch_id,
        mode: app.background.thumbnail_mode,
        database_file: app.runtime.database_file.clone(),
        thumbnails_dir: app.runtime.thumbnails_dir.clone(),
        cancellation: ThumbnailCancellation::new(
            Arc::clone(&app.background.thumbnail_cancel_generation),
            app.background.thumbnail_generation,
        ),
        items: batch,
    };
    Task::perform(async move { do_thumbnail_batch(input) }, |mut result| {
        let dispatched_to_ui_at = Instant::now();
        let worker_to_dispatch_ms = result
            .worker_finished_at
            .map(|finished_at| dispatched_to_ui_at.duration_since(finished_at).as_millis())
            .unwrap_or_default();
        startup_log::log_info(
            "startup.thumbnail.batch.dispatch_to_ui",
            &format!(
                "generation={} batch_id={} mode={} outcomes={} failures={} worker_elapsed_ms={} worker_to_dispatch_ms={}",
                result.generation,
                result.batch_id,
                result.mode.as_str(),
                result.completed_media_ids.len(),
                result.failures.len(),
                result.worker_elapsed.as_millis(),
                worker_to_dispatch_ms,
            ),
        );
        result.dispatched_to_ui_at = Some(dispatched_to_ui_at);
        Message::ThumbnailBatchComplete(Box::new(result))
    })
}

fn compatible_detail_thumbnail_for_work_item(
    thumbnails_dir: &Path,
    item: &ThumbnailWorkItem,
) -> Option<PathBuf> {
    let path = thumbnail_path(
        thumbnails_dir,
        &item.absolute_path,
        item.file_size_bytes,
        item.modified_unix_seconds,
        DETAIL_THUMB_SIZE,
    );
    path.is_file().then_some(path)
}

fn thumbnail_failure_from_error(
    item: &ThumbnailWorkItem,
    error: &ThumbnailError,
) -> ThumbnailFailureEvent {
    match error {
        ThumbnailError::Video(video_error) => {
            let failure_class = match video_error.kind {
                VideoThumbnailErrorKind::FfmpegNotFound => {
                    ThumbnailFailureClass::VideoFfmpegNotFound
                }
                VideoThumbnailErrorKind::SpawnFailed => ThumbnailFailureClass::VideoSpawnFailed,
                VideoThumbnailErrorKind::TimedOut => ThumbnailFailureClass::VideoTimedOut,
                VideoThumbnailErrorKind::ExitNonZero => ThumbnailFailureClass::VideoExitNonZero,
                VideoThumbnailErrorKind::MissingOutput => ThumbnailFailureClass::VideoMissingOutput,
                VideoThumbnailErrorKind::Cancelled => ThumbnailFailureClass::Cancelled,
            };
            ThumbnailFailureEvent {
                media_id: item.media_id,
                media_kind: item.media_kind.clone(),
                failure_class,
                detail: error.to_string(),
                command_line: Some(video_error.command_line.clone()),
                ffmpeg_path: video_error.ffmpeg_path.clone(),
                exit_code: video_error.exit_code,
                stderr_summary: video_error.stderr_summary.clone(),
                timeout_ms: video_error.timeout_ms,
                hard_failure: !matches!(video_error.kind, VideoThumbnailErrorKind::Cancelled),
                disable_video_for_session: matches!(
                    video_error.kind,
                    VideoThumbnailErrorKind::FfmpegNotFound | VideoThumbnailErrorKind::SpawnFailed
                ),
            }
        }
        ThumbnailError::Image(_) => ThumbnailFailureEvent {
            media_id: item.media_id,
            media_kind: item.media_kind.clone(),
            failure_class: ThumbnailFailureClass::ImageDecode,
            detail: error.to_string(),
            command_line: None,
            ffmpeg_path: None,
            exit_code: None,
            stderr_summary: None,
            timeout_ms: None,
            hard_failure: true,
            disable_video_for_session: false,
        },
        ThumbnailError::Io(_) => ThumbnailFailureEvent {
            media_id: item.media_id,
            media_kind: item.media_kind.clone(),
            failure_class: if item.media_kind.eq_ignore_ascii_case("image") {
                ThumbnailFailureClass::ImageIo
            } else {
                ThumbnailFailureClass::Unknown
            },
            detail: error.to_string(),
            command_line: None,
            ffmpeg_path: None,
            exit_code: None,
            stderr_summary: None,
            timeout_ms: None,
            hard_failure: false,
            disable_video_for_session: false,
        },
    }
}

fn log_thumbnail_failure_event(
    generation: u64,
    batch_id: u64,
    item: &ThumbnailWorkItem,
    failure: &ThumbnailFailureEvent,
) {
    if item.media_kind.eq_ignore_ascii_case("video") {
        startup_log::log_error(
            "startup.thumbnail.video.failure",
            &format!(
                "generation={} batch_id={} media_id={} class={} ffmpeg={} command={} exit_code={:?} timeout_ms={:?} stderr={:?} detail={}",
                generation,
                batch_id,
                item.media_id,
                failure.failure_class.as_str(),
                failure
                    .ffmpeg_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
                failure.command_line.clone().unwrap_or_default(),
                failure.exit_code,
                failure.timeout_ms,
                failure.stderr_summary,
                failure.detail,
            ),
        );
    } else {
        startup_log::log_error(
            "startup.thumbnail.generate.failed",
            &format!(
                "generation={} batch_id={} media_id={} kind={} class={} detail={}",
                generation,
                batch_id,
                item.media_id,
                item.media_kind,
                failure.failure_class.as_str(),
                failure.detail,
            ),
        );
    }
}

fn do_thumbnail_batch(input: ThumbnailBatchInput) -> ThumbnailBatchResult {
    let batch_started_at = Instant::now();
    let generation = input.generation;
    let batch_id = input.batch_id;
    let mode = input.mode;
    let database_file = input.database_file;
    let thumbnails_dir = input.thumbnails_dir;
    let cancellation = input.cancellation;
    let items = input.items;
    let image_items = items
        .iter()
        .filter(|item| item.media_kind.eq_ignore_ascii_case("image"))
        .count();
    let video_items = items.len().saturating_sub(image_items);
    startup_log::log_info(
        "startup.thumbnail.batch.start",
        &format!(
            "generation={} batch_id={} mode={} items={} images={} videos={}",
            generation,
            batch_id,
            mode.as_str(),
            items.len(),
            image_items,
            video_items,
        ),
    );
    let mut out = ThumbnailBatchResult {
        generation,
        batch_id,
        mode,
        image_items,
        video_items,
        ..Default::default()
    };
    let storage = Storage::open(&database_file).ok();

    for item in items {
        if cancellation.is_cancelled() {
            out.cancelled = true;
            break;
        }
        let item_started_at = Instant::now();
        let exact_path = thumbnail_path(
            &thumbnails_dir,
            &item.absolute_path,
            item.file_size_bytes,
            item.modified_unix_seconds,
            GALLERY_THUMB_SIZE,
        );
        if exact_path.is_file() {
            out.attempted += 1;
            out.reused_exact += 1;
            if let Some(storage) = storage.as_ref() {
                let relative_path = relative_artifact_path(&thumbnails_dir, &exact_path);
                if let Err(error) = storage.upsert_derived_artifact(
                    item.media_id,
                    DerivedArtifactKind::Thumbnail,
                    GALLERY_THUMB_VARIANT,
                    relative_path.as_deref(),
                    DerivedArtifactStatus::Ready,
                ) {
                    out.errors.push(error.to_string());
                }
            }
            out.completed_media_ids.push(item.media_id);
            if item.media_kind.eq_ignore_ascii_case("video") {
                startup_log::log_duration(
                    "startup.thumbnail.video",
                    item_started_at.elapsed(),
                    &format!(
                        "generation={} batch_id={} media_id={} mode=exact_reuse",
                        generation, batch_id, item.media_id,
                    ),
                );
            }
            out.outcomes.push(ThumbnailWorkOutcome {
                media_id: item.media_id,
                thumbnail_path: Some(exact_path),
            });
            continue;
        }

        if let Some(fallback_path) =
            compatible_detail_thumbnail_for_work_item(&thumbnails_dir, &item)
        {
            out.attempted += 1;
            out.reused_fallback += 1;
            out.completed_media_ids.push(item.media_id);
            if item.media_kind.eq_ignore_ascii_case("video") {
                startup_log::log_duration(
                    "startup.thumbnail.video",
                    item_started_at.elapsed(),
                    &format!(
                        "generation={} batch_id={} media_id={} mode=fallback_reuse",
                        generation, batch_id, item.media_id,
                    ),
                );
            }
            out.outcomes.push(ThumbnailWorkOutcome {
                media_id: item.media_id,
                thumbnail_path: Some(fallback_path),
            });
            continue;
        }

        out.attempted += 1;
        let thumbnail_result = if item.media_kind.eq_ignore_ascii_case("image") {
            ensure_image_thumbnail(
                &thumbnails_dir,
                &item.absolute_path,
                item.file_size_bytes,
                item.modified_unix_seconds,
                GALLERY_THUMB_SIZE,
            )
        } else if item.media_kind.eq_ignore_ascii_case("video") {
            ensure_video_thumbnail_with_options(
                &thumbnails_dir,
                &item.absolute_path,
                item.file_size_bytes,
                item.modified_unix_seconds,
                GALLERY_THUMB_SIZE,
                VideoThumbnailOptions {
                    cancellation: Some(cancellation.clone()),
                    ..VideoThumbnailOptions::default()
                },
            )
        } else {
            continue;
        };

        match thumbnail_result {
            Ok(thumbnail) => {
                if thumbnail.generated {
                    out.generated += 1;
                } else {
                    out.reused_exact += 1;
                }
                if let Some(storage) = storage.as_ref() {
                    let relative_path =
                        relative_artifact_path(&thumbnails_dir, &thumbnail.thumbnail_path);
                    if let Err(error) = storage.upsert_derived_artifact(
                        item.media_id,
                        DerivedArtifactKind::Thumbnail,
                        GALLERY_THUMB_VARIANT,
                        relative_path.as_deref(),
                        DerivedArtifactStatus::Ready,
                    ) {
                        out.errors.push(error.to_string());
                    }
                }
                out.outcomes.push(ThumbnailWorkOutcome {
                    media_id: item.media_id,
                    thumbnail_path: Some(thumbnail.thumbnail_path),
                });
                let outcome_label = if thumbnail.generated {
                    "generated"
                } else {
                    "exact_reuse"
                };
                if item.media_kind.eq_ignore_ascii_case("video") {
                    startup_log::log_duration(
                        "startup.thumbnail.video",
                        item_started_at.elapsed(),
                        &format!(
                            "generation={} batch_id={} media_id={} mode={}",
                            generation, batch_id, item.media_id, outcome_label,
                        ),
                    );
                } else if item_started_at.elapsed() >= Duration::from_millis(150) {
                    startup_log::log_duration(
                        "startup.thumbnail.item.slow",
                        item_started_at.elapsed(),
                        &format!(
                            "generation={} batch_id={} media_id={} kind={} mode={}",
                            generation, batch_id, item.media_id, item.media_kind, outcome_label,
                        ),
                    );
                }
                out.completed_media_ids.push(item.media_id);
            }
            Err(error) => {
                let failure = thumbnail_failure_from_error(&item, &error);
                if matches!(failure.failure_class, ThumbnailFailureClass::Cancelled) {
                    out.cancelled = true;
                    break;
                }
                if let Some(storage) = storage.as_ref() {
                    let _ = storage.upsert_derived_artifact(
                        item.media_id,
                        DerivedArtifactKind::Thumbnail,
                        GALLERY_THUMB_VARIANT,
                        None,
                        DerivedArtifactStatus::Failed,
                    );
                }
                out.failed += 1;
                out.errors.push(error.to_string());
                out.failures.push(failure.clone());
                out.completed_media_ids.push(item.media_id);
                log_thumbnail_failure_event(generation, batch_id, &item, &failure);
            }
        }
    }

    out.worker_elapsed = batch_started_at.elapsed();
    out.worker_finished_at = Some(Instant::now());
    if out.cancelled {
        startup_log::log_duration(
            "startup.thumbnail.batch.cancelled",
            out.worker_elapsed,
            &format!(
                "generation={} batch_id={} mode={} attempted={} images={} videos={} queue_cancelled=true",
                out.generation,
                out.batch_id,
                out.mode.as_str(),
                out.attempted,
                out.image_items,
                out.video_items,
            ),
        );
    } else {
        startup_log::log_duration(
            "startup.thumbnail.batch.end",
            out.worker_elapsed,
            &format!(
                "generation={} batch_id={} mode={} attempted={} images={} videos={} generated={} reused_exact={} reused_fallback={} failed={}",
                out.generation,
                out.batch_id,
                out.mode.as_str(),
                out.attempted,
                out.image_items,
                out.video_items,
                out.generated,
                out.reused_exact,
                out.reused_fallback,
                out.failed,
            ),
        );
    }
    out
}

fn patch_thumbnail_paths(items: &mut [BrowseItem], updates: &HashMap<i64, PathBuf>) {
    if updates.is_empty() {
        return;
    }
    for item in items.iter_mut() {
        if let Some(path) = updates.get(&item.media_id) {
            item.thumbnail_path = Some(path.clone());
        }
    }
}

fn disable_video_thumbnails_for_session(app: &mut Librapix, failure: &ThumbnailFailureEvent) {
    if app.background.video_thumbnails_disabled_reason.is_some() {
        return;
    }

    app.background.video_thumbnails_disabled_reason = Some(failure.detail.clone());
    app.background.video_thumbnails_disabled_ffmpeg = failure.ffmpeg_path.clone();
    startup_log::log_warn(
        "startup.thumbnail.video.disabled_session",
        &format!(
            "reason={} ffmpeg={}",
            failure.detail,
            failure
                .ffmpeg_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
        ),
    );
}

fn thumbnail_failure_cooldown(failure: &ThumbnailFailureEvent, attempts: u32) -> Duration {
    let base = if failure.media_kind.eq_ignore_ascii_case("video") {
        if failure.disable_video_for_session {
            VIDEO_SESSION_DISABLE_COOLDOWN
        } else {
            VIDEO_THUMBNAIL_RETRY_COOLDOWN
        }
    } else {
        IMAGE_THUMBNAIL_RETRY_COOLDOWN
    };
    base.saturating_mul(attempts.clamp(1, 3))
}

fn record_thumbnail_failure(app: &mut Librapix, failure: &ThumbnailFailureEvent) {
    if failure.disable_video_for_session {
        disable_video_thumbnails_for_session(app, failure);
    }

    let attempts = app
        .background
        .thumbnail_retry_state
        .get(&failure.media_id)
        .map(|state| state.attempts.saturating_add(1))
        .unwrap_or(1);
    let cooldown = thumbnail_failure_cooldown(failure, attempts);
    let next_retry_at = Instant::now() + cooldown;
    app.background.thumbnail_retry_state.insert(
        failure.media_id,
        ThumbnailRetryState {
            attempts,
            next_retry_at,
            failure_class: failure.failure_class,
            last_error: failure.detail.clone(),
        },
    );
    startup_log::log_info(
        "startup.thumbnail.backoff.applied",
        &format!(
            "media_id={} kind={} class={} attempts={} retry_after_ms={} hard_failure={} disable_video_for_session={}",
            failure.media_id,
            failure.media_kind,
            failure.failure_class.as_str(),
            attempts,
            cooldown.as_millis(),
            failure.hard_failure,
            failure.disable_video_for_session,
        ),
    );
}

fn apply_thumbnail_batch_result(app: &mut Librapix, result: ThumbnailBatchResult) -> Task<Message> {
    let apply_started_at = Instant::now();
    let worker_to_receive_ms = result
        .worker_finished_at
        .map(|finished_at| apply_started_at.duration_since(finished_at).as_millis())
        .unwrap_or_default();
    let dispatch_to_receive_ms = result
        .dispatched_to_ui_at
        .map(|dispatched_at| apply_started_at.duration_since(dispatched_at).as_millis())
        .unwrap_or_default();
    startup_log::log_info(
        "startup.thumbnail.batch.message_received",
        &format!(
            "generation={} batch_id={} mode={} outcomes={} failures={} worker_elapsed_ms={} worker_to_receive_ms={} dispatch_to_receive_ms={}",
            result.generation,
            result.batch_id,
            result.mode.as_str(),
            result.completed_media_ids.len(),
            result.failures.len(),
            result.worker_elapsed.as_millis(),
            worker_to_receive_ms,
            dispatch_to_receive_ms,
        ),
    );
    if worker_to_receive_ms > 250 {
        startup_log::log_warn(
            "startup.thumbnail.batch.handoff.slow",
            &format!(
                "generation={} batch_id={} mode={} worker_to_receive_ms={} dispatch_to_receive_ms={}",
                result.generation,
                result.batch_id,
                result.mode.as_str(),
                worker_to_receive_ms,
                dispatch_to_receive_ms,
            ),
        );
    }
    if result.generation != app.background.thumbnail_generation {
        app.background.thumbnail_in_flight = false;
        startup_log::log_info(
            "startup.thumbnail.batch.stale",
            &format!(
                "generation={} active_generation={} batch_id={} mode={} attempted={} cancelled={}",
                result.generation,
                app.background.thumbnail_generation,
                result.batch_id,
                result.mode.as_str(),
                result.attempted,
                result.cancelled,
            ),
        );
        return finalize_background_flow(app);
    }

    app.background.thumbnail_in_flight = false;
    app.background.thumbnail_generated += result.generated;
    app.background.thumbnail_reused_exact += result.reused_exact;
    app.background.thumbnail_reused_fallback += result.reused_fallback;
    app.background.thumbnail_failed += result.failed;

    let mut ready_paths = HashMap::<i64, PathBuf>::new();
    for media_id in &result.completed_media_ids {
        app.background.thumbnail_queued_ids.remove(media_id);
        app.background.thumbnail_done = app.background.thumbnail_done.saturating_add(1);
    }
    for outcome in result.outcomes {
        if let Some(path) = outcome.thumbnail_path {
            app.background
                .thumbnail_retry_state
                .remove(&outcome.media_id);
            ready_paths.insert(outcome.media_id, path.clone());
            if let Some(details) = app.media_cache.get_mut(&outcome.media_id)
                && details.detail_thumbnail_path.is_none()
            {
                details.detail_thumbnail_path = Some(path.clone());
            }
            if app.state.selected_media_id == Some(outcome.media_id) {
                app.details_preview_path = Some(path.clone());
            }
            if let Some(announcement) = app.new_media_announcement.as_mut()
                && announcement.media_id == outcome.media_id
            {
                announcement.preview_path = Some(path);
            }
        }
    }

    for failure in &result.failures {
        record_thumbnail_failure(app, failure);
    }
    prune_thumbnail_queues_for_runtime_policy(app, "apply_thumbnail_batch_result");

    patch_thumbnail_paths(&mut app.gallery_items, &ready_paths);
    patch_thumbnail_paths(&mut app.timeline_items, &ready_paths);
    patch_thumbnail_paths(&mut app.search_items, &ready_paths);

    if let Some(first_error) = result.errors.into_iter().next() {
        app.activity_progress.last_error = Some(first_error);
    }
    app.thumbnail_status = thumbnail_status_text(
        app.i18n,
        app.background.thumbnail_generated,
        app.background.thumbnail_reused_exact + app.background.thumbnail_reused_fallback,
        app.background.thumbnail_failed,
    );
    app.activity_progress.items_total = Some(app.background.thumbnail_total);
    app.activity_progress.items_done = app.background.thumbnail_done;
    app.activity_progress.queue_depth = app.background.thumbnail_queue.len();
    app.background.thumbnail_result_window_batches = app
        .background
        .thumbnail_result_window_batches
        .saturating_add(1);
    app.background.thumbnail_result_window_outcomes = app
        .background
        .thumbnail_result_window_outcomes
        .saturating_add(result.completed_media_ids.len());
    app.background.thumbnail_result_window_failures = app
        .background
        .thumbnail_result_window_failures
        .saturating_add(result.failures.len());
    flush_thumbnail_result_window(app, false);
    startup_log::log_info(
        "startup.thumbnail.apply.start",
        &format!(
            "generation={} batch_id={} mode={} outcomes={} failures={} worker_to_receive_ms={} dispatch_to_receive_ms={}",
            result.generation,
            result.batch_id,
            result.mode.as_str(),
            result.completed_media_ids.len(),
            result.failures.len(),
            worker_to_receive_ms,
            dispatch_to_receive_ms,
        ),
    );
    startup_log::log_duration(
        "startup.thumbnail.apply",
        apply_started_at.elapsed(),
        &format!(
            "generation={} batch_id={} mode={} outcomes={} ready_paths={} failures={} worker_elapsed_ms={} worker_to_receive_ms={} dispatch_to_receive_ms={} queue_remaining={} deferred_remaining={} pending_projection={} pending_reconcile={}",
            result.generation,
            result.batch_id,
            result.mode.as_str(),
            result.completed_media_ids.len(),
            ready_paths.len(),
            result.failures.len(),
            result.worker_elapsed.as_millis(),
            worker_to_receive_ms,
            dispatch_to_receive_ms,
            app.background.thumbnail_queue.len(),
            app.background.deferred_thumbnail_queue.len(),
            app.background.pending_projection,
            app.background.pending_reconcile,
        ),
    );

    if result.cancelled {
        return finalize_background_flow(app);
    }

    if !app.background.thumbnail_queue.is_empty() {
        return run_next_thumbnail_batch_if_idle(app);
    }

    log_thumbnail_stage_end(app);
    finalize_background_flow(app)
}

fn finalize_background_flow(app: &mut Librapix) -> Task<Message> {
    if app.background.snapshot_apply.is_some() {
        return Task::none();
    }
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
    if !app.background.startup_ready && startup_blocking_work_complete(app) {
        mark_startup_ready(app);
    }
    if app.background.startup_ready
        && !app.background.thumbnail_in_flight
        && app.background.thumbnail_queue.is_empty()
        && app.background.deferred_thumbnail_due_at.is_none()
        && !app.background.deferred_thumbnail_queue.is_empty()
    {
        schedule_deferred_thumbnail_catchup(app);
    }
    if app.background.startup_ready && all_background_work_idle(app) {
        startup_log::log_info(
            "startup.gallery_working.clear",
            &format!(
                "owner=finalize route={:?} browse_status={} pending_reconcile={} pending_projection={} thumbnail_in_flight={} deferred_thumbnail_due={} deferred_thumbnail_queue={}",
                app.state.active_route,
                app.browse_status,
                app.background.pending_reconcile,
                app.background.pending_projection,
                app.background.thumbnail_in_flight,
                app.background.deferred_thumbnail_due_at.is_some(),
                app.background.deferred_thumbnail_queue.len(),
            ),
        );
        startup_log::log_info(
            "interaction.route_working.clear",
            &format!(
                "owner=finalize route={} browse_status={} gallery_items={} timeline_items={} pending_reconcile={} pending_projection={} thumbnail_in_flight={} deferred_thumbnail_queue={} {}",
                route_name(app.state.active_route),
                app.browse_status,
                app.gallery_items.len(),
                app.timeline_items.len(),
                app.background.pending_reconcile,
                app.background.pending_projection,
                app.background.thumbnail_in_flight,
                app.background.deferred_thumbnail_queue.len(),
                filter_state_summary(app),
            ),
        );
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
    if let Some(path) = app.startup_log_path.as_ref() {
        lines.push(format!("startup log: {}", path.display()));
    }
    lines.push(format!(
        "configured roots: {}",
        app.runtime.configured_library_roots.len()
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
        "filter: kind={:?}, ext={:?}, tag={:?}",
        app.filter_media_kind.as_deref().unwrap_or("all"),
        app.filter_extension.as_deref().unwrap_or("all"),
        app.filter_tag.as_deref().unwrap_or("all")
    ));
    lines.push(format!("min file size: {} bytes", app.min_file_size_bytes));
    lines.push(format!(
        "updates: {}",
        update_check_status_label(&app.update_check_state)
    ));
    lines.push(format!(
        "startup ready recorded: {}",
        app.startup_metrics.startup_ready_recorded
    ));
    lines.push(format!(
        "first usable gallery recorded: {}",
        app.startup_metrics.first_usable_gallery_recorded
    ));
    if let Some(reason) = app.background.video_thumbnails_disabled_reason.as_deref() {
        lines.push(format!(
            "video thumbnails disabled: {} ({})",
            reason,
            app.background
                .video_thumbnails_disabled_ffmpeg
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default()
        ));
    }
    lines.push(format!("browse status: {}", app.browse_status));

    app.diagnostics_lines = lines;
}

fn build_new_media_announcement(
    i18n: Translator,
    previous_media_ids: &std::collections::HashSet<i64>,
    current_media_cache: &std::collections::HashMap<i64, CachedDetails>,
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

    fn test_app() -> Librapix {
        Librapix {
            state: AppState::default(),
            i18n: Translator::new(Locale::EnUs),
            theme_preference: ThemePreference::System,
            runtime: RuntimeContext {
                database_file: PathBuf::from("/tmp/librapix-test.db"),
                thumbnails_dir: PathBuf::from("/tmp/librapix-thumbnails"),
                config_file: PathBuf::from("/tmp/librapix-config.toml"),
                configured_library_roots: Vec::new(),
                default_shorts_output_dir: None,
            },
            startup_log_path: None,
            thumbnail_status: String::new(),
            details_tag_input: String::new(),
            details_lines: Vec::new(),
            details_action_status: String::new(),
            details_preview_path: None,
            details_title: String::new(),
            details_tags: Vec::new(),
            details_editing_tag: None,
            details_loaded_media_ids: HashSet::new(),
            make_short_dialog: MakeShortDialogState::default(),
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
            shorts_output_dir_input: String::new(),
            media_cache: HashMap::new(),
            background: BackgroundCoordinator::default(),
            diagnostics_lines: Vec::new(),
            diagnostics_events: Vec::new(),
            media_scroll_absolute_y: 0.0,
            media_scroll_max_y: 0.0,
            media_viewport_height: 0.0,
            timeline_scrub_value: 0.0,
            timeline_scrubbing: false,
            timeline_scrub_anchor_index: None,
            timeline_scroll_max_y: 0.0,
            browse_layout_generation: 0,
            layout_cache: RefCell::new(MediaLayoutCache::default()),
            drag_layout_preview: RefCell::new(DragLayoutPreviewState::default()),
            viewport_drag: ViewportDragState::default(),
            last_viewport_drag_settled_at: None,
            new_media_announcement: None,
            new_media_preview_loading_phase: 0,
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
            startup_metrics: StartupFlowMetrics::default(),
        }
    }

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
            CatalogMediaRecord {
                media_id: 1,
                source_root_id: 10,
                source_root_display_name: None,
                absolute_path: PathBuf::from("/tmp/a.png"),
                file_name: "a.png".to_owned(),
                file_extension: Some("png".to_owned()),
                media_kind: "image".to_owned(),
                file_size_bytes: 100,
                modified_unix_seconds: Some(10),
                width_px: Some(10),
                height_px: Some(10),
                metadata_status: librapix_storage::IndexedMetadataStatus::Ok,
                search_text: String::new(),
                timeline_day_key: Some("1970-01-01".to_owned()),
                timeline_month_key: Some("1970-01".to_owned()),
                timeline_year_key: Some("1970".to_owned()),
                tags: vec![
                    "kind:image".to_owned(),
                    "Boss".to_owned(),
                    "campaign".to_owned(),
                ],
            },
            CatalogMediaRecord {
                media_id: 2,
                source_root_id: 10,
                source_root_display_name: None,
                absolute_path: PathBuf::from("/tmp/b.mp4"),
                file_name: "b.mp4".to_owned(),
                file_extension: Some("mp4".to_owned()),
                media_kind: "video".to_owned(),
                file_size_bytes: 100,
                modified_unix_seconds: Some(20),
                width_px: None,
                height_px: None,
                metadata_status: librapix_storage::IndexedMetadataStatus::Ok,
                search_text: String::new(),
                timeline_day_key: Some("1970-01-01".to_owned()),
                timeline_month_key: Some("1970-01".to_owned()),
                timeline_year_key: Some("1970".to_owned()),
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
    fn parse_automation_script_supports_waits_filters_and_selection() {
        let steps = parse_automation_script(
            "wait:250;timeline;wait:100;filter_kind:video;select_first;wait:50;filter_kind:none",
        );

        assert_eq!(
            steps,
            vec![
                AutomationStep {
                    wait_ms: 250,
                    action: AutomationAction::OpenTimeline,
                },
                AutomationStep {
                    wait_ms: 100,
                    action: AutomationAction::SetFilterMediaKind(Some("video".to_owned())),
                },
                AutomationStep {
                    wait_ms: 0,
                    action: AutomationAction::SelectFirstVisible,
                },
                AutomationStep {
                    wait_ms: 50,
                    action: AutomationAction::SetFilterMediaKind(None),
                },
            ]
        );
    }

    #[test]
    fn automation_tick_defers_while_background_work_is_active() {
        let mut app = test_app();
        app.background.startup_ready = true;
        app.background.projection_in_flight = true;
        app.background.automation = Some(AutomationRunner {
            steps: VecDeque::from([AutomationStep {
                wait_ms: 0,
                action: AutomationAction::OpenTimeline,
            }]),
            due_at: Some(Instant::now() - Duration::from_millis(1)),
            poll_interval: Duration::from_millis(100),
        });

        let message = maybe_execute_automation_step(&mut app);

        assert!(message.is_none());
        assert!(
            app.background
                .automation
                .as_ref()
                .and_then(|runner| runner.due_at)
                .is_some()
        );
        assert_eq!(
            app.background
                .automation
                .as_ref()
                .map(|runner| runner.steps.len()),
            Some(1)
        );
    }

    #[test]
    fn automation_tick_selects_first_non_header_item_on_active_route() {
        let mut app = test_app();
        app.background.startup_ready = true;
        app.gallery_items = vec![
            BrowseItem {
                media_id: 10,
                title: "Header".to_owned(),
                thumbnail_path: None,
                media_kind: "image".to_owned(),
                metadata_line: String::new(),
                is_group_header: true,
                line: "2026-03-11".to_owned(),
                aspect_ratio: 1.0,
                group_image_count: None,
                group_video_count: None,
            },
            BrowseItem {
                media_id: 11,
                title: "Item".to_owned(),
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
        app.background.automation = Some(AutomationRunner {
            steps: VecDeque::from([AutomationStep {
                wait_ms: 0,
                action: AutomationAction::SelectFirstVisible,
            }]),
            due_at: Some(Instant::now() - Duration::from_millis(1)),
            poll_interval: Duration::from_millis(100),
        });

        let message = maybe_execute_automation_step(&mut app);

        assert!(matches!(message, Some(Message::SelectMedia(11))));
        assert!(
            app.background
                .automation
                .as_ref()
                .is_some_and(|runner| runner.steps.is_empty() && runner.due_at.is_none())
        );
    }

    #[test]
    fn windows_file_drop_payload_uses_dropfiles_header_and_double_nul() {
        let path = std::env::temp_dir().join("librapix").join("clip.png");
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
    fn start_snapshot_hydrate_sets_busy_activity_stage() {
        let mut app = test_app();

        let _ = start_snapshot_hydrate(&mut app);

        assert!(app.activity_progress.busy);
        assert_eq!(app.activity_status, "Loading library snapshot");
        assert_eq!(app.background.snapshot_generation, 1);
    }

    #[test]
    fn finalize_background_flow_only_sets_ready_when_idle() {
        let mut app = test_app();
        set_activity_stage(
            &mut app,
            TextKey::StageRefreshingGalleryLabel,
            String::new(),
            true,
        );
        app.background.projection_in_flight = true;

        let _ = finalize_background_flow(&mut app);
        assert!(app.activity_progress.busy);
        assert_ne!(app.activity_status, "Ready");

        app.background.projection_in_flight = false;
        let _ = finalize_background_flow(&mut app);
        assert!(!app.activity_progress.busy);
        assert_eq!(app.activity_status, "Ready");
    }

    #[test]
    fn finalize_background_flow_marks_ready_with_deferred_thumbnail_catchup() {
        let mut app = test_app();
        app.background.startup_ready = false;
        app.background.deferred_thumbnail_queue = VecDeque::from([ThumbnailWorkItem {
            generation: 1,
            media_id: 77,
            absolute_path: PathBuf::from("/tmp/deferred.png"),
            media_kind: "image".to_owned(),
            file_size_bytes: 10,
            modified_unix_seconds: Some(10),
        }]);

        let _ = finalize_background_flow(&mut app);

        assert!(app.background.startup_ready);
        assert!(!app.activity_progress.busy);
        assert_eq!(app.activity_status, "Ready");
        assert!(app.background.deferred_thumbnail_due_at.is_some());
    }

    #[test]
    fn split_startup_thumbnail_work_prioritizes_visible_media() {
        let mut app = test_app();
        app.background.startup_ready = false;
        app.gallery_items = vec![
            BrowseItem {
                media_id: 10,
                title: "visible-a.png".to_owned(),
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
                media_id: 20,
                title: "visible-b.png".to_owned(),
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

        let mut items = (100..=198)
            .map(|media_id| ThumbnailWorkItem {
                generation: 1,
                media_id,
                absolute_path: PathBuf::from(format!("/tmp/{media_id}.png")),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            })
            .collect::<Vec<_>>();
        items.insert(
            0,
            ThumbnailWorkItem {
                generation: 1,
                media_id: 30,
                absolute_path: PathBuf::from("/tmp/30.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            },
        );
        items.insert(
            1,
            ThumbnailWorkItem {
                generation: 1,
                media_id: 10,
                absolute_path: PathBuf::from("/tmp/10.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            },
        );
        items.insert(
            2,
            ThumbnailWorkItem {
                generation: 1,
                media_id: 40,
                absolute_path: PathBuf::from("/tmp/40.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            },
        );
        items.insert(
            3,
            ThumbnailWorkItem {
                generation: 1,
                media_id: 20,
                absolute_path: PathBuf::from("/tmp/20.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            },
        );

        let (immediate, deferred) = split_startup_thumbnail_work(&app, items);

        let immediate_ids = immediate
            .iter()
            .map(|item| item.media_id)
            .collect::<Vec<_>>();
        let deferred_ids = deferred
            .iter()
            .map(|item| item.media_id)
            .collect::<Vec<_>>();
        assert!(immediate_ids.contains(&10));
        assert!(immediate_ids.contains(&20));
        assert!(!deferred_ids.contains(&10));
        assert!(!deferred_ids.contains(&20));
        assert!(deferred_ids.contains(&30));
        assert!(deferred_ids.contains(&40));
    }

    #[test]
    fn split_startup_thumbnail_work_defers_visible_video_to_background() {
        let mut app = test_app();
        app.background.startup_ready = false;
        app.gallery_items = vec![BrowseItem {
            media_id: 10,
            title: "visible-video.mp4".to_owned(),
            thumbnail_path: None,
            media_kind: "video".to_owned(),
            metadata_line: String::new(),
            is_group_header: false,
            line: String::new(),
            aspect_ratio: 1.0,
            group_image_count: None,
            group_video_count: None,
        }];

        let (immediate, deferred) = split_startup_thumbnail_work(
            &app,
            vec![ThumbnailWorkItem {
                generation: 1,
                media_id: 10,
                absolute_path: PathBuf::from("/tmp/10.mp4"),
                media_kind: "video".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            }],
        );

        assert!(immediate.is_empty());
        assert_eq!(deferred.len(), 1);
        assert_eq!(deferred[0].media_id, 10);
    }

    #[test]
    fn thumbnail_lookup_reuses_exact_deterministic_gallery_file_without_catalog_row() {
        let thumbnails_dir = unique_temp_dir("exact-reuse");
        let row = catalog_row(1, thumbnails_dir.join("source.png"), "image");
        let gallery_path = thumbnail_path(
            &thumbnails_dir,
            &row.absolute_path,
            row.file_size_bytes,
            row.modified_unix_seconds,
            GALLERY_THUMB_SIZE,
        );
        write_thumbnail_stub(&gallery_path);

        let lookup = projection_lookup_for_rows(
            &thumbnails_dir,
            std::slice::from_ref(&row),
            &[],
            &[],
            Route::Gallery,
        );

        assert_eq!(
            lookup.resolved_paths.get(&row.media_id),
            Some(&gallery_path)
        );
        assert_eq!(lookup.summary.exact_deterministic_reused, 1);
        assert_eq!(lookup.summary.scheduled_generation, 0);
        assert_eq!(lookup.summary.priority_placeholder, 0);
    }

    #[test]
    fn thumbnail_lookup_reuses_exact_gallery_artifact_row() {
        let thumbnails_dir = unique_temp_dir("exact-catalog");
        let row = catalog_row(11, thumbnails_dir.join("source.png"), "image");
        let gallery_relative = PathBuf::from("gallery-exact.png");
        let gallery_path = thumbnails_dir.join(&gallery_relative);
        write_thumbnail_stub(&gallery_path);
        let gallery_artifact = DerivedArtifactRecord {
            media_id: row.media_id,
            artifact_kind: DerivedArtifactKind::Thumbnail,
            artifact_variant: GALLERY_THUMB_VARIANT.to_owned(),
            relative_path: Some(gallery_relative),
            status: DerivedArtifactStatus::Ready,
        };

        let lookup = projection_lookup_for_rows(
            &thumbnails_dir,
            std::slice::from_ref(&row),
            &[gallery_artifact],
            &[],
            Route::Gallery,
        );

        assert_eq!(
            lookup.resolved_paths.get(&row.media_id),
            Some(&gallery_path)
        );
        assert_eq!(lookup.summary.exact_catalog_reused, 1);
        assert_eq!(lookup.summary.scheduled_generation, 0);
    }

    #[test]
    fn thumbnail_lookup_uses_detail_artifact_as_compatible_fallback() {
        let thumbnails_dir = unique_temp_dir("fallback-reuse");
        let row = catalog_row(2, thumbnails_dir.join("source.png"), "image");
        let fallback_relative = PathBuf::from("detail-fallback.png");
        let fallback_path = thumbnails_dir.join(&fallback_relative);
        write_thumbnail_stub(&fallback_path);
        let detail_artifact = DerivedArtifactRecord {
            media_id: row.media_id,
            artifact_kind: DerivedArtifactKind::Thumbnail,
            artifact_variant: DETAIL_THUMB_VARIANT.to_owned(),
            relative_path: Some(fallback_relative),
            status: DerivedArtifactStatus::Ready,
        };

        let lookup = projection_lookup_for_rows(
            &thumbnails_dir,
            std::slice::from_ref(&row),
            &[],
            &[detail_artifact],
            Route::Gallery,
        );

        assert_eq!(
            lookup.resolved_paths.get(&row.media_id),
            Some(&fallback_path)
        );
        assert_eq!(lookup.summary.fallback_catalog_reused, 1);
        assert_eq!(lookup.summary.scheduled_generation, 0);
    }

    #[test]
    fn thumbnail_lookup_uses_placeholder_and_schedules_generation_when_unresolved() {
        let thumbnails_dir = unique_temp_dir("placeholder");
        let row = catalog_row(3, thumbnails_dir.join("source.png"), "image");

        let lookup = projection_lookup_for_rows(&thumbnails_dir, &[row], &[], &[], Route::Gallery);

        assert!(lookup.resolved_paths.is_empty());
        assert_eq!(lookup.summary.priority_placeholder, 1);
        assert_eq!(lookup.summary.scheduled_generation, 1);
    }

    #[test]
    fn thumbnail_lookup_rejects_missing_ready_artifact_file() {
        let thumbnails_dir = unique_temp_dir("artifact-reject");
        let row = catalog_row(4, thumbnails_dir.join("source.png"), "image");
        let gallery_artifact = DerivedArtifactRecord {
            media_id: row.media_id,
            artifact_kind: DerivedArtifactKind::Thumbnail,
            artifact_variant: GALLERY_THUMB_VARIANT.to_owned(),
            relative_path: Some(PathBuf::from("missing-gallery.png")),
            status: DerivedArtifactStatus::Ready,
        };

        let lookup = projection_lookup_for_rows(
            &thumbnails_dir,
            &[row],
            &[gallery_artifact],
            &[],
            Route::Gallery,
        );

        assert!(lookup.resolved_paths.is_empty());
        assert_eq!(lookup.summary.rejected_gallery_missing_file, 1);
        assert_eq!(lookup.summary.scheduled_generation, 1);
    }

    #[test]
    fn startup_projection_defers_non_visible_route_refresh() {
        let mut app = test_app();
        app.background.startup_ready = false;
        app.state.apply(AppMessage::OpenGallery);

        let _ = start_projection_refresh(&mut app, BackgroundWorkReason::UserOrSystem, "test");

        assert!(app.background.projection_in_flight);
        assert!(!app.background.startup_deferred_gallery_refresh);
        assert!(app.background.startup_deferred_timeline_refresh);
    }

    #[test]
    fn route_switch_projection_policy_stays_current_surface_after_startup() {
        let mut app = test_app();
        app.background.startup_ready = true;
        app.state.apply(AppMessage::OpenTimeline);

        assert_eq!(
            projection_refresh_policy(&app, BackgroundWorkReason::UserOrSystem, "route_switch"),
            ProjectionRefreshPolicy::CurrentSurface
        );
        assert_eq!(
            projection_refresh_policy(&app, BackgroundWorkReason::UserOrSystem, "filter_change"),
            ProjectionRefreshPolicy::CurrentSurface
        );
        assert_eq!(
            projection_refresh_policy(
                &app,
                BackgroundWorkReason::UserOrSystem,
                "startup_continuation",
            ),
            ProjectionRefreshPolicy::CurrentSurface
        );
        assert_eq!(
            projection_refresh_policy(&app, BackgroundWorkReason::FilesystemWatch, "system"),
            ProjectionRefreshPolicy::CurrentSurface
        );
        assert_eq!(
            projection_refresh_policy(&app, BackgroundWorkReason::UserOrSystem, "system"),
            ProjectionRefreshPolicy::Full
        );
    }

    #[test]
    fn timeline_visible_row_window_stays_bounded_for_large_sections() {
        let layouts = vec![
            JustifiedRowLayout {
                start: 0,
                end: 1,
                height: 100.0,
            };
            1_215
        ];
        let rows_top = 400_040.0;

        let window = compute_visible_row_window(
            &layouts,
            rows_top,
            450_911.2,
            452_029.2,
            GALLERY_GAP as f32,
        );

        assert!(window.visible_rows > 0);
        assert!(window.visible_rows < 40);
        assert!(window.start_row > 0);
        assert!(window.end_row < layouts.len());
    }

    #[test]
    fn timeline_visible_row_window_preserves_total_height_with_spacers() {
        let layouts = vec![
            JustifiedRowLayout {
                start: 0,
                end: 2,
                height: 140.0,
            },
            JustifiedRowLayout {
                start: 2,
                end: 4,
                height: 180.0,
            },
            JustifiedRowLayout {
                start: 4,
                end: 6,
                height: 160.0,
            },
        ];

        let window = compute_visible_row_window(&layouts, 48.0, 190.0, 430.0, GALLERY_GAP as f32);
        let total_height = layouts
            .iter()
            .enumerate()
            .map(|(index, layout)| {
                layout.height
                    + if index + 1 < layouts.len() {
                        GALLERY_GAP as f32
                    } else {
                        0.0
                    }
            })
            .sum::<f32>();
        let visible_height = layouts[window.start_row..window.end_row]
            .iter()
            .enumerate()
            .map(|(index, layout)| {
                layout.height
                    + if window.start_row + index + 1 < window.end_row {
                        GALLERY_GAP as f32
                    } else {
                        0.0
                    }
            })
            .sum::<f32>();

        assert_eq!(window.visible_rows, 2);
        assert!(
            (window.top_spacer + visible_height + window.bottom_spacer - total_height).abs() < 0.01
        );
    }

    #[test]
    fn viewport_snapshot_matches_coalesces_duplicate_updates() {
        let mut app = test_app();
        app.media_scroll_absolute_y = 120.0;
        app.media_scroll_max_y = 600.0;
        app.media_viewport_height = 720.0;

        assert!(viewport_snapshot_matches(&app, 120.2, 600.2, 720.2));
        assert!(!viewport_snapshot_matches(&app, 122.0, 600.0, 720.0));
    }

    #[test]
    fn media_virtual_overscan_tightens_during_active_drag() {
        assert_eq!(media_virtual_overscan_px(false), MEDIA_VIRTUAL_OVERSCAN_PX);
        assert_eq!(
            media_virtual_overscan_px(true),
            MEDIA_VIRTUAL_OVERSCAN_DRAG_PX
        );
        assert!(media_virtual_overscan_px(true) < media_virtual_overscan_px(false));
    }

    #[test]
    fn active_drag_stabilizes_gallery_layout_width_to_last_settled_layout() {
        let mut app = test_app();
        app.viewport_drag.active = true;
        app.layout_cache.borrow_mut().gallery = Some(Arc::new(CachedGalleryLayout {
            generation: app.browse_layout_generation,
            width_key: 438,
            item_count: 10,
            first_media_id: Some(1),
            last_media_id: Some(10),
            rows: Arc::from([]),
        }));

        let first = layout_width_for_surface(&app, MediaSurfaceKind::Gallery, 793.4);
        let second = layout_width_for_surface(&app, MediaSurfaceKind::Gallery, 1165.2);
        let preview = *app.drag_layout_preview.borrow();

        assert_eq!(first, 438.0);
        assert_eq!(second, 438.0);
        assert_eq!(preview.gallery.frozen_width_key, Some(438));
        assert_eq!(preview.gallery.last_measured_width_key, Some(1165));
        assert!(preview.gallery.width_change_count >= 1);
        assert!(preview.gallery.suppressed_rebuilds >= 1);
    }

    #[test]
    fn active_drag_without_cached_layout_uses_first_measured_width_until_settle() {
        let mut app = test_app();
        app.viewport_drag.active = true;

        let first = layout_width_for_surface(&app, MediaSurfaceKind::Timeline, 612.4);
        let second = layout_width_for_surface(&app, MediaSurfaceKind::Timeline, 944.9);

        assert_eq!(first, 612.0);
        assert_eq!(second, 612.0);
        assert_eq!(
            app.drag_layout_preview.borrow().timeline.frozen_width_key,
            Some(612)
        );
    }

    #[test]
    fn tiny_viewport_corrections_do_not_activate_drag_lifecycle() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;

        handle_media_viewport_changed(&mut app, 174_502.1, 174_502.1, 1_251.0);
        handle_media_viewport_changed(&mut app, 174_412.9, 174_412.9, 1_251.0);

        assert!(!app.viewport_drag.active);
        assert_eq!(app.viewport_drag.update_count, 0);
        assert_eq!(app.viewport_drag.candidate_event_count, 2);
    }

    #[test]
    fn sustained_scroll_burst_activates_drag_lifecycle() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;

        handle_media_viewport_changed(&mut app, 0.0, 174_502.2, 1_251.0);
        handle_media_viewport_changed(&mut app, 180.0, 174_502.2, 1_251.0);
        handle_media_viewport_changed(&mut app, 360.0, 174_502.2, 1_251.0);

        assert!(app.viewport_drag.active);
        assert_eq!(app.viewport_drag.update_count, 1);
        assert_eq!(app.viewport_drag.applied_updates, 1);
        handle_media_viewport_changed(&mut app, 540.0, 174_502.2, 1_251.0);
        assert_eq!(app.viewport_drag.applied_updates, 1);
    }

    #[test]
    fn large_jump_fast_path_activates_drag_lifecycle_with_two_events() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;

        handle_media_viewport_changed(&mut app, 0.0, 174_502.2, 1_251.0);
        handle_media_viewport_changed(&mut app, 340.0, 174_502.2, 1_251.0);

        assert!(app.viewport_drag.active);
        assert_eq!(app.viewport_drag.update_count, 1);
        assert_eq!(app.viewport_drag.mode, ViewportDragMode::SettleFirstPreview);
        assert_eq!(app.viewport_drag.applied_updates, 0);
        assert!(app.viewport_drag.max_step_delta_px >= 340.0);
    }

    #[test]
    fn active_drag_escalates_to_settle_first_when_step_delta_grows_large() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;

        handle_media_viewport_changed(&mut app, 0.0, 174_502.2, 1_251.0);
        handle_media_viewport_changed(&mut app, 140.0, 174_502.2, 1_251.0);
        handle_media_viewport_changed(&mut app, 260.0, 174_502.2, 1_251.0);
        assert_eq!(app.viewport_drag.mode, ViewportDragMode::LivePreview);

        handle_media_viewport_changed(&mut app, 4_800.0, 174_502.2, 1_251.0);

        assert!(app.viewport_drag.active);
        assert_eq!(app.viewport_drag.mode, ViewportDragMode::SettleFirstPreview);
        assert!(app.viewport_drag.max_step_delta_px >= VIEWPORT_DRAG_SETTLE_FIRST_DELTA_PX);
    }

    #[test]
    fn settle_first_mode_blocks_preview_apply_without_idle_gap() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.mode = ViewportDragMode::SettleFirstPreview;
        app.viewport_drag.last_event_at = Some(Instant::now());
        app.viewport_drag.pending_viewport = Some(ViewportSnapshot {
            absolute_y: 8_400.0,
            max_y: 17_200.0,
            viewport_height: 650.0,
        });

        let applied = maybe_apply_active_drag_snapshot(&mut app, Instant::now(), false, true);

        assert!(!applied);
        assert_eq!(app.viewport_drag.applied_updates, 0);
        assert!(app.viewport_drag.pending_viewport.is_some());
    }

    #[test]
    fn viewport_settle_tick_clears_active_drag_after_pause() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now() - Duration::from_millis(320));
        app.viewport_drag.last_event_at = Some(Instant::now() - VIEWPORT_SETTLE_DELAY);
        app.viewport_drag.update_count = 6;
        app.viewport_drag.applied_updates = 4;
        app.media_scroll_absolute_y = 8_000.0;
        app.media_scroll_max_y = 42_000.0;
        app.media_viewport_height = 700.0;
        app.drag_layout_preview.borrow_mut().gallery = DragSurfacePreviewState {
            frozen_width_key: Some(438),
            last_measured_width_key: Some(1165),
            width_change_count: 14,
            suppressed_rebuilds: 13,
            anomaly_logged: true,
        };

        settle_media_viewport_drag(&mut app);

        assert!(!app.viewport_drag.active);
        assert_eq!(app.viewport_drag.update_count, 0);
        assert_eq!(
            app.drag_layout_preview.borrow().gallery.frozen_width_key,
            None
        );
        assert_eq!(
            layout_width_for_surface(&app, MediaSurfaceKind::Gallery, 1165.2),
            1165.0
        );
    }

    #[test]
    fn viewport_settle_waits_for_longer_idle_gap() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now() - Duration::from_millis(450));
        app.viewport_drag.last_event_at = Some(Instant::now() - Duration::from_millis(200));

        settle_media_viewport_drag(&mut app);

        assert!(app.viewport_drag.active);
    }

    #[test]
    fn viewport_settle_large_jump_profile_requires_longer_idle_gap() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now() - Duration::from_millis(1_200));
        app.viewport_drag.last_event_at = Some(Instant::now() - Duration::from_millis(300));
        app.viewport_drag.max_step_delta_px = 8_000.0;

        settle_media_viewport_drag(&mut app);

        assert!(app.viewport_drag.active);
    }

    #[test]
    fn active_drag_replaces_stale_pending_targets() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now());
        app.media_viewport_height = 650.0;

        handle_media_viewport_changed(&mut app, 100.0, 10_000.0, 650.0);
        handle_media_viewport_changed(&mut app, 1_800.0, 10_000.0, 650.0);
        handle_media_viewport_changed(&mut app, 3_400.0, 10_000.0, 650.0);

        assert_eq!(app.viewport_drag.update_count, 3);
        assert!(app.viewport_drag.applied_updates < app.viewport_drag.update_count);
        assert!(app.viewport_drag.latest_replacements >= 1);
        assert!(app.viewport_drag.pending_viewport.is_some());
    }

    #[test]
    fn settle_applies_latest_pending_viewport_target() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now() - Duration::from_millis(900));
        app.viewport_drag.last_event_at = Some(Instant::now() - Duration::from_millis(700));
        app.viewport_drag.pending_viewport = Some(ViewportSnapshot {
            absolute_y: 8_400.0,
            max_y: 17_200.0,
            viewport_height: 650.0,
        });
        app.viewport_drag.last_applied_at = Some(Instant::now());
        app.media_scroll_absolute_y = 1_200.0;
        app.media_scroll_max_y = 17_300.0;
        app.media_viewport_height = 650.0;

        settle_media_viewport_drag(&mut app);

        assert!(!app.viewport_drag.active);
        assert!((app.media_scroll_absolute_y - 8_400.0).abs() < 0.1);
        assert!((app.media_scroll_max_y - 17_200.0).abs() < 0.1);
    }

    #[test]
    fn settle_tick_applies_pending_preview_before_idle_settle() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now() - Duration::from_millis(250));
        app.viewport_drag.last_event_at = Some(Instant::now() - Duration::from_millis(30));
        app.viewport_drag.pending_viewport = Some(ViewportSnapshot {
            absolute_y: 2_400.0,
            max_y: 9_100.0,
            viewport_height: 650.0,
        });

        settle_media_viewport_drag(&mut app);

        assert!(app.viewport_drag.active);
        assert_eq!(app.viewport_drag.applied_updates, 1);
        assert!((app.media_scroll_absolute_y - 2_400.0).abs() < 0.1);
    }

    #[test]
    fn active_drag_preview_freezes_max_y_until_final_settle() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.frozen_max_y = Some(5_000.0);
        app.viewport_drag.pending_viewport = Some(ViewportSnapshot {
            absolute_y: 6_200.0,
            max_y: 8_400.0,
            viewport_height: 650.0,
        });

        let now = Instant::now();
        let preview_applied = maybe_apply_active_drag_snapshot(&mut app, now, true, true);
        assert!(preview_applied);
        assert!((app.media_scroll_absolute_y - 5_000.0).abs() < 0.1);
        assert!((app.media_scroll_max_y - 5_000.0).abs() < 0.1);

        app.viewport_drag.pending_viewport = Some(ViewportSnapshot {
            absolute_y: 6_200.0,
            max_y: 8_400.0,
            viewport_height: 650.0,
        });
        let settle_applied = maybe_apply_active_drag_snapshot(&mut app, now, true, false);
        assert!(settle_applied);
        assert!((app.media_scroll_absolute_y - 6_200.0).abs() < 0.1);
        assert!((app.media_scroll_max_y - 8_400.0).abs() < 0.1);
    }

    #[test]
    fn active_drag_ignores_max_only_updates_until_settle() {
        let mut app = test_app();
        app.state.active_route = Route::Gallery;
        app.viewport_drag.active = true;
        app.viewport_drag.started_at = Some(Instant::now());
        app.media_scroll_absolute_y = 2_400.0;
        app.media_scroll_max_y = 9_100.0;
        app.media_viewport_height = 650.0;

        handle_media_viewport_changed(&mut app, 2_400.0, 9_300.0, 650.0);

        assert_eq!(app.viewport_drag.update_count, 0);
        assert_eq!(app.viewport_drag.max_y_preview_skips, 1);
        assert_eq!(app.viewport_drag.coalesced_updates, 1);
        assert!(app.viewport_drag.pending_viewport.is_some());
    }

    #[test]
    fn existing_detail_preview_prefers_existing_file_then_browse_fallback() {
        let thumbnails_dir = unique_temp_dir("detail-preview");
        let row = librapix_storage::MediaReadModel {
            media_id: 77,
            source_root_id: 1,
            absolute_path: thumbnails_dir.join("source.png"),
            media_kind: "image".to_owned(),
            file_size_bytes: 32,
            modified_unix_seconds: Some(100),
            width_px: Some(400),
            height_px: Some(300),
            metadata_status: librapix_storage::IndexedMetadataStatus::Ok,
            tags: Vec::new(),
        };
        let browse_path = thumbnails_dir.join("browse.png");
        let detail_path = thumbnail_path(
            &thumbnails_dir,
            &row.absolute_path,
            row.file_size_bytes,
            row.modified_unix_seconds,
            DETAIL_THUMB_SIZE,
        );

        let (fallback_preview, fallback_source) =
            resolve_existing_detail_preview_path(&thumbnails_dir, &row, Some(browse_path.clone()));
        assert_eq!(fallback_preview, Some(browse_path));
        assert_eq!(fallback_source, "browse_thumbnail");

        write_thumbnail_stub(&detail_path);
        let (detail_preview, detail_source) =
            resolve_existing_detail_preview_path(&thumbnails_dir, &row, None);
        assert_eq!(detail_preview, Some(detail_path));
        assert_eq!(detail_source, "detail_artifact");
    }

    #[test]
    fn open_timeline_requests_projection_when_startup_deferred_it() {
        let mut app = test_app();
        app.background.startup_ready = true;
        app.background.startup_deferred_timeline_refresh = true;

        let _ = update(&mut app, Message::OpenTimeline);

        assert!(app.background.projection_in_flight);
        assert!(matches!(app.state.active_route, Route::Timeline));
    }

    #[test]
    fn unchanged_startup_snapshot_skips_projection_refresh() {
        let mut app = test_app();
        app.background.snapshot_loaded = true;
        app.background.startup_ready = false;
        app.background.reconcile_generation = 1;
        app.background.reconcile_in_flight = true;
        app.gallery_items = vec![browse_item(1), browse_item(2)];
        set_activity_stage(
            &mut app,
            TextKey::StageRefreshingGalleryLabel,
            "Loading gallery...".to_owned(),
            true,
        );

        let _ = apply_scan_job_result(
            &mut app,
            ScanJobResult {
                generation: 1,
                reason: BackgroundWorkReason::UserOrSystem,
                roots: vec![],
                ignore_rules: vec![],
                root_count: 1,
                scanned_root_ids: vec![1],
                indexing_summary: Some(IndexingSummary {
                    scanned_roots: 1,
                    candidate_files: 5,
                    ignored_entries: 0,
                    unreadable_entries: 0,
                    new_files: 0,
                    changed_files: 0,
                    unchanged_files: 2,
                    missing_marked: 0,
                    read_model_count: 5,
                }),
                indexing_status: "Indexing complete".to_owned(),
                error: None,
            },
        );

        assert!(!app.background.projection_in_flight);
        assert!(app.background.startup_ready);
        assert!(app.background.startup_deferred_gallery_refresh);
        assert!(app.background.startup_deferred_timeline_refresh);
        assert!(app.background.startup_gallery_continuation_due_at.is_some());
        assert!(!app.activity_progress.busy);
        assert_eq!(app.activity_status, "Ready");
    }

    #[test]
    fn startup_gallery_continuation_kicks_off_current_surface_projection() {
        let mut app = test_app();
        app.background.startup_ready = true;
        app.background.startup_deferred_gallery_refresh = true;
        app.background.startup_gallery_continuation_due_at =
            Some(Instant::now() - Duration::from_millis(1));
        app.gallery_items = (1..=160).map(browse_item).collect();

        let _ = start_startup_gallery_continuation(&mut app);

        assert!(app.background.projection_in_flight);
        assert!(app.background.startup_gallery_continuation_due_at.is_none());
    }

    #[test]
    fn startup_projection_still_runs_when_reconcile_detects_changes() {
        let mut app = test_app();
        app.background.snapshot_loaded = true;
        app.background.startup_ready = false;
        app.background.reconcile_generation = 1;
        app.background.reconcile_in_flight = true;
        app.gallery_items = vec![browse_item(1), browse_item(2)];

        let _ = apply_scan_job_result(
            &mut app,
            ScanJobResult {
                generation: 1,
                reason: BackgroundWorkReason::UserOrSystem,
                roots: vec![],
                ignore_rules: vec![],
                root_count: 1,
                scanned_root_ids: vec![1],
                indexing_summary: Some(IndexingSummary {
                    scanned_roots: 1,
                    candidate_files: 3,
                    ignored_entries: 0,
                    unreadable_entries: 0,
                    new_files: 1,
                    changed_files: 0,
                    unchanged_files: 2,
                    missing_marked: 0,
                    read_model_count: 3,
                }),
                indexing_status: "Indexing complete".to_owned(),
                error: None,
            },
        );

        assert!(app.background.projection_in_flight);
        assert!(!app.background.startup_ready);
    }

    #[test]
    fn projection_result_with_thumbnail_work_marks_ready_and_stays_busy() {
        let mut app = test_app();
        app.background.projection_generation = 1;
        app.background.projection_in_flight = true;

        let result = ProjectionJobResult {
            generation: 1,
            reason: BackgroundWorkReason::UserOrSystem,
            gallery_items: vec![BrowseItem {
                media_id: 42,
                title: "shot.png".to_owned(),
                thumbnail_path: None,
                media_kind: "image".to_owned(),
                metadata_line: "Image".to_owned(),
                is_group_header: false,
                line: "/tmp/shot.png [image]".to_owned(),
                aspect_ratio: 1.5,
                group_image_count: None,
                group_video_count: None,
            }],
            gallery_preview_lines: vec!["/tmp/shot.png [image]".to_owned()],
            refreshed_gallery: true,
            media_cache: HashMap::from([(
                42,
                CachedDetails {
                    absolute_path: PathBuf::from("/tmp/shot.png"),
                    media_kind: "image".to_owned(),
                    file_size_bytes: 10,
                    modified_unix_seconds: Some(100),
                    width_px: Some(10),
                    height_px: Some(10),
                    detail_thumbnail_path: None,
                },
            )]),
            browse_status: "Gallery loaded".to_owned(),
            thumbnail_candidates: vec![ThumbnailWorkItem {
                generation: 1,
                media_id: 42,
                absolute_path: PathBuf::from("/tmp/shot.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(100),
            }],
            ..ProjectionJobResult::default()
        };

        let _ = apply_projection_job_result(&mut app, result);

        assert!(app.background.startup_ready);
        assert!(app.activity_progress.busy);
        assert_eq!(app.activity_status, "Generating thumbnails");
        assert_eq!(app.background.thumbnail_total, 1);
        assert!(app.background.thumbnail_in_flight);
    }

    #[test]
    fn startup_ready_does_not_wait_for_video_thumbnail_batch() {
        let mut app = test_app();
        app.background.projection_generation = 1;
        app.background.projection_in_flight = true;

        let result = ProjectionJobResult {
            generation: 1,
            reason: BackgroundWorkReason::UserOrSystem,
            gallery_items: vec![BrowseItem {
                media_id: 42,
                title: "clip.mp4".to_owned(),
                thumbnail_path: None,
                media_kind: "video".to_owned(),
                metadata_line: "Video".to_owned(),
                is_group_header: false,
                line: "/tmp/clip.mp4 [video]".to_owned(),
                aspect_ratio: 1.5,
                group_image_count: None,
                group_video_count: None,
            }],
            gallery_preview_lines: vec!["/tmp/clip.mp4 [video]".to_owned()],
            refreshed_gallery: true,
            media_cache: HashMap::from([(
                42,
                CachedDetails {
                    absolute_path: PathBuf::from("/tmp/clip.mp4"),
                    media_kind: "video".to_owned(),
                    file_size_bytes: 10,
                    modified_unix_seconds: Some(100),
                    width_px: Some(1920),
                    height_px: Some(1080),
                    detail_thumbnail_path: None,
                },
            )]),
            browse_status: "Gallery loaded".to_owned(),
            thumbnail_candidates: vec![ThumbnailWorkItem {
                generation: 1,
                media_id: 42,
                absolute_path: PathBuf::from("/tmp/clip.mp4"),
                media_kind: "video".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(100),
            }],
            ..ProjectionJobResult::default()
        };

        let _ = apply_projection_job_result(&mut app, result);

        assert!(app.background.startup_ready);
        assert!(!app.background.thumbnail_in_flight);
        assert_eq!(app.background.deferred_thumbnail_queue.len(), 1);
        assert!(app.background.deferred_thumbnail_due_at.is_some());
    }

    #[test]
    fn projection_refresh_cancels_background_thumbnail_work() {
        let mut app = test_app();
        app.background.startup_ready = true;
        app.background.thumbnail_in_flight = true;
        app.background.thumbnail_generation = 3;
        app.background.thumbnail_queue = VecDeque::from([ThumbnailWorkItem {
            generation: 3,
            media_id: 9,
            absolute_path: PathBuf::from("/tmp/9.png"),
            media_kind: "image".to_owned(),
            file_size_bytes: 10,
            modified_unix_seconds: Some(10),
        }]);

        let _ = request_projection_refresh(&mut app, BackgroundWorkReason::UserOrSystem);

        assert!(app.background.projection_in_flight);
        assert!(app.background.thumbnail_queue.is_empty());
        assert!(!app.background.thumbnail_in_flight);
        assert!(app.background.thumbnail_generation > 3);
    }

    #[test]
    fn video_ffmpeg_failure_disables_later_video_scheduling_for_session() {
        let mut app = test_app();
        app.background.thumbnail_generation = 2;
        app.background.thumbnail_in_flight = true;
        app.background.thumbnail_total = 2;
        app.background.thumbnail_queue = VecDeque::from([ThumbnailWorkItem {
            generation: 2,
            media_id: 88,
            absolute_path: PathBuf::from("/tmp/88.mp4"),
            media_kind: "video".to_owned(),
            file_size_bytes: 10,
            modified_unix_seconds: Some(10),
        }]);
        app.background.thumbnail_queued_ids = HashSet::from([77, 88]);

        let _ = apply_thumbnail_batch_result(
            &mut app,
            ThumbnailBatchResult {
                generation: 2,
                batch_id: 1,
                mode: ThumbnailWorkMode::BackgroundCatchUp,
                completed_media_ids: vec![77],
                failures: vec![ThumbnailFailureEvent {
                    media_id: 77,
                    media_kind: "video".to_owned(),
                    failure_class: ThumbnailFailureClass::VideoFfmpegNotFound,
                    detail: "ffmpeg missing".to_owned(),
                    command_line: Some("ffmpeg.exe -i /tmp/77.mp4".to_owned()),
                    ffmpeg_path: None,
                    exit_code: None,
                    stderr_summary: None,
                    timeout_ms: None,
                    hard_failure: true,
                    disable_video_for_session: true,
                }],
                failed: 1,
                ..ThumbnailBatchResult::default()
            },
        );

        assert_eq!(
            app.background.video_thumbnails_disabled_reason.as_deref(),
            Some("ffmpeg missing")
        );
        assert!(app.background.thumbnail_queue.is_empty());
        assert!(
            filter_thumbnail_candidates_for_runtime_policy(
                &mut app,
                vec![ThumbnailWorkItem {
                    generation: 3,
                    media_id: 99,
                    absolute_path: PathBuf::from("/tmp/99.mp4"),
                    media_kind: "video".to_owned(),
                    file_size_bytes: 10,
                    modified_unix_seconds: Some(10),
                }],
                "test",
            )
            .is_empty()
        );
    }

    #[test]
    fn failed_thumbnail_enters_backoff_and_is_not_rescheduled_immediately() {
        let mut app = test_app();
        app.background.thumbnail_generation = 4;
        app.background.thumbnail_in_flight = true;
        app.background.thumbnail_total = 1;
        app.background.thumbnail_queued_ids = HashSet::from([55]);

        let _ = apply_thumbnail_batch_result(
            &mut app,
            ThumbnailBatchResult {
                generation: 4,
                batch_id: 2,
                mode: ThumbnailWorkMode::BackgroundCatchUp,
                completed_media_ids: vec![55],
                failures: vec![ThumbnailFailureEvent {
                    media_id: 55,
                    media_kind: "image".to_owned(),
                    failure_class: ThumbnailFailureClass::ImageDecode,
                    detail: "invalid image".to_owned(),
                    command_line: None,
                    ffmpeg_path: None,
                    exit_code: None,
                    stderr_summary: None,
                    timeout_ms: None,
                    hard_failure: true,
                    disable_video_for_session: false,
                }],
                failed: 1,
                ..ThumbnailBatchResult::default()
            },
        );

        let filtered = filter_thumbnail_candidates_for_runtime_policy(
            &mut app,
            vec![ThumbnailWorkItem {
                generation: 5,
                media_id: 55,
                absolute_path: PathBuf::from("/tmp/55.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            }],
            "test",
        );
        assert!(filtered.is_empty());
        assert!(app.background.thumbnail_retry_state.contains_key(&55));
    }

    #[test]
    fn cancelled_thumbnail_batch_exits_before_processing_items() {
        let token = Arc::new(AtomicU64::new(9));
        let cancellation = ThumbnailCancellation::new(token.clone(), 8);
        let result = do_thumbnail_batch(ThumbnailBatchInput {
            generation: 8,
            batch_id: 3,
            mode: ThumbnailWorkMode::BackgroundCatchUp,
            database_file: PathBuf::from("/tmp/librapix-test.db"),
            thumbnails_dir: PathBuf::from("/tmp/librapix-thumbnails"),
            cancellation,
            items: vec![ThumbnailWorkItem {
                generation: 8,
                media_id: 1,
                absolute_path: PathBuf::from("/tmp/1.png"),
                media_kind: "image".to_owned(),
                file_size_bytes: 10,
                modified_unix_seconds: Some(10),
            }],
        });

        assert!(result.cancelled);
        assert_eq!(result.attempted, 0);
        assert!(result.completed_media_ids.is_empty());
    }

    fn browse_item(media_id: i64) -> BrowseItem {
        BrowseItem {
            media_id,
            title: format!("shot-{media_id}.png"),
            thumbnail_path: None,
            media_kind: "image".to_owned(),
            metadata_line: "Image".to_owned(),
            is_group_header: false,
            line: format!("/tmp/shot-{media_id}.png [image]"),
            aspect_ratio: 1.5,
            group_image_count: None,
            group_video_count: None,
        }
    }

    fn catalog_row(media_id: i64, absolute_path: PathBuf, media_kind: &str) -> CatalogMediaRecord {
        CatalogMediaRecord {
            media_id,
            source_root_id: 1,
            source_root_display_name: Some("Library".to_owned()),
            absolute_path: absolute_path.clone(),
            file_name: absolute_path
                .file_name()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| absolute_path.display().to_string()),
            file_extension: Some(if media_kind.eq_ignore_ascii_case("video") {
                "mp4".to_owned()
            } else {
                "png".to_owned()
            }),
            media_kind: media_kind.to_owned(),
            file_size_bytes: 32,
            modified_unix_seconds: Some(100),
            width_px: Some(400),
            height_px: Some(300),
            metadata_status: librapix_storage::IndexedMetadataStatus::Ok,
            search_text: String::new(),
            timeline_day_key: Some("1970-01-01".to_owned()),
            timeline_month_key: Some("1970-01".to_owned()),
            timeline_year_key: Some("1970".to_owned()),
            tags: Vec::new(),
        }
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "librapix-app-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temp dir should create");
        path
    }

    fn write_thumbnail_stub(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent dir should create");
        }
        std::fs::write(path, b"stub").expect("thumbnail stub should write");
    }

    fn projection_lookup_for_rows(
        thumbnails_dir: &Path,
        rows: &[CatalogMediaRecord],
        gallery_artifacts: &[DerivedArtifactRecord],
        detail_artifacts: &[DerivedArtifactRecord],
        active_route: Route,
    ) -> ProjectionThumbnailLookup {
        let row_lookup = rows
            .iter()
            .map(|row| (row.media_id, row))
            .collect::<HashMap<_, _>>();
        let gallery_items = rows
            .iter()
            .map(|row| browse_item_from_catalog_row(Translator::new(Locale::EnUs), row, None))
            .collect::<Vec<_>>();
        resolve_projection_thumbnail_lookup(ProjectionThumbnailLookupInput {
            generation: 1,
            all_rows: rows,
            row_lookup: &row_lookup,
            gallery_artifacts,
            detail_artifacts,
            thumbnails_dir,
            active_route,
            search_query: "",
            gallery_items: &gallery_items,
            timeline_items: &[],
            search_items: &[],
        })
    }

    #[derive(Serialize, Deserialize)]
    struct LegacySnapshotPayload {
        version: u32,
        gallery_items: Vec<BrowseItem>,
        timeline_items: Vec<BrowseItem>,
        available_filter_tags: Vec<String>,
        updated_unix_seconds: i64,
    }

    #[test]
    fn startup_snapshot_payload_is_capped_to_recent_gallery_slice() {
        let gallery_items = (1..=5_000).map(browse_item).collect::<Vec<_>>();
        let timeline_items = (10_001..=15_000).map(browse_item).collect::<Vec<_>>();

        let legacy_payload = serde_json::to_string(&LegacySnapshotPayload {
            version: 1,
            gallery_items: gallery_items.clone(),
            timeline_items,
            available_filter_tags: vec!["boss".to_owned()],
            updated_unix_seconds: 1,
        })
        .expect("legacy payload should serialize");
        let startup_payload =
            snapshot_payload_from_projection(&gallery_items, &["boss".to_owned()])
                .expect("startup payload should serialize");
        let legacy_parse_started_at = Instant::now();
        let legacy_snapshot = serde_json::from_str::<LegacySnapshotPayload>(&legacy_payload)
            .expect("legacy payload should deserialize");
        let legacy_parse_duration = legacy_parse_started_at.elapsed();
        let startup_parse_started_at = Instant::now();
        let startup_snapshot =
            serde_json::from_str::<PersistedProjectionSnapshot>(&startup_payload)
                .expect("startup payload should deserialize");
        let startup_parse_duration = startup_parse_started_at.elapsed();

        eprintln!(
            "legacy_bytes={} startup_bytes={} legacy_parse_ms={} startup_parse_ms={}",
            legacy_payload.len(),
            startup_payload.len(),
            legacy_parse_duration.as_millis(),
            startup_parse_duration.as_millis(),
        );

        assert_eq!(legacy_snapshot.gallery_items.len(), gallery_items.len());
        assert_eq!(
            startup_snapshot.gallery_items.len(),
            STARTUP_SNAPSHOT_GALLERY_LIMIT
        );
        assert_eq!(startup_snapshot.gallery_total_items, gallery_items.len());
        assert_eq!(
            startup_snapshot.available_filter_tags,
            vec!["boss".to_owned()]
        );
        assert!(startup_payload.len() < legacy_payload.len());
    }

    #[test]
    fn snapshot_apply_restores_only_gallery_slice() {
        let mut app = test_app();
        app.background.snapshot_generation = 1;
        let snapshot = PersistedProjectionSnapshot {
            version: PROJECTION_SNAPSHOT_VERSION,
            gallery_items: vec![browse_item(1), browse_item(2), browse_item(3)],
            gallery_total_items: 25,
            available_filter_tags: vec!["boss".to_owned()],
            updated_unix_seconds: 1,
        };

        let _ = begin_snapshot_apply(&mut app, 1, snapshot);
        let _ = apply_snapshot_chunk(&mut app);

        assert_eq!(app.gallery_items.len(), 3);
        assert!(app.timeline_items.is_empty());
        assert!(app.timeline_anchors.is_empty());
        assert_eq!(app.available_filter_tags, vec!["boss".to_owned()]);
        assert!(app.startup_metrics.first_usable_gallery_recorded);
    }

    #[test]
    fn make_short_button_visibility_helper_is_video_only() {
        let mut app = test_app();
        app.state.set_selected_media(Some(7));
        app.media_cache.insert(
            7,
            CachedDetails {
                absolute_path: PathBuf::from("/tmp/clip.mp4"),
                media_kind: "video".to_owned(),
                file_size_bytes: 1,
                modified_unix_seconds: None,
                width_px: None,
                height_px: None,
                detail_thumbnail_path: None,
            },
        );
        assert!(selected_media_is_video(&app));
        app.media_cache
            .get_mut(&7)
            .expect("cached media")
            .media_kind = "image".to_owned();
        assert!(!selected_media_is_video(&app));
    }

    #[test]
    fn smooth_effect_triggers_warning_helper() {
        let mut state = MakeShortDialogState::default();
        assert!(!make_short_has_smooth_warning(&state));
        state.effects.push(ShortEffect::Smooth);
        assert!(make_short_has_smooth_warning(&state));
    }
}
