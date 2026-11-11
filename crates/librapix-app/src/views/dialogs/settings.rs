use super::modal_overlay;
use crate::ui::components::input::app_input;
use crate::ui::components::tabs::tab_trigger;
use crate::ui::components::text::section_heading;
use crate::ui::components::tooltip::app_tooltip;
use crate::ui::components::{
    button::{ButtonSize, ButtonVariant, app_button},
    separator::horizontal,
};
use crate::ui::patterns::dialog::{DialogSpec, dialog};
use crate::ui::patterns::field::{FieldSpec, field};
use crate::ui::{
    styles::{
        button::managed_chip_action_style,
        container::{managed_chip_style, raised_surface_style},
        scrollable::scrollbar_style,
        text::{text_secondary, text_tertiary},
    },
    tokens::*,
};
use crate::{Librapix, Message, assets};
use iced::{
    ContentFit, Element, Length,
    widget::image::FilterMethod,
    widget::{Space, column, container, image, responsive, row, scrollable, text},
};
use librapix_config::ThemePreference;
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let indexing_section = column![
        section_heading(app.i18n.text(TextKey::IndexingSectionLabel)),
        app_button(
            row![
                image(assets::icon_index(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_MD))
                    .height(Length::Fixed(ICON_MD))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::IndexRunButton)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::RunIndexing),
            ButtonVariant::Primary,
            ButtonSize::Default
        )
        .width(Length::Fill),
        text(app.indexing_status.clone())
            .size(FONT_CAPTION)
            .style(text_tertiary),
        text(app.thumbnail_status.clone())
            .size(FONT_CAPTION)
            .style(text_tertiary),
    ]
    .spacing(SPACE_SM);

    let ignore_input_actions: Element<'_, Message> = if app.ignore_rule_editing_id.is_some() {
        row![
            app_button(
                text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_CAPTION),
                Some(Message::IgnoreRuleApplyEdit),
                ButtonVariant::Primary,
                ButtonSize::Small
            ),
            app_button(
                text(app.i18n.text(TextKey::DismissButton)).size(FONT_CAPTION),
                Some(Message::IgnoreRuleCancelEdit),
                ButtonVariant::Ghost,
                ButtonSize::Small
            ),
        ]
        .spacing(SPACE_XS)
        .into()
    } else {
        row![app_button(
            text(app.i18n.text(TextKey::IgnoreRuleAddButton)).size(FONT_CAPTION),
            Some(Message::IgnoreRuleAdd),
            ButtonVariant::Ghost,
            ButtonSize::Small
        ),]
        .spacing(SPACE_XS)
        .into()
    };

    let ignore_chip_list: Element<'_, Message> = if app.ignore_rules.is_empty() {
        text(app.i18n.text(TextKey::FilterNoTagsLabel))
            .size(FONT_CAPTION)
            .style(text_tertiary)
            .into()
    } else {
        column(
            app.ignore_rules
                .iter()
                .map(|rule| ignore_rule_row(app, rule)),
        )
        .spacing(SPACE_2XS)
        .into()
    };
    let ignore_section = column![
        field(FieldSpec {
            label: app.i18n.text(TextKey::IgnoreRuleInputLabel),
            control: row![
                app_input(
                    app.i18n.text(TextKey::IgnoreRulePlaceholder),
                    &app.ignore_rule_input,
                    false
                )
                .on_input(Message::IgnoreRuleInputChanged)
                .on_submit(if app.ignore_rule_editing_id.is_some() {
                    Message::IgnoreRuleApplyEdit
                } else {
                    Message::IgnoreRuleAdd
                })
                .width(Length::Fill),
                ignore_input_actions
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
        ignore_chip_list,
        horizontal(),
        field(FieldSpec {
            label: app.i18n.text(TextKey::MinFileSizeLabel),
            control: row![
                app_input(
                    app.i18n.text(TextKey::MinFileSizeKbSuffix),
                    &app.min_file_size_input,
                    false
                )
                .on_input(Message::MinFileSizeInputChanged)
                .on_submit(Message::ApplyMinFileSize)
                .width(Length::Fixed(60.0)),
                text(app.i18n.text(TextKey::MinFileSizeKbSuffix))
                    .size(FONT_CAPTION)
                    .style(text_tertiary),
                app_button(
                    text(app.i18n.text(TextKey::ApplyLabel)).size(FONT_CAPTION),
                    Some(Message::ApplyMinFileSize),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
        horizontal(),
        field(FieldSpec {
            label: app.i18n.text(TextKey::DefaultShortsOutputDirLabel),
            control: row![
                app_input(
                    app.i18n.text(TextKey::DefaultShortsOutputDirPlaceholder),
                    &app.shorts_output_dir_input,
                    false
                )
                .on_input(Message::ShortsOutputDirInputChanged)
                .width(Length::Fill),
                app_button(
                    row![
                        image(assets::icon_browse(app.is_dark_theme()))
                            .width(Length::Fixed(ICON_XS))
                            .height(Length::Fixed(ICON_XS))
                            .content_fit(ContentFit::Contain)
                            .filter_method(FilterMethod::Linear),
                        text(app.i18n.text(TextKey::BrowseFolderButton)).size(FONT_CAPTION),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center),
                    Some(Message::ShortsOutputDirBrowse),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
                app_button(
                    row![
                        image(assets::icon_save(app.is_dark_theme()))
                            .width(Length::Fixed(ICON_XS))
                            .height(Length::Fixed(ICON_XS))
                            .content_fit(ContentFit::Contain)
                            .filter_method(FilterMethod::Linear),
                        text(app.i18n.text(TextKey::LibrarySaveButton)).size(FONT_CAPTION),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center),
                    Some(Message::SaveShortsOutputDirSetting),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
    ]
    .spacing(SPACE_SM);

    let diagnostics_section = {
        let state_lines = if app.diagnostics_lines.is_empty() {
            column![
                text(app.i18n.text(TextKey::DiagnosticsEmptyState))
                    .size(FONT_CAPTION)
                    .style(text_tertiary)
            ]
        } else {
            app.diagnostics_lines
                .iter()
                .fold(column![].spacing(SPACE_2XS), |col, line| {
                    col.push(
                        text(line.as_str())
                            .size(FONT_CAPTION)
                            .style(text_tertiary)
                            .font(iced::Font::MONOSPACE),
                    )
                })
        };
        let event_lines = if app.diagnostics_events.is_empty() {
            column![
                text(app.i18n.text(TextKey::DiagnosticsEmptyEvents))
                    .size(FONT_CAPTION)
                    .style(text_tertiary)
            ]
        } else {
            app.diagnostics_events
                .iter()
                .rev()
                .fold(column![].spacing(SPACE_2XS), |col, line| {
                    col.push(
                        text(line.as_str())
                            .size(FONT_CAPTION)
                            .style(text_tertiary)
                            .font(iced::Font::MONOSPACE),
                    )
                })
        };
        column![
            row![
                section_heading(app.i18n.text(TextKey::DiagnosticsSectionLabel)),
                Space::new().width(Length::Fill),
                app_button(
                    text(app.i18n.text(TextKey::RefreshButton)).size(FONT_CAPTION),
                    Some(Message::RefreshDiagnostics),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                ),
            ]
            .align_y(iced::Alignment::Center),
            text(app.i18n.text(TextKey::DiagnosticsEvents))
                .size(FONT_CAPTION)
                .style(text_secondary),
            scrollable(event_lines)
                .style(scrollbar_style)
                .height(Length::Fixed(120.0)),
            text(app.i18n.text(TextKey::DiagnosticsState))
                .size(FONT_CAPTION)
                .style(text_secondary),
            state_lines,
        ]
        .spacing(SPACE_SM)
    };

    let theme_segment = |label: &str, pref: ThemePreference| {
        let active = app.theme_preference == pref;
        tab_trigger(
            text(label.to_owned()).size(FONT_BODY),
            Some(Message::SetThemePreference(pref)),
            active,
        )
    };
    let appearance_section = column![
        section_heading(app.i18n.text(TextKey::AppearanceSectionLabel)),
        container(
            row![
                theme_segment(app.i18n.text(TextKey::ThemeSystem), ThemePreference::System),
                theme_segment(app.i18n.text(TextKey::ThemeLight), ThemePreference::Light),
                theme_segment(app.i18n.text(TextKey::ThemeDark), ThemePreference::Dark),
            ]
            .spacing(SPACE_2XS),
        )
        .padding(SPACE_2XS as u16)
        .style(raised_surface_style),
    ]
    .spacing(SPACE_SM);

    let dialog_content = column![
        appearance_section,
        horizontal(),
        indexing_section,
        horizontal(),
        ignore_section,
        horizontal(),
        diagnostics_section,
    ]
    .spacing(SPACE_MD);

    modal_overlay(dialog(DialogSpec {
        title: app.i18n.text(TextKey::SettingsDialogTitle).into(),
        close_message: Some((
            app.i18n.text(TextKey::DismissButton),
            Message::CloseSettings,
        )),
        body: dialog_content.into(),
        footer: None,
        max_width: 680.0,
        max_height: 820.0,
    }))
}

fn ignore_rule_row<'a>(
    app: &'a Librapix,
    rule: &'a librapix_storage::IgnoreRuleRecord,
) -> Element<'a, Message> {
    responsive(move |size| {
        let tone = chip_tone_for_key(&rule.pattern, app.is_dark_theme());
        let status = text(app.i18n.text(if rule.is_enabled {
            TextKey::IgnoreRuleEnabled
        } else {
            TextKey::IgnoreRuleDisabled
        }))
        .size(FONT_CAPTION)
        .color(tone.accent_text);
        let actions = row![
            app_button(
                text(app.i18n.text(if rule.is_enabled {
                    TextKey::IgnoreRuleDisableButton
                } else {
                    TextKey::IgnoreRuleAddButton
                }))
                .size(FONT_CAPTION),
                Some(Message::IgnoreRuleToggleEnabled(rule.id)),
                ButtonVariant::Ghost,
                ButtonSize::Small
            )
            .style(managed_chip_action_style(tone, false)),
            app_button(
                text(app.i18n.text(TextKey::RootEditButton)).size(FONT_CAPTION),
                Some(Message::IgnoreRuleStartEdit(rule.id)),
                ButtonVariant::Ghost,
                ButtonSize::Small
            )
            .style(managed_chip_action_style(tone, false)),
            app_tooltip(
                app_button(
                    text("x").size(FONT_CAPTION),
                    Some(Message::IgnoreRuleRemove(rule.id)),
                    ButtonVariant::Ghost,
                    ButtonSize::Small
                )
                .style(managed_chip_action_style(tone, false)),
                app.i18n.text(TextKey::RemoveIgnoreRuleHelp)
            ),
        ]
        .spacing(SPACE_2XS)
        .align_y(iced::Alignment::Center);
        let pattern = text(rule.pattern.as_str())
            .size(FONT_BODY)
            .color(tone.text)
            .width(Length::Fill);
        let content: Element<'_, Message> = if size.width < 420.0 {
            column![
                pattern,
                row![status, Space::new().width(Length::Fill), actions]
                    .align_y(iced::Alignment::Center)
            ]
            .spacing(SPACE_2XS)
            .into()
        } else {
            row![pattern, status, actions]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center)
                .into()
        };
        container(content)
            .style(managed_chip_style(tone))
            .padding([SPACE_2XS as u16, SPACE_SM as u16])
            .width(Length::Fill)
            .into()
    })
    .height(Length::Shrink)
    .into()
}
