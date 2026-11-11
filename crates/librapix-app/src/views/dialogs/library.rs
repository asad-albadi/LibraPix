use super::modal_overlay;
use crate::ui::components::input::app_input;
use crate::ui::components::{
    button::{ButtonSize, ButtonVariant, app_button},
    separator::horizontal,
};
use crate::ui::patterns::dialog::{DialogSpec, dialog};
use crate::ui::patterns::field::{FieldSpec, field};
use crate::ui::*;
use crate::{Librapix, LibraryDialogMode, Message, assets, render_management_chip};
use iced::{
    ContentFit, Element, Length,
    widget::image::FilterMethod,
    widget::{column, image, row, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let (title, is_add_mode, edit_root_id) = match app.library_dialog_mode {
        LibraryDialogMode::Add => (app.i18n.text(TextKey::LibraryDialogAddTitle), true, None),
        LibraryDialogMode::Edit(root_id) => (
            app.i18n.text(TextKey::LibraryDialogEditTitle),
            false,
            Some(root_id),
        ),
    };

    let path_error = (app.root_status == app.i18n.text(TextKey::ErrorInvalidRootPathLabel))
        .then_some(app.root_status.as_str());

    let path_toggle_label = if app.library_dialog_manual_path_open {
        app.i18n.text(TextKey::HidePathFieldLabel)
    } else {
        app.i18n.text(TextKey::ShowPathFieldLabel)
    };

    let path_field: Element<'_, Message> = if app.library_dialog_manual_path_open {
        app_input(
            app.i18n.text(TextKey::FolderPathPlaceholder),
            &app.library_dialog_path_input,
            false,
        )
        .on_input(Message::LibraryDialogPathInputChanged)
        .into()
    } else {
        column![].into()
    };

    let tags_list: Element<'_, Message> = if app.library_dialog_tags.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_CAPTION)
            .style(text_tertiary)
            .into()
    } else {
        app.library_dialog_tags
            .iter()
            .fold(column![].spacing(SPACE_XS), |col, (name, kind)| {
                col.push(render_management_chip(
                    name,
                    name,
                    Some(kind.as_str().to_owned()),
                    None,
                    Some((
                        app.i18n.text(TextKey::RootEditButton).to_owned(),
                        Message::LibraryDialogStartEditTag(name.clone()),
                    )),
                    Some(Message::LibraryDialogRemoveTag(name.clone())),
                    app.is_dark_theme(),
                ))
            })
            .into()
    };

    let library_tag_actions: Element<'_, Message> = if app.library_dialog_editing_tag.is_some() {
        row![
            app_button(
                text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_CAPTION),
                Some(Message::LibraryDialogApplyTagEdit),
                ButtonVariant::Primary,
                ButtonSize::Small
            ),
            app_button(
                text(app.i18n.text(TextKey::DismissButton)).size(FONT_CAPTION),
                Some(Message::LibraryDialogCancelTagEdit),
                ButtonVariant::Ghost,
                ButtonSize::Small
            ),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        row![
            app_button(
                text(app.i18n.text(TextKey::RootTagAddButton)).size(FONT_CAPTION),
                Some(Message::LibraryDialogAddAppTag),
                ButtonVariant::Ghost,
                ButtonSize::Small
            ),
            app_button(
                text(app.i18n.text(TextKey::RootTagGameButton)).size(FONT_CAPTION),
                Some(Message::LibraryDialogAddGameTag),
                ButtonVariant::Ghost,
                ButtonSize::Small
            ),
        ]
        .spacing(SPACE_XS)
        .into()
    };

    let lifecycle_actions: Element<'_, Message> = if edit_root_id.is_some() {
        column![
            horizontal(),
            row![
                app_button(
                    text(app.i18n.text(TextKey::RootDeactivateButton)).size(FONT_CAPTION),
                    Some(Message::DeactivateRoot),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
                app_button(
                    text(app.i18n.text(TextKey::RootReactivateButton)).size(FONT_CAPTION),
                    Some(Message::ReactivateRoot),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
                app_button(
                    text(app.i18n.text(TextKey::RootRemoveButton)).size(FONT_CAPTION),
                    Some(Message::RemoveRoot),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
            ]
            .spacing(SPACE_XS),
        ]
        .into()
    } else {
        column![].into()
    };

    let add_another_button: Element<'_, Message> = if is_add_mode {
        app_button(
            row![
                image(assets::icon_save(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::LibrarySaveAndAddAnotherButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::SaveLibraryAndAddAnother),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        )
        .into()
    } else {
        column![].into()
    };

    let dialog_content = column![
        field(FieldSpec {
            label: app.i18n.text(TextKey::LibraryPathLabel),
            control: column![
                row![
                    app_button(
                        row![
                            image(assets::icon_browse(app.is_dark_theme()))
                                .width(Length::Fixed(ICON_SM))
                                .height(Length::Fixed(ICON_SM))
                                .content_fit(ContentFit::Contain)
                                .filter_method(FilterMethod::Linear),
                            text(app.i18n.text(TextKey::BrowseFolderButton)).size(FONT_BODY),
                        ]
                        .spacing(SPACE_SM)
                        .align_y(iced::Alignment::Center),
                        Some(Message::LibraryDialogBrowseFolder),
                        ButtonVariant::Primary,
                        ButtonSize::Default
                    ),
                    app_button(
                        text(path_toggle_label).size(FONT_CAPTION),
                        Some(Message::ToggleLibraryDialogManualPath),
                        ButtonVariant::Ghost,
                        ButtonSize::Small
                    ),
                ]
                .spacing(SPACE_XS),
                path_field,
                text(app.library_dialog_path_input.as_str())
                    .size(FONT_CAPTION)
                    .style(text_tertiary),
            ]
            .spacing(SPACE_SM)
            .into(),
            help: None,
            error: path_error,
        }),
        horizontal(),
        field(FieldSpec {
            label: app.i18n.text(TextKey::LibraryDisplayNameLabel),
            control: app_input(
                app.i18n.text(TextKey::DisplayNamePlaceholder),
                &app.library_dialog_display_name_input,
                false
            )
            .on_input(Message::LibraryDialogDisplayNameChanged)
            .into(),
            help: None,
            error: None,
        }),
        horizontal(),
        column![
            section_heading(app.i18n.text(TextKey::LibraryTagsLabel)),
            app_input(
                app.i18n.text(TextKey::RootTagInputPlaceholder),
                &app.library_dialog_tag_input,
                false
            )
            .on_input(Message::LibraryDialogTagInputChanged)
            .on_submit(if app.library_dialog_editing_tag.is_some() {
                Message::LibraryDialogApplyTagEdit
            } else {
                Message::LibraryDialogAddAppTag
            }),
            library_tag_actions,
            tags_list,
        ]
        .spacing(SPACE_SM),
        lifecycle_actions,
    ]
    .spacing(SPACE_LG);

    modal_overlay(dialog(DialogSpec {
        title: title.into(),
        close_message: Some((
            app.i18n.text(TextKey::DismissButton),
            Message::CloseLibraryDialog,
        )),
        body: dialog_content.into(),
        footer: Some(
            row![
                app_button(
                    row![
                        image(assets::icon_save(app.is_dark_theme()))
                            .width(Length::Fixed(ICON_SM))
                            .height(Length::Fixed(ICON_SM))
                            .content_fit(ContentFit::Contain)
                            .filter_method(FilterMethod::Linear),
                        text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_BODY),
                    ]
                    .spacing(SPACE_SM)
                    .align_y(iced::Alignment::Center),
                    Some(Message::SaveLibraryDialog),
                    ButtonVariant::Primary,
                    ButtonSize::Default
                ),
                add_another_button,
            ]
            .spacing(SPACE_XS)
            .into(),
        ),
        max_width: 560.0,
        max_height: 680.0,
    }))
}
