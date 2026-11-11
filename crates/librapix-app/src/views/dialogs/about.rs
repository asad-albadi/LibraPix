use super::modal_overlay;
use crate::ui::{self, *};
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
            row![
                text("Libra").size(FONT_SUBTITLE).style(text_primary),
                text("Pix").size(FONT_SUBTITLE).style(text_accent),
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
        .style(text_secondary),
        text(app.i18n.text(TextKey::AboutCreatorLabel))
            .size(FONT_BODY)
            .style(text_secondary),
        text(app.i18n.text(TextKey::AboutWeekendProjectNote))
            .size(FONT_BODY)
            .style(text_secondary),
        text(app.i18n.text(TextKey::AboutSecondNote))
            .size(FONT_BODY)
            .style(text_secondary),
    ]
    .spacing(SPACE_LG);

    modal_overlay(ui::patterns::dialog::dialog(
        ui::patterns::dialog::DialogSpec {
            title: app.i18n.text(TextKey::AboutDialogTitle).into(),
            close_message: Some((app.i18n.text(TextKey::DismissButton), Message::CloseAbout)),
            body: body.into(),
            footer: None,
            max_width: 460.0,
            max_height: 440.0,
        },
    ))
}
