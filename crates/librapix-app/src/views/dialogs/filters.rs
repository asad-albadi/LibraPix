use super::modal_overlay;
use crate::ui::components::button::{ButtonSize, ButtonVariant, app_button};
use crate::ui::components::tabs::tab_trigger;
use crate::ui::components::text::section_heading;
use crate::ui::patterns::dialog::{DialogSpec, dialog};
use crate::ui::{styles::text::text_tertiary, tokens::*};
use crate::{FILTER_DIALOG_MAX_WIDTH, Librapix, Message};
use iced::{
    Element,
    widget::{column, row, text},
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let type_chips = render_wrapped_filter_chips(vec![
        FilterChipSpec {
            label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
            on_press: Message::SetFilterMediaKind(None),
            active: app.filter_media_kind.is_none(),
        },
        FilterChipSpec {
            label: app.i18n.text(TextKey::FilterImagesLabel).to_owned(),
            on_press: Message::SetFilterMediaKind(Some("image".to_owned())),
            active: app.filter_media_kind.as_deref() == Some("image"),
        },
        FilterChipSpec {
            label: app.i18n.text(TextKey::FilterVideosLabel).to_owned(),
            on_press: Message::SetFilterMediaKind(Some("video".to_owned())),
            active: app.filter_media_kind.as_deref() == Some("video"),
        },
    ]);

    let ext_list: &[&str] = match app.filter_media_kind.as_deref() {
        Some("image") => &["png", "jpg", "gif", "webp"],
        Some("video") => &["mp4", "mov", "mkv", "webm", "avi"],
        _ => &["png", "jpg", "gif", "webp", "mp4", "mov", "mkv", "webm"],
    };
    let mut ext_chips = vec![FilterChipSpec {
        label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
        on_press: Message::SetFilterExtension(None),
        active: app.filter_extension.is_none(),
    }];
    for ext in ext_list {
        let is_active = app.filter_extension.as_deref() == Some(ext);
        ext_chips.push(FilterChipSpec {
            label: ext.to_uppercase(),
            on_press: Message::SetFilterExtension(Some((*ext).to_owned())),
            active: is_active,
        });
    }
    let ext_chips = render_wrapped_filter_chips(ext_chips);

    let mut library_chip_specs = vec![FilterChipSpec {
        label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
        on_press: Message::SetFilterLibrary(None),
        active: app.filter_source_root_id.is_none(),
    }];

    for root in &app.state.library_roots {
        let label = root
            .display_name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                root.normalized_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| root.normalized_path.display().to_string())
            });
        let is_active = app.filter_source_root_id == Some(root.id);
        library_chip_specs.push(FilterChipSpec {
            label,
            on_press: Message::SetFilterLibrary(Some(root.id)),
            active: is_active,
        });
    }

    let mut tag_chip_specs = vec![FilterChipSpec {
        label: app.i18n.text(TextKey::FilterAllLabel).to_owned(),
        on_press: Message::SetFilterTag(None),
        active: app.filter_tag.is_none(),
    }];

    for tag in &app.available_filter_tags {
        let active = app
            .filter_tag
            .as_ref()
            .is_some_and(|selected| selected == tag);
        tag_chip_specs.push(FilterChipSpec {
            label: tag.clone(),
            on_press: Message::SetFilterTag(Some(tag.clone())),
            active,
        });
    }

    let tag_section: Element<'_, Message> = if app.available_filter_tags.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_BODY)
            .style(text_tertiary)
            .into()
    } else {
        render_wrapped_filter_chips(tag_chip_specs)
    };

    let library_section: Element<'_, Message> = if app.state.library_roots.len() > 1 {
        column![
            section_heading(app.i18n.text(TextKey::FilterLibraryLabel)),
            render_wrapped_filter_chips(library_chip_specs),
        ]
        .spacing(SPACE_SM)
        .into()
    } else {
        column![].into()
    };

    let dialog_content = column![
        column![
            section_heading(app.i18n.text(TextKey::FilterTypeLabel)),
            type_chips,
        ]
        .spacing(SPACE_SM),
        column![
            section_heading(app.i18n.text(TextKey::FilterExtensionLabel)),
            ext_chips,
        ]
        .spacing(SPACE_SM),
        library_section,
        column![
            section_heading(app.i18n.text(TextKey::FilterTagsLabel)),
            tag_section,
        ]
        .spacing(SPACE_SM),
    ]
    .spacing(SPACE_LG);

    modal_overlay(dialog(DialogSpec {
        title: app.i18n.text(TextKey::FiltersButtonLabel).into(),
        close_message: None,
        body: dialog_content.into(),
        footer: Some(
            app_button(
                text(app.i18n.text(TextKey::DismissButton)).size(FONT_BODY),
                Some(Message::ToggleFilterDialog),
                ButtonVariant::Primary,
                ButtonSize::Default,
            )
            .into(),
        ),
        max_width: FILTER_DIALOG_MAX_WIDTH,
        max_height: 620.0,
    }))
}

#[derive(Debug, Clone)]
struct FilterChipSpec {
    label: String,
    on_press: Message,
    active: bool,
}

fn render_wrapped_filter_chips(chips: Vec<FilterChipSpec>) -> Element<'static, Message> {
    row(chips.into_iter().map(|chip| {
        tab_trigger(
            text(chip.label).size(FONT_BODY),
            Some(chip.on_press),
            chip.active,
        )
        .height(iced::Length::Shrink)
        .padding([(SPACE_XS + SPACE_2XS) as u16, SPACE_MD as u16])
        .into()
    }))
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center)
    .wrap()
    .vertical_spacing(SPACE_SM)
    .into()
}
#[cfg(test)]
#[path = "filter_tests.rs"]
mod tests;
