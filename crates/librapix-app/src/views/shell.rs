//! Product shell composition. Update/persistence/background work stay in the app.

use crate::ui::components::{button::ButtonVariant, icon_button::icon_button, notice::notice};
use crate::ui::patterns::{
    details_pane::{DetailsPaneMode, details_pane},
    sidebar::{GlobalRailAction, SidebarMode, sidebar},
};
use crate::ui::{self, *};
use crate::{
    Librapix, MEDIA_SCROLLABLE_ID, MEDIA_SCROLLBAR_SPACING, Message, assets, display_name_for_root,
    render_about_dialog, render_activity_status, render_details_panel, render_filter_dialog,
    render_library_dialog, render_library_statistics_dialog, render_make_short_dialog,
    render_media_panel, render_new_media_dialog, render_settings_dialog, render_timeline_scrubber,
    render_update_chip, theme,
};
use iced::widget::image::FilterMethod;
use iced::widget::{
    Id, Space, button, column, container, image, row, scrollable, stack, svg, text, text_input,
};
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

    let settings_btn = button(text(app.i18n.text(TextKey::SettingsButtonLabel)).size(FONT_BODY))
        .on_press(Message::OpenSettings)
        .style(subtle_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);
    let about_btn = button(text(app.i18n.text(TextKey::AboutButtonLabel)).size(FONT_BODY))
        .on_press(Message::OpenAbout)
        .style(subtle_button_style)
        .padding([SPACE_XS as u16, SPACE_MD as u16]);

    let github_btn = button(
        image(assets::icon_github(app.is_dark_theme()))
            .width(Length::Fixed(ICON_LG))
            .height(Length::Fixed(ICON_LG))
            .content_fit(ContentFit::Contain)
            .filter_method(FilterMethod::Linear),
    )
    .on_press(Message::OpenGitHub)
    .style(subtle_button_style)
    .padding([SPACE_XS as u16, SPACE_XS as u16]);

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
                text_input(
                    app.i18n.text(TextKey::SearchInputLabel),
                    &app.state.search_query
                )
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::RunSearchQuery)
                .width(Length::Fixed(380.0))
                .style(search_input_style),
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
        || render_expanded_sidebar(app),
        icon_button(
            text("\u{2039}").size(ICON_LG).into(),
            app.i18n.text(TextKey::CollapseLibraries),
            Some(Message::ToggleLeftSidebar),
            ButtonVariant::Ghost,
        ),
        |action| rail_action(app, action),
    );

    // ── Media pane ──
    let (media_header, media_scrollable_content) = render_media_panel(app);
    let base_media_scrollable = scrollable(media_scrollable_content)
        .id(Id::new(MEDIA_SCROLLABLE_ID))
        .style(scrollbar_style)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::default().spacing(MEDIA_SCROLLBAR_SPACING),
        ))
        .on_scroll(|viewport| Message::MediaViewportChanged {
            absolute_y: viewport.absolute_offset().y,
            max_y: (viewport.content_bounds().height - viewport.bounds().height).max(0.0),
            viewport_height: viewport.bounds().height.max(0.0),
        })
        .height(Length::Fill);
    let media_scrollable: Element<'_, Message> = base_media_scrollable.into();
    let media_body: Element<'_, Message> = if matches!(app.state.active_route, Route::Timeline) {
        row![
            container(media_scrollable)
                .width(Length::Fill)
                .height(Length::Fill),
            render_timeline_scrubber(app),
        ]
        .spacing(SPACE_SM)
        .height(Length::Fill)
        .into()
    } else {
        container(media_scrollable)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

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
        || render_details_panel(app),
    );

    // ── Body ──
    let body = row![
        sidebar,
        ui::v_divider(),
        container(column![media_header, media_body,].spacing(SPACE_SM),)
            .padding([SPACE_LG as u16, SPACE_XL as u16])
            .width(Length::Fill),
        ui::v_divider(),
        details,
    ]
    .height(Length::Fill);

    let shell: Element<'_, Message> = container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(app_bg_style)
        .into();

    // Alignment layers overlay feedback without consuming media-layout space.
    // Modals are stacked afterwards so their input remains above the notice.
    let mut overlay: Element<'_, Message> = if let Some(state) = &app.shell_notice {
        stack([
            shell,
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
            .padding(SPACE_LG as u16)
            .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        shell
    };
    if app.filter_dialog_open {
        overlay = stack([overlay, render_filter_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.new_media_announcement.is_some() {
        overlay = stack([overlay, render_new_media_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.settings_open {
        overlay = stack([overlay, render_settings_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.about_open {
        overlay = stack([overlay, render_about_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.library_dialog_open {
        overlay = stack([overlay, render_library_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.library_stats_dialog_open {
        overlay = stack([overlay, render_library_statistics_dialog(app)])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }
    if app.make_short_dialog.open {
        overlay = stack([overlay, render_make_short_dialog(app)])
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

fn render_expanded_sidebar(app: &Librapix) -> Element<'_, Message> {
    let is_gallery = matches!(app.state.active_route, Route::Gallery);
    let is_timeline = matches!(app.state.active_route, Route::Timeline);
    // ── Sidebar: Browse navigation ──
    // Gallery + Timeline are one "Library" surface; the Grid/Timeline split is a
    // toggle in the content header, so the sidebar carries a single entry.
    let nav_section = column![
        section_heading(app.i18n.text(TextKey::BrowseSectionLabel)),
        button(
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
        )
        .width(Length::Fill)
        .on_press(Message::OpenGallery)
        .style(nav_button_style(is_gallery || is_timeline))
        .padding([SPACE_SM as u16, SPACE_MD as u16]),
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
                let p = ui::palette(&theme(app));
                let status_color = match root.lifecycle {
                    RootLifecycle::Active => p.success,
                    RootLifecycle::Unavailable => p.warning,
                    RootLifecycle::Deactivated => p.text_disabled,
                };
                col.push(
                    row![
                        button(
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
                        )
                        .width(Length::Fill)
                        .on_press(Message::SelectRoot(root.id))
                        .style(nav_button_style(is_selected))
                        .padding([SPACE_XS as u16, SPACE_SM as u16]),
                        button(text(app.i18n.text(TextKey::RootEditButton)).size(FONT_CAPTION))
                            .on_press(Message::OpenEditLibraryDialog(root.id))
                            .style(subtle_button_style)
                            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
                        button(text(app.i18n.text(TextKey::RootStatsButton)).size(FONT_CAPTION))
                            .on_press(Message::OpenLibraryStatisticsDialog(root.id))
                            .style(subtle_button_style)
                            .padding([SPACE_2XS as u16, SPACE_SM as u16]),
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
            button(text(app.i18n.text(TextKey::LibraryAddButtonLabel)).size(FONT_BODY))
                .on_press(Message::OpenAddLibraryDialog)
                .style(action_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
            button(text(app.i18n.text(TextKey::RootRefreshButton)).size(FONT_BODY))
                .on_press(Message::RefreshRoots)
                .style(subtle_button_style)
                .padding([SPACE_XS as u16, SPACE_MD as u16]),
        ]
        .spacing(SPACE_XS),
        text(app.root_status.clone())
            .size(FONT_CAPTION)
            .style(text_tertiary),
    ]
    .spacing(SPACE_SM);

    let sidebar_content = scrollable(
        column![nav_section, h_divider(), library_section]
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
        column![sidebar_content, h_divider(), sidebar_status]
            .height(Length::Fill)
            .spacing(SPACE_XS),
    )
    .width(Length::Fill)
    .into()
}
