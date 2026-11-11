//! Selection inspector presentation; actions and persistence remain in update.
use crate::ui::components::{
    button::{ButtonSize, ButtonVariant, app_button},
    card::{CardVariant, card},
    empty_state::empty_state,
    input::app_input,
    loading::skeleton,
    separator::horizontal,
};
use crate::ui::*;
use crate::{
    DetailsTagChip, Librapix, Message, assets, render_management_chip, selected_media_is_video,
};
use iced::widget::image::FilterMethod;
use iced::{
    ContentFit, Element, Length, Size,
    widget::{Space, column, image, responsive, row, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    if app.state.selected_media_id.is_none() {
        return empty_state(
            Space::new().height(0),
            app.i18n.text(TextKey::SelectPhotoTitle),
            Some(app.i18n.text(TextKey::SelectPhotoSubtitle)),
            None,
        )
        .into();
    }

    let preview: Element<'_, Message> = if let Some(path) = &app.details_preview_path {
        card(
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .content_fit(ContentFit::Contain),
            CardVariant::Default,
        )
        .width(Length::Fill)
        .into()
    } else {
        skeleton(Length::Fill, 160.0).into()
    };

    let metadata = if app.details_lines.is_empty() {
        column![
            text(app.i18n.text(TextKey::DetailsNoSelectionLabel))
                .size(FONT_BODY)
                .style(text_tertiary)
        ]
    } else {
        app.details_lines
            .iter()
            .fold(column![].spacing(SPACE_XS), |col, line| {
                col.push(text(line.clone()).size(FONT_BODY).style(text_secondary))
            })
    };

    let details_tag_input_actions: Element<'_, Message> = if app.details_editing_tag.is_some() {
        row![
            app_button(
                text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_BODY),
                Some(Message::DetailsApplyTagEdit),
                ButtonVariant::Primary,
                ButtonSize::Default
            ),
            app_button(
                text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY),
                Some(Message::DetailsCancelTagEdit),
                ButtonVariant::Ghost,
                ButtonSize::Default
            ),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        row![
            app_button(
                text(app.i18n.text(TextKey::DetailsAttachTagButton)).size(FONT_BODY),
                Some(Message::AttachAppTag),
                ButtonVariant::Secondary,
                ButtonSize::Default
            ),
            app_button(
                text(app.i18n.text(TextKey::DetailsAttachGameTagButton)).size(FONT_BODY),
                Some(Message::AttachGameTag),
                ButtonVariant::Secondary,
                ButtonSize::Default
            ),
        ]
        .spacing(SPACE_XS)
        .into()
    };

    let inherited_tag_chips: Vec<&DetailsTagChip> = app
        .details_tags
        .iter()
        .filter(|tag| tag.inherited)
        .collect();
    let manual_tag_chips: Vec<&DetailsTagChip> = app
        .details_tags
        .iter()
        .filter(|tag| !tag.inherited)
        .collect();
    let details_tag_list: Element<'_, Message> = if app.details_tags.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_CAPTION)
            .style(text_tertiary)
            .into()
    } else {
        let inherited_section: Element<'_, Message> = if inherited_tag_chips.is_empty() {
            Space::new().height(Length::Shrink).into()
        } else {
            let list = inherited_tag_chips
                .iter()
                .fold(column![].spacing(SPACE_XS), |col, tag| {
                    col.push(render_management_chip(
                        &tag.name,
                        &tag.name,
                        Some(app.i18n.text(TextKey::InheritedTagLabel).to_owned()),
                        None,
                        None,
                        Some(Message::DetachTagByName(tag.name.clone())),
                        app.is_dark_theme(),
                    ))
                });
            column![
                section_heading(app.i18n.text(TextKey::InheritedTagLabel)),
                list,
            ]
            .spacing(SPACE_XS)
            .into()
        };
        let manual_section =
            manual_tag_chips
                .iter()
                .fold(column![].spacing(SPACE_XS), |col, tag| {
                    col.push(render_management_chip(
                        &tag.name,
                        &tag.name,
                        Some(tag.kind.as_str().to_owned()),
                        None,
                        Some((
                            app.i18n.text(TextKey::RootEditButton).to_owned(),
                            Message::DetailsStartEditTag(tag.name.clone()),
                        )),
                        Some(Message::DetachTagByName(tag.name.clone())),
                        app.is_dark_theme(),
                    ))
                });
        column![inherited_section, manual_section]
            .spacing(SPACE_SM)
            .into()
    };

    column![
        preview,
        text(app.details_title.clone())
            .size(FONT_SUBTITLE)
            .style(text_primary),
        horizontal(),
        column![
            section_heading(app.i18n.text(TextKey::FileInfoLabel)),
            metadata,
        ]
        .spacing(SPACE_SM),
        horizontal(),
        column![
            section_heading(app.i18n.text(TextKey::DetailsTagsSectionLabel)),
            app_input(
                app.i18n.text(TextKey::DetailsTagInputLabel),
                &app.details_tag_input,
                false
            )
            .on_input(Message::DetailsTagInputChanged)
            .on_submit(if app.details_editing_tag.is_some() {
                Message::DetailsApplyTagEdit
            } else {
                Message::AttachAppTag
            }),
            details_tag_input_actions,
            details_tag_list,
        ]
        .spacing(SPACE_SM),
        horizontal(),
        column![
            section_heading(app.i18n.text(TextKey::DetailsActionsSectionLabel)),
            render_details_actions(app),
            text(app.i18n.text(TextKey::DetailsCopyShortcutHint))
                .size(FONT_CAPTION)
                .style(text_tertiary),
        ]
        .spacing(SPACE_SM),
        text(app.details_action_status.clone())
            .size(FONT_CAPTION)
            .style(text_tertiary),
        text(app.i18n.text(TextKey::NonDestructiveNotice))
            .size(FONT_CAPTION)
            .style(text_tertiary),
    ]
    .spacing(SPACE_LG)
    .into()
}

fn render_details_actions(app: &Librapix) -> Element<'_, Message> {
    let show_make_short = selected_media_is_video(app);
    responsive(move |size: Size| {
        let open = app_button(
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
            Some(Message::OpenSelectedFile),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        )
        .width(Length::Fill);
        let open_folder = app_button(
            row![
                image(assets::icon_show_in_folder(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsOpenFolderButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::OpenSelectedFolder),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        )
        .width(Length::Fill);
        let copy_file = app_button(
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
            Some(Message::CopySelectedFile),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        )
        .width(Length::Fill);
        let copy_path = app_button(
            row![
                image(assets::icon_copy_path(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsCopyPathButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::CopySelectedPath),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        )
        .width(Length::Fill);
        let make_short = app_button(
            row![
                image(assets::icon_youtube(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::DetailsMakeShortButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::OpenMakeShortDialog),
            ButtonVariant::Primary,
            ButtonSize::Default,
        )
        .width(Length::Fill);

        if size.width < 220.0 {
            let mut col = column![open, open_folder, copy_file, copy_path].spacing(SPACE_XS);
            if show_make_short {
                col = col.push(make_short);
            }
            col.into()
        } else if size.width < 420.0 {
            let mut col = column![
                row![open, open_folder].spacing(SPACE_XS),
                row![copy_file, copy_path].spacing(SPACE_XS),
            ]
            .spacing(SPACE_XS);
            if show_make_short {
                col = col.push(make_short);
            }
            col.into()
        } else {
            if show_make_short {
                row![open, open_folder, copy_file, copy_path, make_short]
                    .spacing(SPACE_XS)
                    .into()
            } else {
                row![open, open_folder, copy_file, copy_path]
                    .spacing(SPACE_XS)
                    .into()
            }
        }
    })
    .into()
}
