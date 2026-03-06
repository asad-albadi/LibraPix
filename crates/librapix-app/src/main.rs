use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill, Length, Task, Theme};
use librapix_core::app::{AppMessage, AppState, Route};
use librapix_core::domain::non_destructive;
use librapix_i18n::{Locale, TextKey, Translator};

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
}

impl Default for Librapix {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            i18n: Translator::new(Locale::EnUs),
        }
    }
}

fn title(app: &Librapix) -> String {
    let _ = app.i18n.locale();
    app.i18n.text(TextKey::AppTitle).to_owned()
}

fn theme(_app: &Librapix) -> Theme {
    Theme::Dark
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
