use iced::widget::{button, column, container, row, text, text_input};
use iced::{Element, Fill, Length, Task, Theme};
use librapix_config::{LocalePreference, ThemePreference, lexical_normalize_path, load_or_create};
use librapix_core::app::{
    AppMessage, AppState, IndexingSummary, LibraryRootView, RootLifecycle, Route,
};
use librapix_core::domain::non_destructive;
use librapix_i18n::{Locale, TextKey, Translator};
use librapix_indexer::{IgnoreEngine, ScanRoot, candidates_for_storage, scan_roots};
use librapix_storage::{SourceRootLifecycle, Storage};
use std::path::PathBuf;

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
}

struct Librapix {
    state: AppState,
    i18n: Translator,
    theme_preference: ThemePreference,
    runtime: RuntimeContext,
}

#[derive(Debug, Clone)]
struct RuntimeContext {
    database_file: PathBuf,
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
            },
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
            "{}: roots={}, candidates={}, ignored={}",
            app.i18n.text(TextKey::ScanSummaryLabel),
            app.state.indexing_summary.scanned_roots,
            app.state.indexing_summary.candidate_files,
            app.state.indexing_summary.ignored_entries
        )),
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
    roots: Vec<LibraryRootView>,
}

fn bootstrap_runtime() -> BootstrapRuntime {
    let mut runtime = BootstrapRuntime {
        locale: Locale::EnUs,
        theme_preference: ThemePreference::System,
        database_file: PathBuf::from("librapix.db"),
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
        let result = scan_roots(&roots_for_scan, &ignore);
        let rows = candidates_for_storage(&result);
        let persist_ok =
            with_storage(&app.runtime, |storage| storage.replace_indexed_media(&rows)).is_ok();
        if !persist_ok {
            return None;
        }
        Some(IndexingSummary {
            scanned_roots: result.summary.scanned_roots,
            candidate_files: result.summary.candidate_files,
            ignored_entries: result.summary.ignored_entries,
        })
    });

    if let Some(summary) = summary {
        app.state.apply(AppMessage::RecordIndexingSummary);
        app.state.record_indexing_summary(summary);
    }
    refresh_roots(app);
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
