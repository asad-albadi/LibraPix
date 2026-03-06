use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill, Length, Task, Theme};
use librapix_config::{LocalePreference, ThemePreference, lexical_normalize_path, load_or_create};
use librapix_core::app::{
    AppMessage, AppState, IndexingSummary, LibraryRootView, RootLifecycle, Route,
};
use librapix_core::domain::non_destructive;
use librapix_i18n::{Locale, TextKey, Translator};
use librapix_indexer::{IgnoreEngine, ScanRoot, scan_roots};
use librapix_projections::ProjectionMedia;
use librapix_projections::gallery::{GalleryQuery, GallerySort, project_gallery};
use librapix_projections::timeline::{TimelineGranularity, project_timeline};
use librapix_search::{FuzzySearchStrategy, SearchDocument, SearchQuery, SearchStrategy};
use librapix_storage::{
    IndexedMediaWrite, IndexedMetadataStatus, SourceRootLifecycle, Storage, TagKind,
};
use librapix_thumbnails::ensure_image_thumbnail;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn main() -> iced::Result {
    iced::application(Librapix::default, update, view)
        .title(title)
        .theme(theme)
        .run()
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
    SelectedMediaIdChanged(String),
    DetailsTagInputChanged(String),
    LoadMediaDetails,
    AttachAppTag,
    AttachGameTag,
    DetachTag,
    OpenSelectedFile,
    OpenSelectedFolder,
    CopySelectedPath,
}

struct Librapix {
    state: AppState,
    i18n: Translator,
    theme_preference: ThemePreference,
    runtime: RuntimeContext,
    thumbnail_status: String,
    selected_media_id_input: String,
    details_tag_input: String,
    details_lines: Vec<String>,
    details_action_status: String,
}

#[derive(Debug, Clone)]
struct RuntimeContext {
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
}

impl Default for Librapix {
    fn default() -> Self {
        let bootstrap = bootstrap_runtime();

        Self {
            state: AppState {
                library_roots: bootstrap.roots,
                ..AppState::default()
            },
            i18n: Translator::new(bootstrap.locale),
            theme_preference: bootstrap.theme_preference,
            runtime: RuntimeContext {
                database_file: bootstrap.database_file,
                thumbnails_dir: bootstrap.thumbnails_dir,
            },
            thumbnail_status: String::new(),
            selected_media_id_input: String::new(),
            details_tag_input: String::new(),
            details_lines: Vec::new(),
            details_action_status: String::new(),
        }
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

fn update(app: &mut Librapix, message: Message) -> Task<Message> {
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
        }
        Message::AddRoot => {
            if let Some(path) = normalized_input_path(&app.state.root_input)
                && with_storage(&app.runtime, |storage| storage.upsert_source_root(&path)).is_ok()
            {
                refresh_roots(app);
                app.state.clear_selection_and_input();
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
            }
        }
        Message::RemoveRoot => {
            if let Some(id) = app.state.selected_root_id
                && with_storage(&app.runtime, |storage| storage.remove_source_root(id)).is_ok()
            {
                refresh_roots(app);
                app.state.apply(AppMessage::ClearRootSelection);
                app.state.clear_selection_and_input();
            }
        }
        Message::RefreshRoots => {
            refresh_roots(app);
        }
        Message::RunIndexing => {
            run_indexing(app);
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
        Message::SelectedMediaIdChanged(value) => {
            app.selected_media_id_input = value;
        }
        Message::DetailsTagInputChanged(value) => {
            app.details_tag_input = value;
        }
        Message::LoadMediaDetails => {
            load_media_details(app);
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
    }

    Task::none()
}

fn view(app: &Librapix) -> Element<'_, Message> {
    let active_view_text = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::GalleryTab),
        Route::Timeline => app.i18n.text(TextKey::TimelineTab),
    };

    let _required_rules = non_destructive::required_rules();

    let root_rows = app
        .state
        .library_roots
        .iter()
        .fold(column![].spacing(8), |rows, root| {
            rows.push(
                row![
                    text(root.normalized_path.display().to_string()).width(Length::FillPortion(3)),
                    text(format!(
                        "{}: {}",
                        app.i18n.text(TextKey::RootLifecycleLabel),
                        lifecycle_text(app.i18n, root.lifecycle)
                    ))
                    .width(Length::FillPortion(2)),
                    button(app.i18n.text(TextKey::RootSelectButton))
                        .on_press(Message::SelectRoot(root.id)),
                ]
                .spacing(8),
            )
        });

    let selected_label = app
        .state
        .selected_root_id
        .map_or_else(|| "-".to_owned(), |id| id.to_string());

    let gallery_panel = column![
        button(app.i18n.text(TextKey::GalleryRunButton)).on_press(Message::RunGalleryProjection),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::GalleryResultLabel),
            app.state.gallery_preview.len()
        )),
        app.state
            .gallery_preview
            .iter()
            .take(12)
            .fold(column![].spacing(4), |rows, value| rows.push(text(value))),
    ]
    .spacing(8);

    let timeline_panel = column![
        button(app.i18n.text(TextKey::TimelineRunButton)).on_press(Message::RunTimelineProjection),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::TimelineResultLabel),
            app.state.timeline_preview.len()
        )),
        app.state
            .timeline_preview
            .iter()
            .take(12)
            .fold(column![].spacing(4), |rows, value| rows.push(text(value))),
    ]
    .spacing(8);

    let route_panel = match app.state.active_route {
        Route::Gallery => gallery_panel,
        Route::Timeline => timeline_panel,
    };

    let content = column![
        text(app.i18n.text(TextKey::AppTitle)).size(32),
        text(app.i18n.text(TextKey::AppSubtitle)).size(18),
        row![
            button(app.i18n.text(TextKey::GalleryTab)).on_press(Message::OpenGallery),
            button(app.i18n.text(TextKey::TimelineTab)).on_press(Message::OpenTimeline),
        ]
        .spacing(12),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::ActiveViewLabel),
            active_view_text
        )),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::RegisteredRootsLabel),
            app.state.library_roots.len()
        )),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::RootSelectedLabel),
            selected_label
        )),
        text(app.i18n.text(TextKey::RootInputLabel)),
        text_input("", &app.state.root_input)
            .on_input(Message::RootInputChanged)
            .width(Length::Fill),
        row![
            button(app.i18n.text(TextKey::RootAddButton)).on_press(Message::AddRoot),
            button(app.i18n.text(TextKey::RootUpdateButton)).on_press(Message::UpdateRoot),
            button(app.i18n.text(TextKey::RootDeactivateButton)).on_press(Message::DeactivateRoot),
            button(app.i18n.text(TextKey::RootReactivateButton)).on_press(Message::ReactivateRoot),
            button(app.i18n.text(TextKey::RootRemoveButton)).on_press(Message::RemoveRoot),
            button(app.i18n.text(TextKey::RootRefreshButton)).on_press(Message::RefreshRoots),
        ]
        .spacing(8),
        button(app.i18n.text(TextKey::IndexRunButton)).on_press(Message::RunIndexing),
        text(format!(
            "{}: roots={}, candidates={}, ignored={}, {}={}, {}={}, {}={}, {}={}, rows={}",
            app.i18n.text(TextKey::ScanSummaryLabel),
            app.state.indexing_summary.scanned_roots,
            app.state.indexing_summary.candidate_files,
            app.state.indexing_summary.ignored_entries,
            app.i18n.text(TextKey::ScanSummaryNew),
            app.state.indexing_summary.new_files,
            app.i18n.text(TextKey::ScanSummaryChanged),
            app.state.indexing_summary.changed_files,
            app.i18n.text(TextKey::ScanSummaryUnchanged),
            app.state.indexing_summary.unchanged_files,
            app.i18n.text(TextKey::ScanSummaryMissing),
            app.state.indexing_summary.missing_marked,
            app.state.indexing_summary.read_model_count
        )),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::ScanSummaryUnreadable),
            app.state.indexing_summary.unreadable_entries
        )),
        text(app.thumbnail_status.clone()),
        text(app.i18n.text(TextKey::SearchInputLabel)),
        text_input("", &app.state.search_query)
            .on_input(Message::SearchQueryChanged)
            .width(Length::Fill),
        button(app.i18n.text(TextKey::SearchRunButton)).on_press(Message::RunSearchQuery),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::SearchResultLabel),
            app.state.search_preview.len()
        )),
        app.state
            .search_preview
            .iter()
            .take(5)
            .fold(column![].spacing(4), |rows, value| rows.push(text(value))),
        route_panel,
        text(app.i18n.text(TextKey::DetailsSelectedMediaLabel)),
        text_input("", &app.selected_media_id_input)
            .on_input(Message::SelectedMediaIdChanged)
            .width(Length::Fill),
        text(app.i18n.text(TextKey::DetailsTagInputLabel)),
        text_input("", &app.details_tag_input)
            .on_input(Message::DetailsTagInputChanged)
            .width(Length::Fill),
        row![
            button(app.i18n.text(TextKey::DetailsLoadButton)).on_press(Message::LoadMediaDetails),
            button(app.i18n.text(TextKey::DetailsAttachTagButton)).on_press(Message::AttachAppTag),
            button(app.i18n.text(TextKey::DetailsAttachGameTagButton))
                .on_press(Message::AttachGameTag),
            button(app.i18n.text(TextKey::DetailsDetachTagButton)).on_press(Message::DetachTag),
        ]
        .spacing(8),
        row![
            button(app.i18n.text(TextKey::DetailsOpenFileButton))
                .on_press(Message::OpenSelectedFile),
            button(app.i18n.text(TextKey::DetailsOpenFolderButton))
                .on_press(Message::OpenSelectedFolder),
            button(app.i18n.text(TextKey::DetailsCopyPathButton))
                .on_press(Message::CopySelectedPath),
        ]
        .spacing(8),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsActionStatusLabel),
            app.details_action_status
        )),
        if app.details_lines.is_empty() {
            column![text(app.i18n.text(TextKey::DetailsNoSelectionLabel))]
        } else {
            app.details_lines
                .iter()
                .take(8)
                .fold(column![].spacing(4), |rows, value| rows.push(text(value)))
        },
        root_rows,
        text(app.i18n.text(TextKey::NonDestructiveNotice)).size(14),
    ]
    .spacing(16)
    .max_width(640);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Fill)
        .center_y(Fill)
        .padding(24)
        .into()
}

struct BootstrapRuntime {
    locale: Locale,
    theme_preference: ThemePreference,
    database_file: PathBuf,
    thumbnails_dir: PathBuf,
    roots: Vec<LibraryRootView>,
}

fn bootstrap_runtime() -> BootstrapRuntime {
    let mut runtime = BootstrapRuntime {
        locale: Locale::EnUs,
        theme_preference: ThemePreference::System,
        database_file: PathBuf::from("librapix.db"),
        thumbnails_dir: PathBuf::from("thumbnails"),
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
}

fn run_indexing(app: &mut Librapix) {
    let prep = with_storage(&app.runtime, |storage| {
        storage.reconcile_source_root_availability()?;
        storage.ensure_default_ignore_rules()?;

        let roots = storage.list_eligible_source_roots()?;
        let roots_for_scan = roots
            .iter()
            .map(|root| ScanRoot {
                source_root_id: root.id,
                normalized_path: root.normalized_path.clone(),
            })
            .collect::<Vec<_>>();

        let patterns = storage.list_enabled_ignore_patterns("global")?;
        Ok((roots_for_scan, patterns))
    });

    let summary = prep.ok().and_then(|(roots_for_scan, patterns)| {
        let ignore = IgnoreEngine::new(&patterns).ok()?;
        let root_ids = roots_for_scan
            .iter()
            .map(|root| root.source_root_id)
            .collect::<Vec<_>>();
        let existing = with_storage(&app.runtime, |storage| {
            storage.list_existing_indexed_media_snapshots(&root_ids)
        })
        .ok()?;

        let existing_for_indexer = existing
            .into_iter()
            .map(|entry| librapix_indexer::ExistingIndexedEntry {
                source_root_id: entry.source_root_id,
                absolute_path: entry.absolute_path,
                file_size_bytes: entry.file_size_bytes,
                modified_unix_seconds: entry.modified_unix_seconds,
            })
            .collect::<Vec<_>>();

        let result = scan_roots(&roots_for_scan, &ignore, &existing_for_indexer);
        let writes = result
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
                    librapix_indexer::MetadataStatus::Unreadable => {
                        IndexedMetadataStatus::Unreadable
                    }
                },
            })
            .collect::<Vec<_>>();

        let apply_summary = with_storage(&app.runtime, |storage| {
            storage.apply_incremental_index(&writes, &result.scanned_root_ids)
        })
        .ok()?;

        let _ = with_storage(&app.runtime, |storage| {
            storage.ensure_media_kind_tags_attached()
        });

        let read_models = with_storage(&app.runtime, |storage| {
            storage.list_media_read_models(200, 0)
        })
        .ok()?;

        let mut generated = 0usize;
        let mut reused = 0usize;
        let mut failed = 0usize;
        for row in &read_models {
            if row.media_kind != "image" {
                continue;
            }
            match ensure_image_thumbnail(
                &app.runtime.thumbnails_dir,
                &row.absolute_path,
                row.file_size_bytes,
                row.modified_unix_seconds,
                256,
            ) {
                Ok(outcome) => {
                    if outcome.generated {
                        generated += 1;
                    } else {
                        reused += 1;
                    }
                }
                Err(_) => failed += 1,
            }
        }
        app.thumbnail_status = format!(
            "{}: {}={generated}, {}={reused}, {}={failed}",
            app.i18n.text(TextKey::ThumbnailStatusLabel),
            app.i18n.text(TextKey::ThumbnailGeneratedLabel),
            app.i18n.text(TextKey::ThumbnailReusedLabel),
            app.i18n.text(TextKey::ThumbnailFailedLabel),
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
    });

    if let Some(summary) = summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
    }
    refresh_roots(app);
}

fn run_read_model_query(app: &mut Librapix) {
    let query = app.state.search_query.clone();
    let rows = with_storage(&app.runtime, |storage| {
        storage.list_media_read_models(200, 0)
    })
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
            .map(|(hit, row)| {
                if row.tags.is_empty() {
                    format!(
                        "{:.3} {} [{}] {}x{}",
                        hit.score,
                        row.absolute_path.display(),
                        row.media_kind,
                        row.width_px.unwrap_or(0),
                        row.height_px.unwrap_or(0)
                    )
                } else {
                    format!(
                        "{:.3} {} [{}] tags={}",
                        hit.score,
                        row.absolute_path.display(),
                        row.media_kind,
                        row.tags.join("|")
                    )
                }
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();

    app.state.apply(AppMessage::ReplaceSearchPreview);
    app.state.replace_search_preview(rows);
}

fn run_timeline_projection(app: &mut Librapix) {
    let rows = with_storage(&app.runtime, |storage| {
        storage.list_media_read_models(500, 0)
    })
    .map(|rows| {
        let media = rows_to_projection_media(&rows);
        project_timeline(&media, TimelineGranularity::Day)
            .into_iter()
            .map(|bucket| format!("{} ({})", bucket.label, bucket.item_count))
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceTimelinePreview);
    app.state.replace_timeline_preview(rows);
}

fn run_gallery_projection(app: &mut Librapix) {
    let rows = with_storage(&app.runtime, |storage| {
        storage.list_media_read_models(500, 0)
    })
    .map(|rows| {
        let media = rows_to_projection_media(&rows);
        let query = GalleryQuery {
            media_kind: None,
            tag: None,
            sort: GallerySort::ModifiedDesc,
            limit: 20,
            offset: 0,
        };
        project_gallery(&media, &query)
            .into_iter()
            .map(|item| {
                let original = PathBuf::from(&item.absolute_path);
                let thumbnail_text = rows
                    .iter()
                    .find(|row| row.media_id == item.media_id)
                    .and_then(|row| {
                        if row.media_kind != "image" {
                            return None;
                        }
                        ensure_image_thumbnail(
                            &app.runtime.thumbnails_dir,
                            &row.absolute_path,
                            row.file_size_bytes,
                            row.modified_unix_seconds,
                            256,
                        )
                        .ok()
                        .map(|outcome| outcome.thumbnail_path.display().to_string())
                    })
                    .unwrap_or_else(|| app.i18n.text(TextKey::ThumbnailUnavailable).to_owned());
                format!(
                    "{} [{}] {}={thumbnail_text}",
                    original.display(),
                    item.media_kind,
                    app.i18n.text(TextKey::ThumbnailStatusLabel),
                )
            })
            .collect::<Vec<_>>()
    })
    .unwrap_or_default();
    app.state.apply(AppMessage::ReplaceGalleryPreview);
    app.state.replace_gallery_preview(rows);
}

fn parse_selected_media_id(app: &Librapix) -> Option<i64> {
    app.selected_media_id_input.trim().parse::<i64>().ok()
}

fn load_media_details(app: &mut Librapix) {
    let Some(media_id) = parse_selected_media_id(app) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsInvalidMediaId).to_owned();
        return;
    };
    let details = with_storage(&app.runtime, |storage| {
        storage.get_media_read_model_by_id(media_id)
    })
    .ok()
    .flatten();
    if let Some(details) = details {
        app.details_lines = vec![
            format!("id={}", details.media_id),
            format!("path={}", details.absolute_path.display()),
            format!("kind={}", details.media_kind),
            format!("size={}", details.file_size_bytes),
            format!(
                "modified={}",
                details.modified_unix_seconds.unwrap_or_default()
            ),
            format!(
                "dimensions={}x{}",
                details.width_px.unwrap_or(0),
                details.height_px.unwrap_or(0)
            ),
            format!("tags={}", details.tags.join("|")),
        ];
        app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned();
    } else {
        app.details_lines.clear();
        app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned();
    }
}

fn attach_tag_to_selected_media(app: &mut Librapix, kind: TagKind) {
    let Some(media_id) = parse_selected_media_id(app) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsInvalidMediaId).to_owned();
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
    let Some(media_id) = parse_selected_media_id(app) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsInvalidMediaId).to_owned();
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
    let Some(media_id) = parse_selected_media_id(app) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsInvalidMediaId).to_owned();
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
    match open_with_system_default(&target) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned()
        }
    }
}

fn copy_selected_path(app: &mut Librapix) {
    let Some(media_id) = parse_selected_media_id(app) else {
        app.details_action_status = app.i18n.text(TextKey::DetailsInvalidMediaId).to_owned();
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
    match copy_to_clipboard(&row.absolute_path.display().to_string()) {
        Ok(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionSuccess).to_owned()
        }
        Err(_) => {
            app.details_action_status = app.i18n.text(TextKey::DetailsActionFailed).to_owned()
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

fn lifecycle_text(translator: Translator, lifecycle: RootLifecycle) -> &'static str {
    match lifecycle {
        RootLifecycle::Active => translator.text(TextKey::RootLifecycleActive),
        RootLifecycle::Unavailable => translator.text(TextKey::RootLifecycleUnavailable),
        RootLifecycle::Deactivated => translator.text(TextKey::RootLifecycleDeactivated),
    }
}
