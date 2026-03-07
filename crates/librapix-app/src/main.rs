mod format;
mod ui;

use chrono::Local;
use iced::keyboard;
use iced::keyboard::key;
use iced::widget::{
    Id, Space, button, column, container, image, operation, responsive, row, scrollable, stack,
    text, text_input, vertical_slider,
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
    IndexedMediaWrite, IndexedMetadataStatus, SourceRootLifecycle, Storage, TagKind,
};
use librapix_thumbnails::{ensure_image_thumbnail, ensure_video_thumbnail};
use notify::{EventKind, RecursiveMode, Watcher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
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
    RootInputChanged(String),
    SelectRoot(i64),
    AddRoot,
    UpdateRoot,
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
    DetachTag,
    OpenSelectedFile,
    OpenSelectedFolder,
    CopySelectedFile,
    CopySelectedPath,
    IgnoreRuleInputChanged(String),
    EnableIgnoreRule,
    DisableIgnoreRule,
    StartupRestore,
    BrowseFolder,
    FilesystemChanged,
    SetFilterMediaKind(Option<String>),
    SetFilterExtension(Option<String>),
    SetFilterTag(Option<String>),
    MinFileSizeInputChanged(String),
    ApplyMinFileSize,
    RootTagInputChanged(String),
    AddRootAppTag,
    AddRootGameTag,
    RemoveRootTag(String),
    TimelineScrubChanged(f32),
    TimelineScrubReleased,
    JumpToTimelineAnchor(usize),
    MediaViewportChanged { absolute_y: f32, max_y: f32 },
    KeyboardEvent(keyboard::Event),
    BackgroundWorkComplete(Box<BackgroundWorkResult>),
    OpenMediaById(i64),
    CopyMediaFileById(i64),
    DismissNewMediaAnnouncement,
    RefreshDiagnostics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BackgroundWorkReason {
    #[default]
    UserOrSystem,
    FilesystemWatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum BackgroundWorkMode {
    #[default]
    IndexAndProject,
    ProjectOnly,
}

#[derive(Debug, Clone, Default)]
struct BackgroundWorkResult {
    mode: BackgroundWorkMode,
    reason: BackgroundWorkReason,
    roots: Vec<LibraryRootView>,
    indexing_summary: Option<IndexingSummary>,
    thumbnail_status: String,
    indexing_status: String,
    gallery_items: Vec<BrowseItem>,
    timeline_items: Vec<BrowseItem>,
    search_items: Vec<BrowseItem>,
    timeline_anchors: Vec<TimelineAnchor>,
    gallery_preview_lines: Vec<String>,
    timeline_preview_lines: Vec<String>,
    search_preview_lines: Vec<String>,
    media_cache: std::collections::HashMap<i64, CachedDetails>,
    available_filter_tags: Vec<String>,
    ignore_rules_preview: Vec<String>,
    root_tags_preview: Vec<(String, String)>,
    browse_status: String,
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
    ignore_rule_input: String,
    ignore_rules_preview: Vec<String>,
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
    filter_media_kind: Option<String>,
    filter_extension: Option<String>,
    filter_tag: Option<String>,
    available_filter_tags: Vec<String>,
    min_file_size_bytes: u64,
    min_file_size_input: String,
    media_cache: std::collections::HashMap<i64, CachedDetails>,
    root_tag_input: String,
    root_tags_preview: Vec<(String, String)>,
    diagnostics_lines: Vec<String>,
    diagnostics_events: Vec<String>,
    show_diagnostics: bool,
    timeline_scrub_value: f32,
    timeline_scrubbing: bool,
    timeline_scrub_anchor_index: Option<usize>,
    timeline_scroll_max_y: f32,
    new_media_announcement: Option<NewMediaAnnouncement>,
}

#[derive(Debug, Clone)]
struct BrowseItem {
    media_id: i64,
    title: String,
    thumbnail_path: Option<PathBuf>,
    media_kind: String,
    metadata_line: String,
    is_group_header: bool,
    line: String,
    aspect_ratio: f32,
}

#[derive(Debug, Clone)]
struct CachedDetails {
    absolute_path: PathBuf,
    media_kind: String,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    width_px: Option<u32>,
    height_px: Option<u32>,
    tags: Vec<String>,
    detail_thumbnail_path: Option<PathBuf>,
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

#[derive(Debug, Clone)]
struct BackgroundWorkInput {
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    min_file_size_bytes: u64,
    filter_media_kind: Option<String>,
    filter_extension: Option<String>,
    filter_tag: Option<String>,
    search_query: String,
    active_route: Route,
    i18n: Translator,
    selected_root_id: Option<i64>,
    current_thumbnail_status: String,
    current_indexing_status: String,
    mode: BackgroundWorkMode,
    reason: BackgroundWorkReason,
}

const GALLERY_THUMB_SIZE: u32 = 400;
const DETAIL_THUMB_SIZE: u32 = 800;
const TARGET_ROW_HEIGHT: f32 = 200.0;
const MAX_ROW_HEIGHT: f32 = 350.0;
const MAX_DIAGNOSTICS_EVENTS: usize = 100;
const MEDIA_SCROLLABLE_ID: &str = "media-pane-scrollable";
const SCRUBBER_YEAR_MARKER_LIMIT: usize = 10;
const MEDIA_SCROLLBAR_SPACING: f32 = SPACE_XS as f32;

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
            ignore_rule_input: String::new(),
            ignore_rules_preview: Vec::new(),
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
            filter_media_kind: None,
            filter_extension: None,
            filter_tag: None,
            available_filter_tags: Vec::new(),
            min_file_size_bytes: 0,
            min_file_size_input: String::new(),
            media_cache: std::collections::HashMap::new(),
            root_tag_input: String::new(),
            root_tags_preview: Vec::new(),
            diagnostics_lines: Vec::new(),
            diagnostics_events: Vec::new(),
            show_diagnostics: true,
            timeline_scrub_value: 0.0,
            timeline_scrubbing: false,
            timeline_scrub_anchor_index: None,
            timeline_scroll_max_y: 0.0,
            new_media_announcement: None,
        };
        refresh_ignore_rules_preview(&mut app);
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

    let roots = app
        .state
        .library_roots
        .iter()
        .filter(|root| matches!(root.lifecycle, RootLifecycle::Active))
        .map(|root| root.normalized_path.clone())
        .collect::<Vec<_>>();

    if roots.is_empty() {
        keyboard_subscription
    } else {
        Subscription::batch(vec![
            keyboard_subscription,
            Subscription::run_with(WatchSubscriptionConfig { roots }, watch_filesystem),
        ])
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
        Message::RootInputChanged(v) => format!("RootInputChanged({})", v.len()),
        Message::SelectRoot(id) => format!("SelectRoot({id})"),
        Message::AddRoot => "AddRoot".into(),
        Message::UpdateRoot => "UpdateRoot".into(),
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
        Message::DetachTag => "DetachTag".into(),
        Message::OpenSelectedFile => "OpenSelectedFile".into(),
        Message::OpenSelectedFolder => "OpenSelectedFolder".into(),
        Message::CopySelectedFile => "CopySelectedFile".into(),
        Message::CopySelectedPath => "CopySelectedPath".into(),
        Message::IgnoreRuleInputChanged(v) => format!("IgnoreRuleInputChanged({})", v.len()),
        Message::EnableIgnoreRule => "EnableIgnoreRule".into(),
        Message::DisableIgnoreRule => "DisableIgnoreRule".into(),
        Message::StartupRestore => "StartupRestore".into(),
        Message::BrowseFolder => "BrowseFolder".into(),
        Message::FilesystemChanged => "FilesystemChanged".into(),
        Message::SetFilterMediaKind(k) => format!("SetFilterMediaKind({:?})", k.as_deref()),
        Message::SetFilterExtension(e) => format!("SetFilterExtension({:?})", e.as_deref()),
        Message::SetFilterTag(tag) => format!("SetFilterTag({:?})", tag.as_deref()),
        Message::MinFileSizeInputChanged(v) => format!("MinFileSizeInputChanged({})", v.len()),
        Message::ApplyMinFileSize => "ApplyMinFileSize".into(),
        Message::RootTagInputChanged(v) => format!("RootTagInputChanged({})", v.len()),
        Message::AddRootAppTag => "AddRootAppTag".into(),
        Message::AddRootGameTag => "AddRootGameTag".into(),
        Message::RemoveRootTag(n) => format!("RemoveRootTag({n})"),
        Message::TimelineScrubChanged(value) => format!("TimelineScrubChanged({value:.3})"),
        Message::TimelineScrubReleased => "TimelineScrubReleased".into(),
        Message::JumpToTimelineAnchor(index) => format!("JumpToTimelineAnchor({index})"),
        Message::MediaViewportChanged { absolute_y, max_y } => {
            format!("MediaViewportChanged({absolute_y:.1}/{max_y:.1})")
        }
        Message::KeyboardEvent(_) => "KeyboardEvent".into(),
        Message::BackgroundWorkComplete(_) => "BackgroundWorkComplete".into(),
        Message::OpenMediaById(id) => format!("OpenMediaById({id})"),
        Message::CopyMediaFileById(id) => format!("CopyMediaFileById({id})"),
        Message::DismissNewMediaAnnouncement => "DismissNewMediaAnnouncement".into(),
        Message::RefreshDiagnostics => "RefreshDiagnostics".into(),
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
        Message::RootInputChanged(value) => {
            app.state.apply(AppMessage::SetRootInput);
            app.state.set_root_input(value);
        }
        Message::SelectRoot(id) => {
            app.state.apply(AppMessage::SetSelectedRoot);
            app.state.set_selected_root(Some(id));
            refresh_root_tags_preview(app);
        }
        Message::AddRoot => {
            if let Some(path) = normalized_input_path(&app.state.root_input)
                && with_storage(&app.runtime, |storage| storage.upsert_source_root(&path)).is_ok()
            {
                persist_root_to_config(&app.runtime.config_file, &path);
                refresh_roots(app);
                app.state.clear_selection_and_input();
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
                app.activity_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
                return spawn_background_work(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    BackgroundWorkMode::IndexAndProject,
                );
            } else {
                app.root_status = app.i18n.text(TextKey::ErrorInvalidRootPathLabel).to_owned();
            }
        }
        Message::UpdateRoot => {
            if let (Some(id), Some(path)) = (
                app.state.selected_root_id,
                normalized_input_path(&app.state.root_input),
            ) && with_storage(&app.runtime, |storage| {
                storage.update_source_root_path(id, &path)
            })
            .is_ok()
            {
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            } else {
                app.root_status = app.i18n.text(TextKey::ErrorInvalidRootPathLabel).to_owned();
            }
        }
        Message::DeactivateRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| {
                    storage.set_source_root_lifecycle(id, SourceRootLifecycle::Deactivated)
                })
                .is_ok()
            {
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            }
        }
        Message::ReactivateRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| {
                    storage.set_source_root_lifecycle(id, SourceRootLifecycle::Active)
                })
                .is_ok()
            {
                refresh_roots(app);
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
            }
        }
        Message::RemoveRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| storage.remove_source_root(id)).is_ok()
            {
                refresh_roots(app);
                app.state.apply(AppMessage::ClearRootSelection);
                app.state.clear_selection_and_input();
                app.root_status = app.i18n.text(TextKey::RootActionSuccess).to_owned();
                app.activity_status = app.i18n.text(TextKey::LoadingGalleryLabel).to_owned();
                return spawn_background_work(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    BackgroundWorkMode::ProjectOnly,
                );
            }
        }
        Message::RefreshRoots => {
            refresh_roots(app);
        }
        Message::RunIndexing => {
            app.activity_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::IndexAndProject,
            );
        }
        Message::SearchQueryChanged(value) => {
            app.state.apply(AppMessage::SetSearchQuery);
            app.state.set_search_query(value);
        }
        Message::RunSearchQuery => {
            app.activity_status = app.i18n.text(TextKey::LoadingSearchLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::ProjectOnly,
            );
        }
        Message::RunTimelineProjection => {
            app.activity_status = app.i18n.text(TextKey::LoadingTimelineLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::ProjectOnly,
            );
        }
        Message::RunGalleryProjection => {
            app.activity_status = app.i18n.text(TextKey::LoadingGalleryLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::ProjectOnly,
            );
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
        Message::DetachTag => {
            detach_tag_from_selected_media(app);
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
        Message::EnableIgnoreRule => {
            set_ignore_rule_enabled(app, true);
        }
        Message::DisableIgnoreRule => {
            set_ignore_rule_enabled(app, false);
        }
        Message::StartupRestore => {
            if app.state.library_roots.is_empty() {
                return Task::none();
            }
            app.activity_status = app.i18n.text(TextKey::StatusRestoringLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::IndexAndProject,
            );
        }
        Message::BrowseFolder => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.state.apply(AppMessage::SetRootInput);
                app.state.set_root_input(path.display().to_string());
            }
        }
        Message::FilesystemChanged => {
            app.activity_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::FilesystemWatch,
                BackgroundWorkMode::IndexAndProject,
            );
        }
        Message::SetFilterMediaKind(kind) => {
            app.filter_media_kind = kind;
            app.filter_extension = None;
            app.activity_status = projection_loading_label(app).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::ProjectOnly,
            );
        }
        Message::SetFilterExtension(ext) => {
            app.filter_extension = ext;
            app.activity_status = projection_loading_label(app).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::ProjectOnly,
            );
        }
        Message::SetFilterTag(tag) => {
            app.filter_tag = tag;
            app.activity_status = projection_loading_label(app).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::ProjectOnly,
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
            app.activity_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
            return spawn_background_work(
                app,
                BackgroundWorkReason::UserOrSystem,
                BackgroundWorkMode::IndexAndProject,
            );
        }
        Message::RootTagInputChanged(value) => {
            app.root_tag_input = value;
        }
        Message::AddRootAppTag => {
            if add_root_tag(app, TagKind::App) {
                return spawn_background_work(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    BackgroundWorkMode::IndexAndProject,
                );
            }
        }
        Message::AddRootGameTag => {
            if add_root_tag(app, TagKind::Game) {
                return spawn_background_work(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    BackgroundWorkMode::IndexAndProject,
                );
            }
        }
        Message::RemoveRootTag(tag_name) => {
            if remove_root_tag(app, &tag_name) {
                return spawn_background_work(
                    app,
                    BackgroundWorkReason::UserOrSystem,
                    BackgroundWorkMode::IndexAndProject,
                );
            }
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
        Message::BackgroundWorkComplete(result) => {
            apply_background_result(app, *result);
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

fn view(app: &Librapix) -> Element<'_, Message> {
    let _required_rules = non_destructive::required_rules();
    let is_gallery = matches!(app.state.active_route, Route::Gallery);
    let is_timeline = matches!(app.state.active_route, Route::Timeline);

    // ── Header ──
    let brand = row![
        text("Libra").size(FONT_DISPLAY).color(TEXT_PRIMARY),
        text("Pix").size(FONT_DISPLAY).color(ACCENT),
    ]
    .spacing(0);

    let header = container(
        row![
            brand,
            text("\u{00B7} Media Library")
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
            Space::new().width(Length::Fill),
            text_input(
                app.i18n.text(TextKey::SearchInputLabel),
                &app.state.search_query
            )
            .on_input(Message::SearchQueryChanged)
            .on_submit(Message::RunSearchQuery)
            .width(Length::Fixed(400.0))
            .style(search_input_style),
            Space::new().width(Length::Fill),
            text(app.activity_status.clone())
                .size(FONT_CAPTION)
                .color(ACCENT),
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
        button(text(app.i18n.text(TextKey::GalleryTab)).size(FONT_BODY))
            .width(Length::Fill)
            .on_press(Message::OpenGallery)
            .style(nav_button_style(is_gallery))
            .padding([SPACE_SM as u16, SPACE_MD as u16]),
        button(text(app.i18n.text(TextKey::TimelineTab)).size(FONT_BODY))
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
                let path_name = root
                    .normalized_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.normalized_path.display().to_string());
                let status_color = match root.lifecycle {
                    RootLifecycle::Active => SUCCESS_COLOR,
                    RootLifecycle::Unavailable => WARNING_COLOR,
                    RootLifecycle::Deactivated => TEXT_DISABLED,
                };
                col.push(
                    button(
                        row![
                            text("\u{25CF}").size(FONT_CAPTION).color(status_color),
                            text(path_name).size(FONT_BODY).color(if is_selected {
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
                )
            })
            .into()
    };

    let selected_root_actions: Element<'_, Message> = if app.state.selected_root_id.is_some() {
        column![
            row![
                button(text(app.i18n.text(TextKey::RootUpdateButton)).size(FONT_CAPTION))
                    .on_press(Message::UpdateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                button(text(app.i18n.text(TextKey::RootDeactivateButton)).size(FONT_CAPTION))
                    .on_press(Message::DeactivateRoot)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            ]
            .spacing(SPACE_XS),
            row![
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
        .spacing(SPACE_XS)
        .into()
    } else {
        column![].into()
    };

    let library_section = column![
        section_heading(app.i18n.text(TextKey::LibrarySectionLabel)),
        roots_list,
        row![
            button(text(app.i18n.text(TextKey::BrowseFolderButton)).size(FONT_BODY))
                .on_press(Message::BrowseFolder)
                .style(primary_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::RootAddButton)).size(FONT_BODY))
                .on_press(Message::AddRoot)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::RootRefreshButton)).size(FONT_BODY))
                .on_press(Message::RefreshRoots)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .spacing(SPACE_XS),
        text_input(
            app.i18n.text(TextKey::FolderPathPlaceholder),
            &app.state.root_input
        )
        .on_input(Message::RootInputChanged)
        .style(field_input_style),
        selected_root_actions,
        text(app.root_status.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ]
    .spacing(SPACE_SM);

    // ── Sidebar: Indexing ──
    let indexing_section = column![
        section_heading(app.i18n.text(TextKey::IndexingSectionLabel)),
        button(text(app.i18n.text(TextKey::IndexRunButton)).size(FONT_BODY))
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

    // ── Sidebar: Exclusion rules ──
    let ignore_section = column![
        section_heading(app.i18n.text(TextKey::IgnoreRuleInputLabel)),
        text_input("*.tmp, **/cache/**", &app.ignore_rule_input)
            .on_input(Message::IgnoreRuleInputChanged)
            .style(field_input_style),
        row![
            button(text(app.i18n.text(TextKey::IgnoreRuleAddButton)).size(FONT_CAPTION))
                .on_press(Message::EnableIgnoreRule)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            button(text(app.i18n.text(TextKey::IgnoreRuleDisableButton)).size(FONT_CAPTION))
                .on_press(Message::DisableIgnoreRule)
                .style(subtle_button_style)
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        ]
        .spacing(SPACE_XS),
        app.ignore_rules_preview
            .iter()
            .take(6)
            .fold(column![].spacing(SPACE_2XS), |col, rule| {
                col.push(text(rule.clone()).size(FONT_CAPTION).color(TEXT_TERTIARY))
            }),
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

    // ── Sidebar: Root auto-tags ──
    let auto_tag_section: Element<'_, Message> = if app.state.selected_root_id.is_some() {
        let tag_list =
            app.root_tags_preview
                .iter()
                .fold(column![].spacing(SPACE_2XS), |col, (name, kind)| {
                    col.push(
                        row![
                            text(format!("{name} ({kind})"))
                                .size(FONT_CAPTION)
                                .color(TEXT_SECONDARY),
                            Space::new().width(Length::Fill),
                            button(
                                text(app.i18n.text(TextKey::RootTagRemoveButton))
                                    .size(FONT_CAPTION)
                            )
                            .on_press(Message::RemoveRootTag(name.clone()))
                            .style(subtle_button_style)
                            .padding([SPACE_2XS as u16, SPACE_XS as u16]),
                        ]
                        .spacing(SPACE_XS)
                        .align_y(iced::Alignment::Center),
                    )
                });

        column![
            section_heading(app.i18n.text(TextKey::RootTagsSectionLabel)),
            text_input(
                app.i18n.text(TextKey::RootTagInputPlaceholder),
                &app.root_tag_input,
            )
            .on_input(Message::RootTagInputChanged)
            .style(field_input_style),
            row![
                button(text(app.i18n.text(TextKey::RootTagAddButton)).size(FONT_CAPTION))
                    .on_press(Message::AddRootAppTag)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                button(text(app.i18n.text(TextKey::RootTagGameButton)).size(FONT_CAPTION))
                    .on_press(Message::AddRootGameTag)
                    .style(subtle_button_style)
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            ]
            .spacing(SPACE_XS),
            tag_list,
        ]
        .spacing(SPACE_SM)
        .into()
    } else {
        column![].into()
    };

    let diagnostics_section: Element<'_, Message> = if app.show_diagnostics {
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
        .into()
    } else {
        column![].into()
    };

    let sidebar = container(
        scrollable(
            column![
                nav_section,
                h_divider(),
                library_section,
                h_divider(),
                indexing_section,
                h_divider(),
                ignore_section,
                h_divider(),
                auto_tag_section,
                h_divider(),
                diagnostics_section,
            ]
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
        container(scrollable(details_content).height(Length::Fill))
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

    if app.new_media_announcement.is_some() {
        stack([shell, render_new_media_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        shell
    }
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
        button(text(app.i18n.text(TextKey::RefreshButton)).size(FONT_BODY))
            .on_press(run_msg)
            .style(subtle_button_style)
            .padding([SPACE_XS as u16, SPACE_MD as u16]),
        text(format!(
            "{}: {} \u{00B7} {}: {} \u{00B7} {}: {}",
            app.i18n.text(TextKey::StatsShownLabel),
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

    let type_chips = row![
        button(text(app.i18n.text(TextKey::FilterAllLabel)).size(FONT_CAPTION))
            .on_press(Message::SetFilterMediaKind(None))
            .style(filter_chip_style(app.filter_media_kind.is_none()))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        button(text(app.i18n.text(TextKey::FilterImagesLabel)).size(FONT_CAPTION))
            .on_press(Message::SetFilterMediaKind(Some("image".to_owned())))
            .style(filter_chip_style(
                app.filter_media_kind.as_deref() == Some("image"),
            ))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        button(text(app.i18n.text(TextKey::FilterVideosLabel)).size(FONT_CAPTION))
            .on_press(Message::SetFilterMediaKind(Some("video".to_owned())))
            .style(filter_chip_style(
                app.filter_media_kind.as_deref() == Some("video"),
            ))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
    ]
    .spacing(SPACE_XS);

    let ext_list: &[&str] = match app.filter_media_kind.as_deref() {
        Some("image") => &["png", "jpg", "gif", "webp"],
        Some("video") => &["mp4", "mov", "mkv", "webm", "avi"],
        _ => &["png", "jpg", "gif", "webp", "mp4", "mov", "mkv", "webm"],
    };
    let mut ext_chips = row![
        button(text(app.i18n.text(TextKey::FilterAllLabel)).size(FONT_CAPTION))
            .on_press(Message::SetFilterExtension(None))
            .style(filter_chip_style(app.filter_extension.is_none()))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
    ]
    .spacing(SPACE_XS);
    for ext in ext_list {
        let is_active = app.filter_extension.as_deref() == Some(ext);
        ext_chips = ext_chips.push(
            button(text(ext.to_uppercase()).size(FONT_CAPTION))
                .on_press(Message::SetFilterExtension(Some((*ext).to_owned())))
                .style(filter_chip_style(is_active))
                .padding([SPACE_2XS as u16, SPACE_SM as u16]),
        );
    }

    let mut tag_chip_row = row![
        text(app.i18n.text(TextKey::FilterTagsLabel))
            .size(FONT_CAPTION)
            .color(TEXT_SECONDARY),
        button(text(app.i18n.text(TextKey::FilterAllLabel)).size(FONT_CAPTION))
            .on_press(Message::SetFilterTag(None))
            .style(filter_chip_style(app.filter_tag.is_none()))
            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
    ]
    .spacing(SPACE_XS)
    .align_y(iced::Alignment::Center);

    if app.available_filter_tags.is_empty() {
        tag_chip_row = tag_chip_row.push(
            text(app.i18n.text(TextKey::FilterNoTagsLabel))
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        );
    } else {
        for tag in &app.available_filter_tags {
            let active = app
                .filter_tag
                .as_ref()
                .is_some_and(|selected| selected == tag);
            tag_chip_row = tag_chip_row.push(
                button(text(tag.as_str()).size(FONT_CAPTION))
                    .on_press(Message::SetFilterTag(Some(tag.clone())))
                    .style(filter_chip_style(active))
                    .padding([SPACE_2XS as u16, SPACE_SM as u16]),
            );
        }
    }

    let filter_row = row![type_chips, Space::new().width(SPACE_LG), ext_chips,]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center);

    let tag_filter_row: Element<'_, Message> = scrollable(tag_chip_row)
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .width(Length::Fill)
        .height(Length::Shrink)
        .into();

    let header: Element<'_, Message> = column![content_header, filter_row, tag_filter_row]
        .spacing(SPACE_SM)
        .into();

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
                render_justified_gallery(&app.search_items, app.state.selected_media_id),
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
            Route::Gallery => render_justified_gallery(browse_items, app.state.selected_media_id),
            Route::Timeline => render_timeline_view(browse_items, app.state.selected_media_id),
        }
    };

    let scrollable_content: Element<'_, Message> = column![search_section, browse_content]
        .spacing(SPACE_LG)
        .into();

    (header, scrollable_content)
}

fn render_justified_gallery<'a>(
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
                let card = render_media_card(item, selected_id == Some(item.media_id), row_height);
                row_widget = row_widget.push(container(card).width(Length::FillPortion(portion)));
            }
            grid = grid.push(row_widget);
            row_start = row_end;
        }

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

    let kind_badge = container(
        text(media_kind_icon_symbol(&item.media_kind))
            .size(FONT_CAPTION)
            .color(TEXT_PRIMARY),
    )
    .padding([SPACE_2XS as u16, SPACE_XS as u16])
    .style(media_kind_badge_style(
        item.media_kind.eq_ignore_ascii_case("video"),
    ));

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

fn media_kind_icon_symbol(media_kind: &str) -> &'static str {
    if media_kind.eq_ignore_ascii_case("video") {
        "\u{25B6}"
    } else {
        "\u{25A3}"
    }
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

            let group_header = container(
                text(header_item.title.clone())
                    .size(FONT_SUBTITLE)
                    .color(TEXT_PRIMARY),
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
                        let card =
                            render_media_card(item, selected_id == Some(item.media_id), row_height);
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
            .width(Length::Fixed(88.0))
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
    let chip_position = active_anchor
        .map(|anchor| anchor.normalized_position)
        .unwrap_or(app.timeline_scrub_value)
        .clamp(0.0, 1.0);

    let chip_track: Element<'_, Message> = if app.timeline_scrubbing {
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
        Space::new().width(Length::Fixed(0.0)).into()
    };

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
        .width(Length::Fixed(128.0))
        .height(Length::Fill)
        .padding([SPACE_SM as u16, SPACE_XS as u16])
        .style(scrubber_panel_style)
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

    let preview: Element<'_, Message> = if let Some(path) = &announcement.preview_path {
        container(
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fixed(220.0))
                .content_fit(ContentFit::Contain),
        )
        .style(card_style)
        .into()
    } else {
        container(
            text(announcement.title.as_str())
                .size(FONT_CAPTION)
                .color(TEXT_TERTIARY),
        )
        .height(Length::Fixed(180.0))
        .center_y(Length::Shrink)
        .style(thumb_placeholder_style)
        .into()
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

    let actions = responsive(move |size: Size| {
        let primary_padding = [SPACE_XS as u16, SPACE_MD as u16];
        let subtle_padding = [SPACE_XS as u16, SPACE_MD as u16];

        let view = button(text(app.i18n.text(TextKey::MediaSelectButton)).size(FONT_BODY))
            .on_press(Message::SelectMedia(announcement.media_id))
            .width(Length::Fill)
            .style(subtle_button_style)
            .padding(subtle_padding);
        let open = button(text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY))
            .on_press(Message::OpenMediaById(announcement.media_id))
            .width(Length::Fill)
            .style(action_button_style)
            .padding(primary_padding);
        let copy_file = button(text(app.i18n.text(TextKey::DetailsCopyFileButton)).size(FONT_BODY))
            .on_press(Message::CopyMediaFileById(announcement.media_id))
            .width(Length::Fill)
            .style(action_button_style)
            .padding(primary_padding);
        let dismiss = button(text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY))
            .on_press(Message::DismissNewMediaAnnouncement)
            .width(Length::Fill)
            .style(subtle_button_style)
            .padding(subtle_padding);

        if size.width < 460.0 {
            column![
                row![view, open].spacing(SPACE_XS),
                row![copy_file, dismiss].spacing(SPACE_XS),
            ]
            .spacing(SPACE_XS)
            .into()
        } else {
            row![view, open, copy_file, dismiss]
                .spacing(SPACE_XS)
                .into()
        }
    });

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

    let dialog = container(
        scrollable(dialog_content)
            .direction(scrollable::Direction::Vertical(
                scrollable::Scrollbar::default(),
            ))
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .max_width(640.0)
    .max_height(640.0)
    .padding(SPACE_LG as u16)
    .style(modal_dialog_style);

    container(
        container(dialog)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding([SPACE_2XL as u16, SPACE_XL as u16])
    .style(modal_backdrop_style)
    .into()
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
            .style(field_input_style),
            row![
                button(text(app.i18n.text(TextKey::DetailsAttachTagButton)).size(FONT_BODY))
                    .on_press(Message::AttachAppTag)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsAttachGameTagButton)).size(FONT_BODY))
                    .on_press(Message::AttachGameTag)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsDetachTagButton)).size(FONT_BODY))
                    .on_press(Message::DetachTag)
                    .style(subtle_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
            ]
            .spacing(SPACE_XS),
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
        let open = button(text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY))
            .on_press(Message::OpenSelectedFile)
            .width(Length::Fill)
            .style(action_button_style)
            .padding([SPACE_XS as u16, SPACE_MD as u16]);
        let open_folder =
            button(text(app.i18n.text(TextKey::DetailsOpenFolderButton)).size(FONT_BODY))
                .on_press(Message::OpenSelectedFolder)
                .width(Length::Fill)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]);
        let copy_file = button(text(app.i18n.text(TextKey::DetailsCopyFileButton)).size(FONT_BODY))
            .on_press(Message::CopySelectedFile)
            .width(Length::Fill)
            .style(action_button_style)
            .padding([SPACE_XS as u16, SPACE_MD as u16]);
        let copy_path = button(text(app.i18n.text(TextKey::DetailsCopyPathButton)).size(FONT_BODY))
            .on_press(Message::CopySelectedPath)
            .width(Length::Fill)
            .style(subtle_button_style)
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

    for source in &loaded.config.library_source_roots {
        let _ = storage.upsert_source_root(&source.path);
    }
    let _ = storage.ensure_default_ignore_rules();
    let _ = storage.reconcile_source_root_availability();

    runtime.roots = storage
        .list_source_roots()
        .map_or_else(|_| Vec::new(), map_roots_from_storage);
    runtime
}

fn persist_root_to_config(config_file: &Path, path: &Path) {
    let Ok(mut config) = load_from_path(config_file) else {
        return;
    };
    let path_buf = path.to_path_buf();
    if config
        .library_source_roots
        .iter()
        .any(|r| r.path == path_buf)
    {
        return;
    }
    config
        .library_source_roots
        .push(librapix_config::LibrarySourceRoot { path: path_buf });
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

fn refresh_roots(app: &mut Librapix) {
    let roots = with_storage(&app.runtime, |storage| {
        storage.reconcile_source_root_availability()?;
        storage.list_source_roots()
    })
    .map(map_roots_from_storage)
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(roots);
    refresh_ignore_rules_preview(app);
}

fn add_root_tag(app: &mut Librapix, kind: TagKind) -> bool {
    let Some(root_id) = app.state.selected_root_id else {
        return false;
    };
    let tag = app.root_tag_input.trim().to_owned();
    if tag.is_empty() {
        return false;
    }
    let _ = with_storage(&app.runtime, |storage| {
        storage.upsert_source_root_tag(root_id, &tag, kind)
    });
    app.root_tag_input.clear();
    refresh_root_tags_preview(app);
    true
}

fn remove_root_tag(app: &mut Librapix, tag_name: &str) -> bool {
    let Some(root_id) = app.state.selected_root_id else {
        return false;
    };
    let _ = with_storage(&app.runtime, |storage| {
        storage.remove_source_root_tag(root_id, tag_name)
    });
    refresh_root_tags_preview(app);
    true
}

fn refresh_root_tags_preview(app: &mut Librapix) {
    let Some(root_id) = app.state.selected_root_id else {
        app.root_tags_preview.clear();
        return;
    };
    let tags = with_storage(&app.runtime, |storage| {
        storage.list_source_root_tags(root_id)
    })
    .map(|rows| {
        rows.into_iter()
            .map(|r| (r.tag_name, r.tag_kind.as_str().to_owned()))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    app.root_tags_preview = tags;
}

fn refresh_ignore_rules_preview(app: &mut Librapix) {
    let rows = with_storage(&app.runtime, |storage| storage.list_ignore_rules("global"))
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    let status = if row.is_enabled {
                        app.i18n.text(TextKey::IgnoreRuleEnabled)
                    } else {
                        app.i18n.text(TextKey::IgnoreRuleDisabled)
                    };
                    format!("{} ({status})", row.pattern)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    app.ignore_rules_preview = rows;
}

fn set_ignore_rule_enabled(app: &mut Librapix, is_enabled: bool) {
    let pattern = app.ignore_rule_input.trim();
    if pattern.is_empty() {
        return;
    }
    let _ = with_storage(&app.runtime, |storage| {
        storage.upsert_ignore_rule("global", pattern, is_enabled)
    });
    refresh_ignore_rules_preview(app);
}

fn load_media_details(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
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
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
    } else {
        app.details_lines.clear();
        app.details_preview_path = None;
        app.details_title.clear();
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
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
}

fn detach_tag_from_selected_media(app: &mut Librapix) {
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
        storage.detach_tag_name_from_media(media_id, tag)
    }) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
            load_media_details(app);
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        }
    }
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
        Command::new("cmd")
            .args(["/C", "start", "", &path.display().to_string()])
            .status()?;
        Ok(())
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
    if row.media_kind == "image" {
        ensure_image_thumbnail(
            thumbnails_dir,
            &row.absolute_path,
            row.file_size_bytes,
            row.modified_unix_seconds,
            max_edge,
        )
        .ok()
        .map(|o| o.thumbnail_path)
    } else if row.media_kind == "video" {
        ensure_video_thumbnail(
            thumbnails_dir,
            &row.absolute_path,
            row.file_size_bytes,
            row.modified_unix_seconds,
            max_edge,
        )
        .ok()
        .map(|o| o.thumbnail_path)
    } else {
        None
    }
}

fn populate_media_cache(
    cache: &mut std::collections::HashMap<i64, CachedDetails>,
    rows: &[librapix_storage::MediaReadModel],
    thumbnails_dir: &std::path::Path,
) {
    cache.clear();
    for row in rows {
        let detail_thumbnail_path = resolve_thumbnail(thumbnails_dir, row, DETAIL_THUMB_SIZE);
        cache.insert(
            row.media_id,
            CachedDetails {
                absolute_path: row.absolute_path.clone(),
                media_kind: row.media_kind.clone(),
                file_size_bytes: row.file_size_bytes,
                modified_unix_seconds: row.modified_unix_seconds,
                width_px: row.width_px,
                height_px: row.height_px,
                tags: row.tags.clone(),
                detail_thumbnail_path,
            },
        );
    }
}

fn load_media_details_cached(app: &mut Librapix) {
    let Some(media_id) = app.state.selected_media_id else {
        app.details_action_status = app.i18n.text(TextKey::DetailsNoSelectionLabel).to_owned();
        app.details_preview_path = None;
        app.details_title.clear();
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
        if !cached.tags.is_empty() {
            app.details_lines.push(format!(
                "{}: {}",
                app.i18n.text(TextKey::DetailsTagsSectionLabel),
                cached.tags.join(", ")
            ));
        }
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

fn spawn_background_work(
    app: &Librapix,
    reason: BackgroundWorkReason,
    mode: BackgroundWorkMode,
) -> Task<Message> {
    let input = BackgroundWorkInput {
        database_file: app.runtime.database_file.clone(),
        thumbnails_dir: app.runtime.thumbnails_dir.clone(),
        min_file_size_bytes: app.min_file_size_bytes,
        filter_media_kind: app.filter_media_kind.clone(),
        filter_extension: app.filter_extension.clone(),
        filter_tag: app.filter_tag.clone(),
        search_query: app.state.search_query.clone(),
        active_route: app.state.active_route,
        i18n: app.i18n,
        selected_root_id: app.state.selected_root_id,
        current_thumbnail_status: app.thumbnail_status.clone(),
        current_indexing_status: app.indexing_status.clone(),
        mode,
        reason,
    };

    Task::perform(async move { do_background_work(input) }, |result| {
        Message::BackgroundWorkComplete(Box::new(result))
    })
}

fn do_background_work(input: BackgroundWorkInput) -> BackgroundWorkResult {
    let BackgroundWorkInput {
        database_file,
        thumbnails_dir,
        min_file_size_bytes,
        filter_media_kind,
        filter_extension,
        filter_tag,
        search_query,
        active_route,
        i18n,
        selected_root_id,
        current_thumbnail_status,
        current_indexing_status,
        mode,
        reason,
    } = input;

    let mut out = BackgroundWorkResult {
        mode,
        reason,
        thumbnail_status: current_thumbnail_status,
        indexing_status: current_indexing_status,
        ..Default::default()
    };

    let mut storage = match Storage::open(&database_file) {
        Ok(s) => s,
        Err(_) => {
            out.indexing_status = i18n.text(TextKey::ErrorIndexingFailedLabel).to_owned();
            return out;
        }
    };

    let _ = storage.reconcile_source_root_availability();
    let _ = storage.ensure_default_ignore_rules();

    out.ignore_rules_preview = storage
        .list_ignore_rules("global")
        .map(|rows| {
            rows.into_iter()
                .map(|row| {
                    let status = if row.is_enabled {
                        i18n.text(TextKey::IgnoreRuleEnabled)
                    } else {
                        i18n.text(TextKey::IgnoreRuleDisabled)
                    };
                    format!("{} ({status})", row.pattern)
                })
                .collect()
        })
        .unwrap_or_default();

    if let Some(root_id) = selected_root_id {
        out.root_tags_preview = storage
            .list_source_root_tags(root_id)
            .map(|rows| {
                rows.into_iter()
                    .map(|r| (r.tag_name, r.tag_kind.as_str().to_owned()))
                    .collect()
            })
            .unwrap_or_default();
    }

    let eligible_roots = storage.list_eligible_source_roots().unwrap_or_default();
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

    if matches!(mode, BackgroundWorkMode::IndexAndProject) {
        let indexing_summary = (|| -> Option<IndexingSummary> {
            let ignore = IgnoreEngine::new(&patterns).ok()?;
            let root_ids: Vec<i64> = roots_for_scan.iter().map(|r| r.source_root_id).collect();
            let existing = storage
                .list_existing_indexed_media_snapshots(&root_ids)
                .ok()?;

            let existing_for_indexer: Vec<librapix_indexer::ExistingIndexedEntry> = existing
                .into_iter()
                .map(|entry| librapix_indexer::ExistingIndexedEntry {
                    source_root_id: entry.source_root_id,
                    absolute_path: entry.absolute_path,
                    file_size_bytes: entry.file_size_bytes,
                    modified_unix_seconds: entry.modified_unix_seconds,
                    width_px: entry.width_px,
                    height_px: entry.height_px,
                })
                .collect();

            let scan_options = ScanOptions {
                min_file_size_bytes,
            };
            let result = scan_roots(
                &roots_for_scan,
                &ignore,
                &existing_for_indexer,
                &scan_options,
            );

            let writes: Vec<IndexedMediaWrite> = result
                .candidates
                .iter()
                .map(|c| IndexedMediaWrite {
                    source_root_id: c.source_root_id,
                    absolute_path: c.absolute_path.clone(),
                    media_kind: c.media_kind.as_str().to_owned(),
                    file_size_bytes: c.file_size_bytes,
                    modified_unix_seconds: c.modified_unix_seconds,
                    width_px: c.width_px,
                    height_px: c.height_px,
                    metadata_status: match c.metadata_status {
                        librapix_indexer::MetadataStatus::Ok => IndexedMetadataStatus::Ok,
                        librapix_indexer::MetadataStatus::Partial => IndexedMetadataStatus::Partial,
                        librapix_indexer::MetadataStatus::Unreadable => {
                            IndexedMetadataStatus::Unreadable
                        }
                    },
                })
                .collect();

            let apply_summary = storage
                .apply_incremental_index(&writes, &result.scanned_root_ids)
                .ok()?;
            let _ = storage.ensure_media_kind_tags_attached();
            let _ = storage.ensure_root_tags_exist();
            let _ = storage.apply_root_auto_tags();

            let read_models = storage.list_all_media_read_models().ok()?;

            let mut generated = 0usize;
            let mut reused = 0usize;
            let mut failed = 0usize;
            for row in &read_models {
                let thumb = if row.media_kind == "image" {
                    ensure_image_thumbnail(
                        &thumbnails_dir,
                        &row.absolute_path,
                        row.file_size_bytes,
                        row.modified_unix_seconds,
                        GALLERY_THUMB_SIZE,
                    )
                } else if row.media_kind == "video" {
                    ensure_video_thumbnail(
                        &thumbnails_dir,
                        &row.absolute_path,
                        row.file_size_bytes,
                        row.modified_unix_seconds,
                        GALLERY_THUMB_SIZE,
                    )
                } else {
                    continue;
                };
                match thumb {
                    Ok(o) if o.generated => generated += 1,
                    Ok(_) => reused += 1,
                    Err(_) => failed += 1,
                }
            }

            out.thumbnail_status = format!(
                "{}: {}={generated}, {}={reused}, {}={failed}",
                i18n.text(TextKey::ThumbnailStatusLabel),
                i18n.text(TextKey::ThumbnailGeneratedLabel),
                i18n.text(TextKey::ThumbnailReusedLabel),
                i18n.text(TextKey::ThumbnailFailedLabel),
            );

            Some(IndexingSummary {
                scanned_roots: result.summary.scanned_roots,
                candidate_files: result.summary.candidate_files,
                ignored_entries: result.summary.ignored_entries,
                unreadable_entries: result.summary.unreadable_entries,
                new_files: result.summary.new_files,
                changed_files: result.summary.changed_files,
                unchanged_files: result.summary.unchanged_files,
                missing_marked: apply_summary.missing_marked_count,
                read_model_count: read_models.len(),
            })
        })();

        if indexing_summary.is_some() {
            out.indexing_summary = indexing_summary;
            out.indexing_status = i18n.text(TextKey::IndexingCompletedLabel).to_owned();
        } else {
            out.indexing_status = i18n.text(TextKey::ErrorIndexingFailedLabel).to_owned();
        }
    }

    let _ = storage.reconcile_source_root_availability();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();

    let all_rows = storage.list_all_media_read_models().unwrap_or_default();
    out.available_filter_tags = collect_available_filter_tags(&all_rows);
    let active_tag_filter = filter_tag.as_ref().filter(|selected| {
        out.available_filter_tags
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(selected))
    });
    let row_lookup = all_rows
        .iter()
        .map(|row| (row.media_id, row))
        .collect::<std::collections::HashMap<_, _>>();

    let media = rows_to_projection_media(&all_rows);

    let gallery_query = GalleryQuery {
        media_kind: filter_media_kind.clone(),
        extension: filter_extension.clone(),
        tag: active_tag_filter.cloned(),
        sort: GallerySort::ModifiedDesc,
        limit: all_rows.len(),
        offset: 0,
    };
    out.gallery_items = project_gallery(&media, &gallery_query)
        .into_iter()
        .map(|item| {
            let matched_row = row_lookup.get(&item.media_id).copied();
            if let Some(row) = matched_row {
                browse_item_from_row(i18n, &thumbnails_dir, row)
            } else {
                let original = PathBuf::from(&item.absolute_path);
                BrowseItem {
                    media_id: item.media_id,
                    title: original
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| original.display().to_string()),
                    thumbnail_path: None,
                    media_kind: item.media_kind.clone(),
                    metadata_line: build_card_metadata_line(
                        i18n,
                        &item.media_kind,
                        None,
                        None,
                        None,
                    ),
                    is_group_header: false,
                    line: format!("{} [{}]", original.display(), item.media_kind),
                    aspect_ratio: 1.5,
                }
            }
        })
        .collect();

    out.gallery_preview_lines = out.gallery_items.iter().map(|i| i.line.clone()).collect();

    let filtered_media: Vec<ProjectionMedia> = media
        .into_iter()
        .filter(|m| {
            filter_media_kind
                .as_ref()
                .is_none_or(|k| m.media_kind.eq_ignore_ascii_case(k))
        })
        .filter(|m| {
            filter_extension.as_ref().is_none_or(|ext| {
                m.absolute_path
                    .rsplit('.')
                    .next()
                    .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            })
        })
        .filter(|m| projection_matches_tag_filter(m, active_tag_filter.map(|tag| tag.as_str())))
        .collect();

    let buckets = project_timeline(&filtered_media, TimelineGranularity::Day);
    out.timeline_anchors = build_timeline_anchors(&buckets);
    let mut timeline_lines = Vec::new();
    let mut timeline_items = Vec::new();
    for bucket in buckets {
        timeline_lines.push(format!("{} ({})", bucket.label, bucket.item_count));
        timeline_items.push(BrowseItem {
            media_id: 0,
            title: bucket.label.clone(),
            thumbnail_path: None,
            media_kind: String::new(),
            metadata_line: String::new(),
            is_group_header: true,
            line: bucket.label.clone(),
            aspect_ratio: 1.5,
        });
        for tl_item in bucket.items {
            let matched_row = row_lookup.get(&tl_item.media_id).copied();
            if let Some(row) = matched_row {
                let mut item = browse_item_from_row(i18n, &thumbnails_dir, row);
                item.line = format!("{} [{}]", tl_item.absolute_path, tl_item.media_kind);
                timeline_items.push(item);
            } else {
                timeline_items.push(BrowseItem {
                    media_id: tl_item.media_id,
                    title: PathBuf::from(&tl_item.absolute_path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or(tl_item.absolute_path.clone()),
                    thumbnail_path: None,
                    media_kind: tl_item.media_kind.clone(),
                    metadata_line: build_card_metadata_line(
                        i18n,
                        &tl_item.media_kind,
                        None,
                        None,
                        None,
                    ),
                    is_group_header: false,
                    line: format!("{} [{}]", tl_item.absolute_path, tl_item.media_kind),
                    aspect_ratio: 1.5,
                });
            }
        }
    }
    out.timeline_items = timeline_items;
    out.timeline_preview_lines = timeline_lines;

    if !search_query.trim().is_empty() {
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
                text: search_query.clone(),
                limit: all_rows.len(),
            },
        );
        out.search_items = hits
            .into_iter()
            .filter_map(|hit| row_lookup.get(&hit.media_id).copied().map(|row| (hit, row)))
            .filter(|(_, row)| {
                filter_media_kind
                    .as_ref()
                    .is_none_or(|k| row.media_kind.eq_ignore_ascii_case(k))
            })
            .filter(|(_, row)| {
                filter_extension.as_ref().is_none_or(|ext| {
                    row.absolute_path
                        .extension()
                        .and_then(|e| e.to_str())
                        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
                })
            })
            .filter(|(_, row)| {
                row_matches_tag_filter(row, active_tag_filter.map(|tag| tag.as_str()))
            })
            .map(|(_hit, row)| browse_item_from_row(i18n, &thumbnails_dir, row))
            .collect::<Vec<_>>();
        out.search_preview_lines = out
            .search_items
            .iter()
            .map(|item| item.line.clone())
            .collect();
    }

    populate_media_cache(&mut out.media_cache, &all_rows, &thumbnails_dir);

    if search_query.trim().is_empty() {
        out.browse_status = if matches!(active_route, Route::Timeline) {
            i18n.text(TextKey::TimelineCompletedLabel).to_owned()
        } else {
            i18n.text(TextKey::GalleryCompletedLabel).to_owned()
        };
    } else {
        out.browse_status = i18n.text(TextKey::SearchCompletedLabel).to_owned();
    }
    out
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
        "filter: kind={:?}, ext={:?}, tag={:?}",
        app.filter_media_kind.as_deref().unwrap_or("all"),
        app.filter_extension.as_deref().unwrap_or("all"),
        app.filter_tag.as_deref().unwrap_or("all")
    ));
    lines.push(format!("min file size: {} bytes", app.min_file_size_bytes));
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

fn apply_background_result(app: &mut Librapix, result: BackgroundWorkResult) {
    let previous_media_ids = app
        .media_cache
        .keys()
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let announcement = if matches!(result.reason, BackgroundWorkReason::FilesystemWatch) {
        build_new_media_announcement(app.i18n, &previous_media_ids, &result.media_cache)
    } else {
        None
    };

    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);

    if let Some(summary) = result.indexing_summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
    }

    app.thumbnail_status = result.thumbnail_status;
    app.indexing_status = result.indexing_status;

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
    app.available_filter_tags = result.available_filter_tags;
    if app.filter_tag.as_ref().is_some_and(|tag| {
        !app.available_filter_tags
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(tag))
    }) {
        app.filter_tag = None;
    }
    app.browse_status = result.browse_status;
    app.ignore_rules_preview = result.ignore_rules_preview;
    app.root_tags_preview = result.root_tags_preview;
    if matches!(result.mode, BackgroundWorkMode::IndexAndProject)
        && app.state.search_query.trim().is_empty()
    {
        app.search_items.clear();
    }
    if let Some(announcement) = announcement {
        app.new_media_announcement = Some(announcement);
    }
    app.activity_status.clear();
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
                tags: Vec::new(),
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
                tags: Vec::new(),
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
                tags: Vec::new(),
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
}
