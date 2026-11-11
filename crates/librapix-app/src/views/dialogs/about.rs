use super::modal_overlay;
use crate::ui::components::brand::{BrandPlacement, brand};
use crate::ui::{styles::text::text_secondary, tokens::*};
use crate::{APP_VERSION, Librapix, Message};
use iced::{
    Element,
    widget::{column, row, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let body = column![
        row![
            brand(app.is_dark_theme(), BrandPlacement::AboutMark),
            column![
                brand(app.is_dark_theme(), BrandPlacement::AboutWordmark),
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
