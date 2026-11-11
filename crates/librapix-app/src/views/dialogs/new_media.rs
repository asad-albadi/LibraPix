use super::modal_overlay;
use crate::ui::components::{
    button::{ButtonSize, ButtonVariant, app_button},
    separator::horizontal,
};
use crate::ui::patterns::dialog::{DialogSpec, dialog};
use crate::ui::{
    styles::{
        container::raised_surface_style,
        text::{text_primary, text_secondary, text_tertiary},
    },
    tokens::*,
};
use crate::{Librapix, Message, assets, format, localized_media_kind_label};
use iced::{
    ContentFit, Element, Length,
    widget::image::FilterMethod,
    widget::{column, container, image, row, stack, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
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
        .style(text_tertiary)
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
        .style(raised_surface_style)
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
        .style(text_secondary),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsSizeLabel),
            format::format_file_size(announcement.file_size_bytes)
        ))
        .size(FONT_CAPTION)
        .style(text_secondary),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsModifiedLabel),
            format::format_timestamp(announcement.modified_unix_seconds)
        ))
        .size(FONT_CAPTION)
        .style(text_secondary),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsDimensionsLabel),
            format::format_dimensions(announcement.width_px, announcement.height_px),
        ))
        .size(FONT_CAPTION)
        .style(text_secondary),
        text(format!(
            "{}: {}",
            app.i18n.text(TextKey::DetailsPathLabel),
            announcement.absolute_path.display()
        ))
        .size(FONT_CAPTION)
        .style(text_tertiary),
    ]
    .spacing(SPACE_2XS);

    let actions = row![
        app_button(
            row![
                image(assets::icon_gallery(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::MediaSelectButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::SelectMedia(announcement.media_id)),
            ButtonVariant::Ghost,
            ButtonSize::Default
        )
        .width(Length::Fill),
        app_button(
            row![
                image(assets::icon_open(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsOpenFileButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::OpenMediaById(announcement.media_id)),
            ButtonVariant::Secondary,
            ButtonSize::Default
        )
        .width(Length::Fill),
        app_button(
            row![
                image(assets::icon_copy_file(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsCopyFileButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::CopyMediaFileById(announcement.media_id)),
            ButtonVariant::Secondary,
            ButtonSize::Default
        )
        .width(Length::Fill),
        app_button(
            text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY),
            Some(Message::DismissNewMediaAnnouncement),
            ButtonVariant::Ghost,
            ButtonSize::Default
        )
        .width(Length::Fill),
    ]
    .spacing(SPACE_XS);

    let dialog_content = column![
        text(announcement.title.as_str())
            .size(FONT_SUBTITLE)
            .style(text_primary),
        text(announcement.metadata_line.as_str())
            .size(FONT_CAPTION)
            .style(text_secondary),
        more_line,
        horizontal(),
        preview,
        metadata_lines,
    ]
    .spacing(SPACE_SM);

    modal_overlay(dialog(DialogSpec {
        title: app.i18n.text(TextKey::NewFileAnnouncementTitle).into(),
        close_message: None,
        body: dialog_content.into(),
        footer: Some(actions.into()),
        max_width: 640.0,
        max_height: 680.0,
    }))
}

fn render_new_media_preview_loading_state(app: &Librapix, height: f32) -> Element<'_, Message> {
    use crate::ui::components::loading::skeleton;
    use crate::ui::patterns::status::{StatusTone, status_row};
    stack([
        skeleton(Length::Fill, height).into(),
        container(status_row(
            app.i18n
                .text(TextKey::NewFileAnnouncementPreparingPreviewLabel),
            StatusTone::Working,
            app.new_media_preview_loading_phase,
        ))
        .center_x(Length::Fill)
        .center_y(height)
        .into(),
    ])
    .width(Length::Fill)
    .height(height)
    .into()
}
