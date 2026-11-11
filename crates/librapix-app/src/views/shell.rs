//! Product shell composition. Update/persistence/background work stay in the app.
use crate::ui::components::text::section_heading;
use crate::ui::{
    styles::{
        container::{app_bg_style, header_style},
        scrollable::scrollbar_style,
        text::{text_accent, text_primary, text_tertiary, text_warning},
    },
    tokens::*,
};

use super::dialogs;

use crate::ui::components::{
    badge::{BadgeVariant, badge},
    button::{ButtonSize, ButtonVariant, app_button},
    icon_button::icon_button,
    input::app_input,
    notice::notice,
    separator::{horizontal, vertical},
    tabs::tab_trigger,
    tooltip::app_tooltip,
};
use crate::ui::patterns::status::{StatusTone, status_row};
use crate::ui::patterns::{
    details_pane::{DetailsPaneMode, details_pane},
    sidebar::{GlobalRailAction, SidebarMode, navigation_item, sidebar},
};
use crate::{
    ActivityIndicatorMode, Librapix, Message, UpdateCheckState, activity_indicator_mode, assets,
    display_name_for_root, theme, update_chip_text_key,
};
use iced::widget::image::FilterMethod;
use iced::widget::progress_bar;
use iced::widget::{Space, column, container, image, row, scrollable, stack, svg, text};
use iced::{ContentFit, Element, Length};
use librapix_core::{
    app::{RootLifecycle, Route},
    domain::non_destructive,
};
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    let _required_rules = non_destructive::required_rules();

    // ── Header ──
    let brand = row![
        svg(assets::logo_svg())
            .width(Length::Fixed(ICON_XL))
            .height(Length::Fixed(ICON_XL))
            .content_fit(ContentFit::Contain),
        row![
            text("Libra").size(FONT_DISPLAY).style(text_primary),
            text("Pix").size(FONT_DISPLAY).style(text_accent),
        ]
        .spacing(0)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    let settings_btn = app_button(
        text(app.i18n.text(TextKey::SettingsButtonLabel)).size(FONT_BODY),
        Some(Message::OpenSettings),
        ButtonVariant::Ghost,
        ButtonSize::Default,
    );
    let about_btn = app_button(
        text(app.i18n.text(TextKey::AboutButtonLabel)).size(FONT_BODY),
        Some(Message::OpenAbout),
        ButtonVariant::Ghost,
        ButtonSize::Default,
    );

    let github_btn = icon_button(
        image(assets::icon_github(app.is_dark_theme()))
            .width(Length::Fixed(ICON_LG))
            .height(Length::Fixed(ICON_LG))
            .content_fit(ContentFit::Contain)
            .filter_method(FilterMethod::Linear)
            .into(),
        app.i18n.text(TextKey::GitHubButtonLabel),
        Some(Message::OpenGitHub),
        ButtonVariant::Ghost,
    );

    let header = container(
        row![
            brand,
            Space::new().width(Length::Fill),
            row![
                image(assets::icon_search(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_MD))
                    .height(Length::Fixed(ICON_MD))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                app_input(
                    app.i18n.text(TextKey::SearchInputLabel),
                    &app.state.search_query,
                    false
                )
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::RunSearchQuery)
                .width(Length::Fixed(380.0)),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Space::new().width(Length::Fill),
            settings_btn,
            about_btn,
            github_btn,
        ]
        .spacing(SPACE_SM)
        .align_y(iced::Alignment::Center),
    )
    .padding([0, SPACE_XL as u16])
    .center_y(HEADER_HEIGHT)
    .width(Length::Fill)
    .style(header_style);

    let sidebar_mode = SidebarMode::from_collapsed(app.left_sidebar_collapsed);
    let sidebar = sidebar(
        sidebar_mode,
        |control| render_expanded_sidebar(app, control),
        icon_button(
            text("\u{2039}").size(ICON_LG).into(),
            app.i18n.text(TextKey::CollapseLibraries),
            Some(Message::ToggleLeftSidebar),
            ButtonVariant::Ghost,
        ),
        |action| rail_action(app, action),
    );

    // ── Details pane ──
    let details_mode = DetailsPaneMode::from_collapsed(app.details_sidebar_collapsed);
    let details = details_pane(
        details_mode,
        app.i18n.text(TextKey::DetailsLabel),
        app.i18n.text(if app.details_sidebar_collapsed {
            TextKey::ExpandDetails
        } else {
            TextKey::CollapseDetails
        }),
        Message::ToggleDetailsSidebar,
        || super::details::view(app),
    );

    // ── Body ──
    let body = row![
        sidebar,
        vertical(),
        container(super::library::view(app))
            .padding([SPACE_LG as u16, SPACE_XL as u16])
            .width(Length::Fill),
        vertical(),
        details,
    ]
    .height(Length::Fill);

    let shell: Element<'_, Message> = container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app_bg_style)
        .into();

    // Keep the shell at the same tree position so notices preserve widget state.
    // Modals are stacked afterwards so their input remains above the notice.
    let mut shell_layers = stack([shell]).width(Length::Fill).height(Length::Fill);
    if let Some(state) = &app.shell_notice {
        shell_layers = shell_layers.push(
            container(notice(
                state,
                app.i18n.text(TextKey::ErrorActionFailedLabel),
                Some((
                    app.i18n.text(TextKey::DismissButton),
                    Message::DismissShellNotice,
                )),
            ))
            .align_right(Length::Fill)
            .align_bottom(Length::Fill)
            .padding(SPACE_LG as u16),
        );
    }
    let mut overlay: Element<'_, Message> = shell_layers.into();
    if app.filter_dialog_open {
        overlay = stack([overlay, dialogs::filters::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.new_media_announcement.is_some() {
        overlay = stack([overlay, dialogs::new_media::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.settings_open {
        overlay = stack([overlay, dialogs::settings::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.about_open {
        overlay = stack([overlay, dialogs::about::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.library_dialog_open {
        overlay = stack([overlay, dialogs::library::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.library_stats_dialog_open {
        overlay = stack([overlay, dialogs::library_statistics::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.make_short_dialog.open {
        overlay = stack([overlay, dialogs::make_short::view(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    overlay
}

fn rail_action(app: &Librapix, action: GlobalRailAction) -> Element<'_, Message> {
    let dark = app.is_dark_theme();
    let icon = |handle| -> Element<'_, Message> {
        image(handle)
            .width(ICON_MD)
            .height(ICON_MD)
            .content_fit(ContentFit::Contain)
            .filter_method(FilterMethod::Linear)
            .into()
    };
    let (content, label, message) = match action {
        GlobalRailAction::Library => (
            icon(assets::icon_gallery(dark)),
            TextKey::LibraryTab,
            Message::OpenGallery,
        ),
        GlobalRailAction::AddLibrary => (
            icon(assets::icon_browse(dark)),
            TextKey::LibraryAddButtonLabel,
            Message::OpenAddLibraryDialog,
        ),
        GlobalRailAction::Refresh => (
            icon(assets::icon_refresh(dark)),
            TextKey::RootRefreshButton,
            Message::RefreshRoots,
        ),
        GlobalRailAction::Expand => (
            text("\u{203A}").size(ICON_LG).into(),
            TextKey::ExpandLibraries,
            Message::ToggleLeftSidebar,
        ),
    };
    icon_button(
        content,
        app.i18n.text(label),
        Some(message),
        ButtonVariant::Ghost,
    )
}

fn render_expanded_sidebar<'a>(
    app: &'a Librapix,
    collapse_control: Element<'a, Message>,
) -> Element<'a, Message> {
    let is_gallery = matches!(app.state.active_route, Route::Gallery);
    let is_timeline = matches!(app.state.active_route, Route::Timeline);
    // ── Sidebar: Browse navigation ──
    // Gallery + Timeline are one "Library" surface; the Grid/Timeline split is a
    // toggle in the content header, so the sidebar carries a single entry.
    let nav_section = column![
        row![
            section_heading(app.i18n.text(TextKey::BrowseSectionLabel)),
            Space::new().width(Length::Fill),
            collapse_control,
        ]
        .align_y(iced::Alignment::Center),
        tab_trigger(
            row![
                image(assets::icon_gallery(app.is_dark_theme()))
                    .width(Length::Fixed(ICON_MD))
                    .height(Length::Fixed(ICON_MD))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(app.i18n.text(TextKey::LibraryTab)).size(FONT_BODY),
            ]
            .spacing(SPACE_SM)
            .align_y(iced::Alignment::Center),
            Some(Message::OpenGallery),
            is_gallery || is_timeline
        )
        .width(Length::Fill),
    ]
    .spacing(SPACE_XS);

    // ── Sidebar: Library roots ──
    let roots_list: Element<'_, Message> = if app.state.library_roots.is_empty() {
        text(app.i18n.text(TextKey::EmptyRootsLabel))
            .size(FONT_BODY)
            .style(text_tertiary)
            .into()
    } else {
        app.state
            .library_roots
            .iter()
            .fold(column![].spacing(SPACE_2XS), |col, root| {
                let is_selected = app.state.selected_root_id == Some(root.id);
                let label = display_name_for_root(root);
                let p = palette(&theme(app));
                let status_color = match root.lifecycle {
                    RootLifecycle::Active => p.success,
                    RootLifecycle::Unavailable => p.warning,
                    RootLifecycle::Deactivated => p.text_disabled,
                };
                col.push(
                    row![
                        navigation_item(
                            row![
                                text("\u{25CF}").size(FONT_CAPTION).color(status_color),
                                text(label).size(FONT_BODY).color(if is_selected {
                                    p.text_primary
                                } else {
                                    p.text_secondary
                                }),
                            ]
                            .spacing(SPACE_SM)
                            .align_y(iced::Alignment::Center),
                            Some(Message::SelectRoot(root.id)),
                            is_selected
                        ),
                        app_button(
                            text(app.i18n.text(TextKey::RootEditButton)).size(FONT_CAPTION),
                            Some(Message::OpenEditLibraryDialog(root.id)),
                            ButtonVariant::Ghost,
                            ButtonSize::Small
                        ),
                        app_button(
                            text(app.i18n.text(TextKey::RootStatsButton)).size(FONT_CAPTION),
                            Some(Message::OpenLibraryStatisticsDialog(root.id)),
                            ButtonVariant::Ghost,
                            ButtonSize::Small
                        ),
                    ]
                    .spacing(SPACE_XS)
                    .align_y(iced::Alignment::Center),
                )
            })
            .into()
    };
    let library_section = column![
        section_heading(app.i18n.text(TextKey::LibrarySectionLabel)),
        roots_list,
        row![
            app_button(
                text(app.i18n.text(TextKey::LibraryAddButtonLabel)).size(FONT_BODY),
                Some(Message::OpenAddLibraryDialog),
                ButtonVariant::Secondary,
                ButtonSize::Default
            ),
            app_button(
                text(app.i18n.text(TextKey::RootRefreshButton)).size(FONT_BODY),
                Some(Message::RefreshRoots),
                ButtonVariant::Ghost,
                ButtonSize::Default
            ),
        ]
        .spacing(SPACE_XS),
        text(app.root_status.clone())
            .size(FONT_CAPTION)
            .style(text_tertiary),
    ]
    .spacing(SPACE_SM);

    let sidebar_content = scrollable(
        column![nav_section, horizontal(), library_section]
            .spacing(SPACE_LG)
            .padding(SPACE_LG as u16),
    )
    .style(scrollbar_style)
    .height(Length::Fill);
    let sidebar_status = container(
        column![
            section_heading(app.i18n.text(TextKey::StatusSectionLabel)),
            render_update_chip(app).width(Length::Fill),
            render_activity_status(app),
        ]
        .spacing(SPACE_XS),
    )
    .width(Length::Fill)
    .padding([SPACE_SM as u16, SPACE_MD as u16]);
    container(
        column![sidebar_content, horizontal(), sidebar_status]
            .height(Length::Fill)
            .spacing(SPACE_XS),
    )
    .width(Length::Fill)
    .into()
}

fn render_activity_status(app: &Librapix) -> Element<'_, Message> {
    let progress = &app.activity_progress;
    let indicator_mode = activity_indicator_mode(progress);
    let progress_line = match indicator_mode {
        ActivityIndicatorMode::Determinate { total, done } => format!(
            "{} {} / {}",
            app.i18n.text(TextKey::ProgressItemsLabel),
            done,
            total
        ),
        ActivityIndicatorMode::Indeterminate => {
            format!("{} --", app.i18n.text(TextKey::ProgressItemsLabel))
        }
        ActivityIndicatorMode::Idle => {
            format!("{} 0 / 0", app.i18n.text(TextKey::ProgressItemsLabel))
        }
    };
    let queue_line = format!(
        "{} {}",
        app.i18n.text(TextKey::ProgressQueueLabel),
        progress.queue_depth
    );
    let roots_line = progress.roots_total.map(|total| {
        format!(
            "{} {} / {}",
            app.i18n.text(TextKey::ProgressRootsLabel),
            progress.roots_done.min(total),
            total
        )
    });

    if matches!(indicator_mode, ActivityIndicatorMode::Idle) {
        return status_row(&app.activity_status, StatusTone::Success, 0);
    }

    let indicator: Element<'_, Message> = match indicator_mode {
        ActivityIndicatorMode::Determinate { total, done } => {
            let capped_total = total.max(1) as f32;
            let done = done.min(total) as f32;
            container(
                progress_bar(0.0..=capped_total, done)
                    .style(crate::ui::styles::progress::activity_progress_style),
            )
            .height(Length::Fixed(4.0))
            .into()
        }
        ActivityIndicatorMode::Indeterminate => badge(
            app.i18n.text(TextKey::ActivityWorkingLabel),
            BadgeVariant::Secondary,
        )
        .into(),
        ActivityIndicatorMode::Idle => Space::new().into(),
    };

    let mut metrics = vec![progress_line, queue_line];
    if let Some(roots_line) = roots_line {
        metrics.push(roots_line);
    }
    let metrics_line = metrics.join(" | ");

    let mut lines = column![
        text(progress.stage_text.clone())
            .size(FONT_BODY)
            .style(text_primary),
        indicator,
        text(metrics_line).size(FONT_CAPTION).style(text_tertiary),
    ]
    .spacing(SPACE_2XS);
    if !progress.detail_text.trim().is_empty() {
        lines = lines.push(
            text(progress.detail_text.clone())
                .size(FONT_CAPTION)
                .style(text_tertiary),
        );
    }
    if let Some(error) = progress.last_error.as_ref() {
        let error_line = format!("{}: {}", app.i18n.text(TextKey::ProgressErrorLabel), error);
        let (error_compact, error_truncated) = truncate_for_sidebar(&error_line, 120);
        let error_text = text(error_compact)
            .size(FONT_CAPTION)
            .style(text_warning)
            .width(Length::Fill);
        let error_widget: Element<'_, Message> = if error_truncated {
            app_tooltip(error_text, error_line).into()
        } else {
            error_text.into()
        };
        lines = lines.push(error_widget);
    }

    container(lines).width(Length::Fill).into()
}

fn render_update_chip(app: &Librapix) -> iced::widget::Button<'_, Message> {
    let variant = match app.update_check_state {
        UpdateCheckState::UpdateAvailable { .. } => ButtonVariant::Primary,
        UpdateCheckState::Checking | UpdateCheckState::UpToDate => ButtonVariant::Secondary,
        UpdateCheckState::Unknown | UpdateCheckState::Failed => ButtonVariant::Ghost,
    };
    app_button(
        text(app.i18n.text(update_chip_text_key(&app.update_check_state))).size(FONT_CAPTION),
        (!matches!(app.update_check_state, UpdateCheckState::Checking))
            .then_some(Message::UpdateChipPressed),
        variant,
        ButtonSize::Small,
    )
    .width(Length::Fill)
}

fn truncate_for_sidebar(value: &str, max_chars: usize) -> (String, bool) {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        return (format!("{truncated}..."), true);
    }
    (value.to_owned(), false)
}
