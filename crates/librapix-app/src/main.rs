use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Length, Task, Theme};
use librapix_config::{LocalePreference, ThemePreference, load_or_create};
use librapix_core::app::{AppMessage, AppState, Route};
use librapix_core::domain::non_destructive;
use librapix_i18n::{Locale, TextKey, Translator};
use librapix_storage::Storage;

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
}

struct Librapix {
    state: AppState,
    i18n: Translator,
    theme_preference: ThemePreference,
    registered_roots: usize,
}

impl Default for Librapix {
    fn default() -> Self {
        let bootstrap = bootstrap_runtime();

        Self {
            state: AppState::default(),
            i18n: Translator::new(bootstrap.locale),
            theme_preference: bootstrap.theme_preference,
            registered_roots: bootstrap.registered_roots,
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
        Message::OpenGallery => app.state.apply(AppMessage::OpenGallery),
        Message::OpenTimeline => app.state.apply(AppMessage::OpenTimeline),
    }

    Task::none()
}

fn view(app: &Librapix) -> Element<'_, Message> {
    let active_view_text = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::GalleryTab),
        Route::Timeline => app.i18n.text(TextKey::TimelineTab),
    };

    let _required_rules = non_destructive::required_rules();

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
            app.registered_roots
        )),
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
    registered_roots: usize,
}

fn bootstrap_runtime() -> BootstrapRuntime {
    let mut runtime = BootstrapRuntime {
        locale: Locale::EnUs,
        theme_preference: ThemePreference::System,
        registered_roots: 0,
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

    let storage = match Storage::open(&database_file) {
        Ok(storage) => storage,
        Err(_) => return runtime,
    };

    for source in &loaded.config.library_source_roots {
        let _ = storage.upsert_source_root(&source.path);
    }

    runtime.registered_roots = storage.list_source_roots().map_or(0, |roots| roots.len());
    runtime
}
