use super::modal_overlay;
use crate::ui::{
    styles::text::{text_primary, text_secondary},
    tokens::*,
};
use crate::{APP_VERSION, Librapix, Message, assets};
use iced::{
    ContentFit, Element, Length,
    widget::{column, row, svg, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let body = column![
        row![
            svg(assets::logo_svg())
                .width(Length::Fixed(44.0))
                .height(Length::Fixed(44.0))
                .content_fit(ContentFit::Contain),
            column![
                text(app.i18n.text(TextKey::AppTitle))
                    .size(FONT_TITLE)
                    .style(text_primary),
                text(format!(
                    "{}: {}",
                    app.i18n.text(TextKey::AboutVersionLabel),
                    APP_VERSION
                ))
                .size(FONT_CAPTION)
                .style(text_secondary),
            ]
            .spacing(SPACE_2XS),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        text(app.i18n.text(TextKey::AboutDescription))
            .size(FONT_BODY)
            .style(text_secondary),
        text(app.i18n.text(TextKey::AboutSafetyNote))
            .size(FONT_BODY)
            .style(text_secondary),
        crate::ui::components::separator::horizontal(),
        text(app.i18n.text(TextKey::AboutCreatorLabel))
            .size(FONT_CAPTION)
            .style(text_secondary),
    ]
    .spacing(SPACE_MD);

    modal_overlay(crate::ui::patterns::dialog::dialog(
        crate::ui::patterns::dialog::DialogSpec {
            title: app.i18n.text(TextKey::AboutDialogTitle).into(),
            close_message: Some((app.i18n.text(TextKey::DismissButton), Message::CloseAbout)),
            body: body.into(),
            footer: None,
            max_width: 460.0,
            max_height: 440.0,
        },
    ))
}
