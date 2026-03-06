mod format;
mod ui;

use chrono::Local;
use iced::widget::{
    Space, button, column, container, image, responsive, row, scrollable, text, text_input,
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
use librapix_projections::timeline::{TimelineGranularity, project_timeline};
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
    CopySelectedPath,
    IgnoreRuleInputChanged(String),
    EnableIgnoreRule,
    DisableIgnoreRule,
    StartupRestore,
    BrowseFolder,
    FilesystemChanged,
    SetFilterMediaKind(Option<String>),
    SetFilterExtension(Option<String>),
    MinFileSizeInputChanged(String),
    ApplyMinFileSize,
    RootTagInputChanged(String),
    AddRootAppTag,
    AddRootGameTag,
    RemoveRootTag(String),
    BackgroundWorkComplete(Box<BackgroundWorkResult>),
    RefreshDiagnostics,
}

#[derive(Debug, Clone, Default)]
struct BackgroundWorkResult {
    roots: Vec<LibraryRootView>,
    indexing_summary: Option<IndexingSummary>,
    thumbnail_status: String,
    indexing_status: String,
    gallery_items: Vec<BrowseItem>,
    timeline_items: Vec<BrowseItem>,
    gallery_preview_lines: Vec<String>,
    timeline_preview_lines: Vec<String>,
    media_cache: std::collections::HashMap<i64, CachedDetails>,
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
    search_items: Vec<BrowseItem>,
    indexing_status: String,
    browse_status: String,
    root_status: String,
    last_click_media_id: Option<i64>,
    last_click_time: Option<Instant>,
    activity_status: String,
    filter_media_kind: Option<String>,
    filter_extension: Option<String>,
    min_file_size_bytes: u64,
    min_file_size_input: String,
    media_cache: std::collections::HashMap<i64, CachedDetails>,
    root_tag_input: String,
    root_tags_preview: Vec<(String, String)>,
    diagnostics_lines: Vec<String>,
    diagnostics_events: Vec<String>,
    show_diagnostics: bool,
}

#[derive(Debug, Clone)]
struct BrowseItem {
    media_id: i64,
    title: String,
    subtitle: String,
    thumbnail_path: Option<PathBuf>,
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

const GALLERY_THUMB_SIZE: u32 = 400;
const DETAIL_THUMB_SIZE: u32 = 800;
const TARGET_ROW_HEIGHT: f32 = 200.0;
const MAX_ROW_HEIGHT: f32 = 350.0;
const MAX_DIAGNOSTICS_EVENTS: usize = 100;

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
            search_items: Vec::new(),
            indexing_status: String::new(),
            browse_status: String::new(),
            root_status: String::new(),
            last_click_media_id: None,
            last_click_time: None,
            activity_status: String::new(),
            filter_media_kind: None,
            filter_extension: None,
            min_file_size_bytes: 0,
            min_file_size_input: String::new(),
            media_cache: std::collections::HashMap::new(),
            root_tag_input: String::new(),
            root_tags_preview: Vec::new(),
            diagnostics_lines: Vec::new(),
            diagnostics_events: Vec::new(),
            show_diagnostics: true,
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
    let roots = app
        .state
        .library_roots
        .iter()
        .filter(|root| matches!(root.lifecycle, RootLifecycle::Active))
        .map(|root| root.normalized_path.clone())
        .collect::<Vec<_>>();

    if roots.is_empty() {
        Subscription::none()
    } else {
        Subscription::run_with(WatchSubscriptionConfig { roots }, watch_filesystem)
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
        Message::CopySelectedPath => "CopySelectedPath".into(),
        Message::IgnoreRuleInputChanged(v) => format!("IgnoreRuleInputChanged({})", v.len()),
        Message::EnableIgnoreRule => "EnableIgnoreRule".into(),
        Message::DisableIgnoreRule => "DisableIgnoreRule".into(),
        Message::StartupRestore => "StartupRestore".into(),
        Message::BrowseFolder => "BrowseFolder".into(),
        Message::FilesystemChanged => "FilesystemChanged".into(),
        Message::SetFilterMediaKind(k) => format!("SetFilterMediaKind({:?})", k.as_deref()),
        Message::SetFilterExtension(e) => format!("SetFilterExtension({:?})", e.as_deref()),
        Message::MinFileSizeInputChanged(v) => format!("MinFileSizeInputChanged({})", v.len()),
        Message::ApplyMinFileSize => "ApplyMinFileSize".into(),
        Message::RootTagInputChanged(v) => format!("RootTagInputChanged({})", v.len()),
        Message::AddRootAppTag => "AddRootAppTag".into(),
        Message::AddRootGameTag => "AddRootGameTag".into(),
        Message::RemoveRootTag(n) => format!("RemoveRootTag({n})"),
        Message::BackgroundWorkComplete(_) => "BackgroundWorkComplete".into(),
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
        }
        Message::OpenTimeline => {
            app.state.apply(AppMessage::OpenTimeline);
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
                return spawn_background_work(app);
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
                run_gallery_projection(app);
            }
        }
        Message::RefreshRoots => {
            refresh_roots(app);
        }
        Message::RunIndexing => {
            app.activity_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
            return spawn_background_work(app);
        }
        Message::SearchQueryChanged(value) => {
            app.state.apply(AppMessage::SetSearchQuery);
            app.state.set_search_query(value);
        }
        Message::RunSearchQuery => {
            run_read_model_query(app);
        }
        Message::RunTimelineProjection => {
            run_timeline_projection(app);
        }
        Message::RunGalleryProjection => {
            run_gallery_projection(app);
        }
        Message::SelectMedia(media_id) => {
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
            return spawn_background_work(app);
        }
        Message::BrowseFolder => {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                app.state.apply(AppMessage::SetRootInput);
                app.state.set_root_input(path.display().to_string());
            }
        }
        Message::FilesystemChanged => {
            app.activity_status = app.i18n.text(TextKey::LoadingIndexingLabel).to_owned();
            return spawn_background_work(app);
        }
        Message::SetFilterMediaKind(kind) => {
            app.filter_media_kind = kind;
            app.filter_extension = None;
            run_gallery_projection(app);
            run_timeline_projection(app);
            if !app.state.search_query.trim().is_empty() {
                run_read_model_query(app);
            }
        }
        Message::SetFilterExtension(ext) => {
            app.filter_extension = ext;
            run_gallery_projection(app);
            run_timeline_projection(app);
            if !app.state.search_query.trim().is_empty() {
                run_read_model_query(app);
            }
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
            return spawn_background_work(app);
        }
        Message::RootTagInputChanged(value) => {
            app.root_tag_input = value;
        }
        Message::AddRootAppTag => {
            if add_root_tag(app, TagKind::App) {
                return spawn_background_work(app);
            }
        }
        Message::AddRootGameTag => {
            if add_root_tag(app, TagKind::Game) {
                return spawn_background_work(app);
            }
        }
        Message::RemoveRootTag(tag_name) => {
            if remove_root_tag(app, &tag_name) {
                return spawn_background_work(app);
            }
        }
        Message::BackgroundWorkComplete(result) => {
            apply_background_result(app, *result);
            refresh_diagnostics(app);
        }
        Message::RefreshDiagnostics => {
            refresh_diagnostics(app);
        }
    }

    Task::none()
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
    let (media_header, media_scrollable) = render_media_panel(app);

    // ── Details pane ──
    let details_content = render_details_panel(app);

    // ── Body ──
    let body = row![
        sidebar,
        container(
            column![
                media_header,
                scrollable(media_scrollable).height(Length::Fill),
            ]
            .spacing(SPACE_SM),
        )
        .padding(SPACE_LG as u16)
        .width(Length::Fill),
        container(scrollable(details_content).height(Length::Fill))
            .width(Length::Fixed(DETAILS_WIDTH))
            .padding(SPACE_LG as u16)
            .style(details_pane_style),
    ]
    .height(Length::Fill);

    container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app_bg_style)
        .into()
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
    let item_count = browse_items.iter().filter(|i| !i.is_group_header).count();

    let content_header = row![
        text(route_title).size(FONT_TITLE).color(TEXT_PRIMARY),
        Space::new().width(Length::Fill),
        button(text(app.i18n.text(TextKey::RefreshButton)).size(FONT_BODY))
            .on_press(run_msg)
            .style(subtle_button_style)
            .padding([SPACE_XS as u16, SPACE_MD as u16]),
        text(format!(
            "{item_count} {}",
            app.i18n.text(TextKey::ItemsLabel)
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

    let filter_bar = row![type_chips, Space::new().width(SPACE_LG), ext_chips,]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center);

    let header: Element<'_, Message> = column![content_header, filter_bar].spacing(SPACE_SM).into();

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

    let card_content = column![
        thumb,
        text(item.subtitle.clone())
            .size(FONT_CAPTION)
            .color(TEXT_TERTIARY),
    ];

    button(card_content)
        .width(Length::Fill)
        .on_press(Message::SelectMedia(item.media_id))
        .style(card_button_style(selected))
        .padding(0)
        .into()
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
            row![
                button(text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY))
                    .on_press(Message::OpenSelectedFile)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsOpenFolderButton)).size(FONT_BODY))
                    .on_press(Message::OpenSelectedFolder)
                    .style(action_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
                button(text(app.i18n.text(TextKey::DetailsCopyPathButton)).size(FONT_BODY))
                    .on_press(Message::CopySelectedPath)
                    .style(subtle_button_style)
                    .padding([SPACE_XS as u16, SPACE_MD as u16]),
            ]
            .spacing(SPACE_XS),
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

fn run_read_model_query(app: &mut Librapix) {
    app.browse_status = app.i18n.text(TextKey::LoadingSearchLabel).to_owned();
    let query = app.state.search_query.clone();
    let rows = with_storage(&app.runtime, |storage| storage.list_all_media_read_models())
        .map(|rows| {
            let docs = rows
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
                    text: query.clone(),
                    limit: 20,
                },
            );

            hits.into_iter()
                .filter_map(|hit| {
                    rows.iter()
                        .find(|row| row.media_id == hit.media_id)
                        .map(|row| (hit, row))
                })
                .filter(|(_, row)| {
                    app.filter_media_kind
                        .as_ref()
                        .is_none_or(|k| row.media_kind.eq_ignore_ascii_case(k))
                })
                .filter(|(_, row)| {
                    app.filter_extension.as_ref().is_none_or(|ext| {
                        row.absolute_path
                            .extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
                    })
                })
                .map(|(_hit, row)| BrowseItem {
                    media_id: row.media_id,
                    title: row
                        .absolute_path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| row.absolute_path.display().to_string()),
                    subtitle: if row.tags.is_empty() {
                        row.media_kind.clone()
                    } else {
                        format!("{} \u{00B7} {}", row.media_kind, row.tags.join(", "))
                    },
                    thumbnail_path: resolve_thumbnail(
                        &app.runtime.thumbnails_dir,
                        row,
                        GALLERY_THUMB_SIZE,
                    ),
                    is_group_header: false,
                    line: format!("{} | {}", row.absolute_path.display(), row.media_kind),
                    aspect_ratio: aspect_ratio_from(row.width_px, row.height_px),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    app.search_items = rows;
    app.browse_status = app.i18n.text(TextKey::SearchCompletedLabel).to_owned();
}

fn run_timeline_projection(app: &mut Librapix) {
    app.browse_status = app.i18n.text(TextKey::LoadingTimelineLabel).to_owned();
    let rows = with_storage(&app.runtime, |storage| storage.list_all_media_read_models())
        .map(|rows| {
            let media = rows_to_projection_media(&rows);
            let filtered: Vec<ProjectionMedia> = media
                .into_iter()
                .filter(|m| {
                    app.filter_media_kind
                        .as_ref()
                        .is_none_or(|k| m.media_kind.eq_ignore_ascii_case(k))
                })
                .filter(|m| {
                    app.filter_extension.as_ref().is_none_or(|ext| {
                        m.absolute_path
                            .rsplit('.')
                            .next()
                            .is_some_and(|e| e.eq_ignore_ascii_case(ext))
                    })
                })
                .collect();
            let buckets = project_timeline(&filtered, TimelineGranularity::Day);
            let mut lines = Vec::new();
            let mut items = Vec::new();
            for bucket in buckets {
                lines.push(format!("{} ({})", bucket.label, bucket.item_count));
                items.push(BrowseItem {
                    media_id: 0,
                    title: bucket.label.clone(),
                    subtitle: String::new(),
                    thumbnail_path: None,
                    is_group_header: true,
                    line: bucket.label.clone(),
                    aspect_ratio: 1.5,
                });
                for tl_item in bucket.items {
                    let matched_row = rows.iter().find(|r| r.media_id == tl_item.media_id);
                    let thumbnail_path = matched_row.and_then(|row| {
                        resolve_thumbnail(&app.runtime.thumbnails_dir, row, GALLERY_THUMB_SIZE)
                    });
                    let ar = matched_row
                        .map(|r| aspect_ratio_from(r.width_px, r.height_px))
                        .unwrap_or(1.5);
                    items.push(BrowseItem {
                        media_id: tl_item.media_id,
                        title: PathBuf::from(&tl_item.absolute_path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or(tl_item.absolute_path.clone()),
                        subtitle: tl_item.media_kind.clone(),
                        thumbnail_path,
                        is_group_header: false,
                        line: format!("{} [{}]", tl_item.absolute_path, tl_item.media_kind),
                        aspect_ratio: ar,
                    });
                }
            }
            populate_media_cache(&mut app.media_cache, &rows, &app.runtime.thumbnails_dir);
            (lines, items)
        })
        .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state.replace_timeline_preview(rows.0);
    app.timeline_items = rows.1;
    app.browse_status = app.i18n.text(TextKey::TimelineCompletedLabel).to_owned();
}

fn run_gallery_projection(app: &mut Librapix) {
    app.browse_status = app.i18n.text(TextKey::LoadingGalleryLabel).to_owned();
    let rows = with_storage(&app.runtime, |storage| storage.list_all_media_read_models())
        .map(|rows| {
            let media = rows_to_projection_media(&rows);
            let query = GalleryQuery {
                media_kind: app.filter_media_kind.clone(),
                extension: app.filter_extension.clone(),
                tag: None,
                sort: GallerySort::ModifiedDesc,
                limit: rows.len(),
                offset: 0,
            };
            let items: Vec<BrowseItem> = project_gallery(&media, &query)
                .into_iter()
                .map(|item| {
                    let original = PathBuf::from(&item.absolute_path);
                    let matched_row = rows.iter().find(|row| row.media_id == item.media_id);
                    let thumbnail_path = matched_row.and_then(|row| {
                        resolve_thumbnail(&app.runtime.thumbnails_dir, row, GALLERY_THUMB_SIZE)
                    });
                    let (aspect_ratio, subtitle) = matched_row
                        .map(|row| {
                            let ar = aspect_ratio_from(row.width_px, row.height_px);
                            let sub = format!(
                                "{} \u{00B7} {}",
                                row.media_kind,
                                format::format_file_size(row.file_size_bytes)
                            );
                            (ar, sub)
                        })
                        .unwrap_or((1.5, item.media_kind.clone()));
                    BrowseItem {
                        media_id: item.media_id,
                        title: original
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| original.display().to_string()),
                        subtitle,
                        thumbnail_path,
                        is_group_header: false,
                        line: format!("{} [{}]", original.display(), item.media_kind),
                        aspect_ratio,
                    }
                })
                .collect();
            populate_media_cache(&mut app.media_cache, &rows, &app.runtime.thumbnails_dir);
            items
        })
        .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state
        .replace_gallery_preview(rows.iter().map(|item| item.line.clone()).collect());
    app.gallery_items = rows;
    app.browse_status = app.i18n.text(TextKey::GalleryCompletedLabel).to_owned();
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
    let row = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    let Some(row) = row else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    let target = if containing_folder {
        row.absolute_path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| row.absolute_path.clone())
    } else {
        row.absolute_path
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
    let row = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    let Some(row) = row else {
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
        return;
    };
    if !row.absolute_path.exists() {
        app.details_action_status = app.i18n.text(TextKey::ErrorUnavailableFileLabel).to_owned();
        return;
    }
    match copy_to_clipboard(&row.absolute_path.display().to_string()) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::ErrorActionFailedLabel).to_owned()
        }
    }
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

fn copy_to_clipboard(value: &str) -> Result<(), std::io::Error> {
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

fn aspect_ratio_from(width: Option<u32>, height: Option<u32>) -> f32 {
    match (width, height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => w as f32 / h as f32,
        _ => 1.5,
    }
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

fn spawn_background_work(app: &Librapix) -> Task<Message> {
    let db_file = app.runtime.database_file.clone();
    let thumbs_dir = app.runtime.thumbnails_dir.clone();
    let min_size = app.min_file_size_bytes;
    let kind_filter = app.filter_media_kind.clone();
    let ext_filter = app.filter_extension.clone();
    let i18n = app.i18n;
    let selected_root_id = app.state.selected_root_id;

    Task::perform(
        async move {
            do_background_work(
                db_file,
                thumbs_dir,
                min_size,
                kind_filter,
                ext_filter,
                i18n,
                selected_root_id,
            )
        },
        |result| Message::BackgroundWorkComplete(Box::new(result)),
    )
}

fn do_background_work(
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    min_file_size_bytes: u64,
    filter_media_kind: Option<String>,
    filter_extension: Option<String>,
    i18n: Translator,
    selected_root_id: Option<i64>,
) -> BackgroundWorkResult {
    let mut out = BackgroundWorkResult::default();

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

    let _ = storage.reconcile_source_root_availability();
    out.roots = storage
        .list_source_roots()
        .map(map_roots_from_storage)
        .unwrap_or_default();

    let all_rows = storage.list_all_media_read_models().unwrap_or_default();

    let media = rows_to_projection_media(&all_rows);

    let gallery_query = GalleryQuery {
        media_kind: filter_media_kind.clone(),
        extension: filter_extension.clone(),
        tag: None,
        sort: GallerySort::ModifiedDesc,
        limit: all_rows.len(),
        offset: 0,
    };
    out.gallery_items = project_gallery(&media, &gallery_query)
        .into_iter()
        .map(|item| {
            let original = PathBuf::from(&item.absolute_path);
            let matched_row = all_rows.iter().find(|row| row.media_id == item.media_id);
            let thumbnail_path = matched_row
                .and_then(|row| resolve_thumbnail(&thumbnails_dir, row, GALLERY_THUMB_SIZE));
            let (aspect_ratio, subtitle) = matched_row
                .map(|row| {
                    let ar = aspect_ratio_from(row.width_px, row.height_px);
                    let sub = format!(
                        "{} \u{00B7} {}",
                        row.media_kind,
                        format::format_file_size(row.file_size_bytes)
                    );
                    (ar, sub)
                })
                .unwrap_or((1.5, item.media_kind.clone()));
            BrowseItem {
                media_id: item.media_id,
                title: original
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| original.display().to_string()),
                subtitle,
                thumbnail_path,
                is_group_header: false,
                line: format!("{} [{}]", original.display(), item.media_kind),
                aspect_ratio,
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
        .collect();

    let buckets = project_timeline(&filtered_media, TimelineGranularity::Day);
    let mut timeline_lines = Vec::new();
    let mut timeline_items = Vec::new();
    for bucket in buckets {
        timeline_lines.push(format!("{} ({})", bucket.label, bucket.item_count));
        timeline_items.push(BrowseItem {
            media_id: 0,
            title: bucket.label.clone(),
            subtitle: String::new(),
            thumbnail_path: None,
            is_group_header: true,
            line: bucket.label.clone(),
            aspect_ratio: 1.5,
        });
        for tl_item in bucket.items {
            let matched_row = all_rows.iter().find(|r| r.media_id == tl_item.media_id);
            let thumbnail_path = matched_row
                .and_then(|row| resolve_thumbnail(&thumbnails_dir, row, GALLERY_THUMB_SIZE));
            let ar = matched_row
                .map(|r| aspect_ratio_from(r.width_px, r.height_px))
                .unwrap_or(1.5);
            timeline_items.push(BrowseItem {
                media_id: tl_item.media_id,
                title: PathBuf::from(&tl_item.absolute_path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or(tl_item.absolute_path.clone()),
                subtitle: tl_item.media_kind.clone(),
                thumbnail_path,
                is_group_header: false,
                line: format!("{} [{}]", tl_item.absolute_path, tl_item.media_kind),
                aspect_ratio: ar,
            });
        }
    }
    out.timeline_items = timeline_items;
    out.timeline_preview_lines = timeline_lines;

    populate_media_cache(&mut out.media_cache, &all_rows, &thumbnails_dir);

    out.browse_status = i18n.text(TextKey::GalleryCompletedLabel).to_owned();
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
        "filter: kind={:?}, ext={:?}",
        app.filter_media_kind.as_deref().unwrap_or("all"),
        app.filter_extension.as_deref().unwrap_or("all")
    ));
    lines.push(format!("min file size: {} bytes", app.min_file_size_bytes));
    lines.push(format!("browse status: {}", app.browse_status));

    app.diagnostics_lines = lines;
}

fn apply_background_result(app: &mut Librapix, result: BackgroundWorkResult) {
    app.state.apply(AppMessage::ReplaceLibraryRoots);
    app.state.replace_library_roots(result.roots);

    if let Some(summary) = result.indexing_summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
    }

    app.thumbnail_status = result.thumbnail_status;
    app.indexing_status = result.indexing_status;

    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state
        .replace_gallery_preview(result.gallery_preview_lines);
    app.gallery_items = result.gallery_items;

    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state
        .replace_timeline_preview(result.timeline_preview_lines);
    app.timeline_items = result.timeline_items;

    app.media_cache = result.media_cache;
    app.browse_status = result.browse_status;
    app.ignore_rules_preview = result.ignore_rules_preview;
    app.root_tags_preview = result.root_tags_preview;
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
