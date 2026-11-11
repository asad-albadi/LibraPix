use super::modal_overlay;
use crate::ui::components::{
    button::{ButtonSize, ButtonVariant, app_button},
    input::app_input,
    separator::horizontal,
    tabs::tab_trigger,
    tooltip::app_tooltip,
};
use crate::ui::patterns::{
    dialog::{DialogSpec, dialog},
    field::{FieldSpec, field},
    status::{StatusTone, status_row},
};
use crate::ui::*;
use crate::{Librapix, MakeShortRunState, Message, assets, make_short_has_smooth_warning};
use iced::{
    ContentFit, Element, Length,
    widget::image::FilterMethod,
    widget::{Space, column, image, row, text},
};
use librapix_i18n::TextKey;
use librapix_video_tools::{CropPosition, Effect as ShortEffect, GenerationStage, Preset};

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let state = &app.make_short_dialog;
    let is_running = matches!(state.run_state, MakeShortRunState::Running { .. });
    let effect_chip = |effect: ShortEffect, help_key: TextKey| {
        row![
            tab_trigger(
                text(effect.as_str()).size(FONT_CAPTION),
                (!is_running).then_some(Message::MakeShortToggleEffect(effect)),
                state.effects.contains(&effect)
            ),
            help(app.i18n.text(help_key)),
        ]
        .spacing(SPACE_XS)
    };
    let crop_chip = |position: CropPosition, label: &'static str| {
        tab_trigger(
            text(label).size(FONT_CAPTION),
            (!is_running).then_some(Message::MakeShortSetCropPosition(position)),
            state.crop_position == position,
        )
    };
    let preset_chip = |preset: Preset, label: &'static str| {
        tab_trigger(
            text(label).size(FONT_CAPTION),
            (!is_running).then_some(Message::MakeShortSetPreset(preset)),
            state.preset == preset,
        )
    };
    let status: Element<'_, Message> = match &state.run_state {
        MakeShortRunState::Running { stage, status } => {
            let tone = match stage {
                GenerationStage::Completed => StatusTone::Success,
                GenerationStage::Failed => StatusTone::Error,
                _ => StatusTone::Working,
            };
            status_row(status, tone, app.new_media_preview_loading_phase)
        }
        MakeShortRunState::Success { .. } => status_row(
            app.i18n.text(TextKey::MakeShortSuccessLabel),
            StatusTone::Success,
            0,
        ),
        MakeShortRunState::Canceled { summary } => status_row(summary, StatusTone::Neutral, 0),
        MakeShortRunState::Failed { summary, details } => column![
            status_row(summary, StatusTone::Error, 0),
            text(details).size(FONT_CAPTION).style(text_tertiary),
        ]
        .spacing(SPACE_2XS)
        .into(),
        MakeShortRunState::Idle => Space::new().into(),
    };
    let warning: Element<'_, Message> = if make_short_has_smooth_warning(state) {
        text(app.i18n.text(TextKey::MakeShortSmoothWarning))
            .size(FONT_CAPTION)
            .style(crate::ui::styles::text::text_error)
            .into()
    } else {
        Space::new().into()
    };
    let validation: Element<'_, Message> = if let Some(error) = &state.validation_error {
        status_row(error, StatusTone::Error, 0)
    } else {
        Space::new().into()
    };
    let mut actions = if is_running {
        row![app_button(
            text(app.i18n.text(TextKey::MakeShortCancelButton)).size(FONT_BODY),
            state
                .cancellation
                .as_ref()
                .map(|_| Message::CancelMakeShortGeneration),
            ButtonVariant::Ghost,
            ButtonSize::Default,
        )]
    } else {
        row![
            app_button(
                text(app.i18n.text(TextKey::MakeShortCloseButton)).size(FONT_BODY),
                Some(Message::CloseMakeShortDialog),
                ButtonVariant::Ghost,
                ButtonSize::Default
            ),
            app_button(
                row![
                    image(assets::icon_generate(app.is_dark_theme()))
                        .width(ICON_SM)
                        .height(ICON_SM)
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(app.i18n.text(TextKey::MakeShortRunButton)).size(FONT_BODY),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center),
                Some(Message::RunMakeShort),
                ButtonVariant::Primary,
                ButtonSize::Default
            ),
        ]
    }
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);
    if matches!(state.run_state, MakeShortRunState::Success { .. }) {
        actions = actions.push(app_button(
            text(app.i18n.text(TextKey::MakeShortOpenFileButton)).size(FONT_BODY),
            Some(Message::OpenGeneratedShortFile),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        ));
        actions = actions.push(app_button(
            text(app.i18n.text(TextKey::MakeShortOpenFolderButton)).size(FONT_BODY),
            Some(Message::OpenGeneratedShortFolder),
            ButtonVariant::Secondary,
            ButtonSize::Default,
        ));
    }
    let output_row = row![
        app_input(
            app.i18n.text(TextKey::MakeShortOutputPathLabel),
            &state.output_file_input,
            false
        )
        .on_input(Message::MakeShortOutputPathChanged)
        .width(Length::Fill),
        app_button(
            row![
                image(assets::icon_browse(app.is_dark_theme()))
                    .width(ICON_XS)
                    .height(ICON_XS)
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::MakeShortChooseOutputButton)).size(FONT_CAPTION),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
            Some(Message::MakeShortBrowseOutputPath),
            ButtonVariant::Ghost,
            ButtonSize::Small
        ),
        help(app.i18n.text(TextKey::MakeShortHelpOutput)),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);
    let body = column![
        field(FieldSpec {
            label: app.i18n.text(TextKey::MakeShortOutputPathLabel),
            control: output_row.into(),
            help: None,
            error: None
        }),
        section_heading(app.i18n.text(TextKey::MakeShortEffectsLabel)),
        row![
            effect_chip(ShortEffect::Clean, TextKey::MakeShortHelpEffectsClean),
            effect_chip(ShortEffect::Enhanced, TextKey::MakeShortHelpEffectsEnhanced),
            effect_chip(
                ShortEffect::Cinematic,
                TextKey::MakeShortHelpEffectsCinematic
            ),
        ]
        .spacing(SPACE_SM),
        row![
            effect_chip(ShortEffect::Night, TextKey::MakeShortHelpEffectsNight),
            effect_chip(ShortEffect::Scenic, TextKey::MakeShortHelpEffectsScenic),
            effect_chip(ShortEffect::Smooth, TextKey::MakeShortHelpEffectsSmooth),
        ]
        .spacing(SPACE_SM),
        warning,
        horizontal(),
        field(FieldSpec {
            label: app.i18n.text(TextKey::MakeShortCropLabel),
            control: row![
                crop_chip(CropPosition::Center, "center"),
                crop_chip(CropPosition::Left, "left"),
                crop_chip(CropPosition::Right, "right"),
                help(app.i18n.text(TextKey::MakeShortHelpCrop))
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
        row![
            text(app.i18n.text(TextKey::MakeShortFadeLabel))
                .size(FONT_CAPTION)
                .style(text_secondary),
            tab_trigger(
                text(if state.add_fade { "On" } else { "Off" }).size(FONT_CAPTION),
                (!is_running).then_some(Message::MakeShortSetAddFade(!state.add_fade)),
                state.add_fade
            ),
            help(app.i18n.text(TextKey::MakeShortHelpFade)),
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
        horizontal(),
        field(FieldSpec {
            label: app.i18n.text(TextKey::MakeShortSpeedLabel),
            control: row![
                app_input("1.0", &state.speed_input, false)
                    .on_input(Message::MakeShortSpeedChanged)
                    .width(80),
                help(app.i18n.text(TextKey::MakeShortHelpSpeed))
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
        field(FieldSpec {
            label: app.i18n.text(TextKey::MakeShortCrfLabel),
            control: row![
                app_input("18", &state.crf_input, false)
                    .on_input(Message::MakeShortCrfChanged)
                    .width(80),
                help(app.i18n.text(TextKey::MakeShortHelpCrf))
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
        field(FieldSpec {
            label: app.i18n.text(TextKey::MakeShortPresetLabel),
            control: row![
                preset_chip(Preset::Fast, "fast"),
                preset_chip(Preset::Medium, "medium"),
                preset_chip(Preset::Slow, "slow"),
                help(app.i18n.text(TextKey::MakeShortHelpPreset))
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center)
            .into(),
            help: None,
            error: None,
        }),
        validation,
    ]
    .spacing(SPACE_SM);
    modal_overlay(dialog(DialogSpec {
        title: app.i18n.text(TextKey::MakeShortDialogTitle).into(),
        close_message: (!is_running).then_some((
            app.i18n.text(TextKey::DismissButton),
            Message::CloseMakeShortDialog,
        )),
        body: body.into(),
        footer: Some(column![status, actions].spacing(SPACE_SM).into()),
        max_width: 620.0,
        max_height: 780.0,
    }))
}

fn help(label: &str) -> Element<'_, Message> {
    app_tooltip(text("(?)").size(FONT_CAPTION).style(text_tertiary), label).into()
}
