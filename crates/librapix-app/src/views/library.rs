//! Library presentation; layout, caches, viewport math and orchestration stay in the app.
use crate::ui::components::{
    badge::selection_check,
    button::ButtonVariant,
    card::{CardVariant, card, interactive_card},
    empty_state::empty_state,
    icon_button::icon_button,
    tabs::{tab_list, tab_trigger},
};
use crate::ui::{
    styles::{
        container::{thumb_placeholder_style, thumb_scrim_style},
        scrollable::scrollbar_style,
        text::{text_primary, text_secondary, text_tertiary},
    },
    tokens::*,
};
use crate::{
    BrowseItem, Librapix, MEDIA_SCROLLABLE_ID, MEDIA_SCROLLBAR_SPACING, Message, assets,
    compute_browse_stats,
};
use iced::widget::image::FilterMethod;
use iced::{
    ContentFit, Element, Length,
    widget::{Id, Space, column, container, image, row, scrollable, stack, text},
};
use librapix_core::app::Route;
use librapix_i18n::TextKey;

pub(crate) fn view(app: &Librapix) -> Element<'_, Message> {
    // ── Media pane ──
    let (media_header, media_scrollable_content) = panels(app);
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
            super::timeline::scrubber(app),
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

    column![media_header, media_body].spacing(SPACE_SM).into()
}

pub(crate) fn panels(app: &Librapix) -> (Element<'_, Message>, Element<'_, Message>) {
    let is_grid = matches!(app.state.active_route, Route::Gallery);
    let run_msg = match app.state.active_route {
        Route::Gallery => Message::RunGalleryProjection,
        Route::Timeline => Message::RunTimelineProjection,
    };
    // One canonical projection pass feeds both: the flat grid renders the
    // header-stripped `gallery_items`, the timeline renders the grouped feed.
    let browse_items = match app.state.active_route {
        Route::Gallery => &app.gallery_items,
        Route::Timeline => &app.timeline_items,
    };
    let stats_source = if app.state.search_query.trim().is_empty() {
        browse_items
    } else {
        &app.search_items
    };
    let stats = compute_browse_stats(stats_source);

    // Segmented Grid | Timeline toggle (Google/Windows Photos style).
    let dark = app.is_dark_theme();
    let segment_label = |handle, label: &str, active: bool, msg: Message| {
        tab_trigger(
            row![
                image(handle)
                    .width(Length::Fixed(ICON_SM))
                    .height(Length::Fixed(ICON_SM))
                    .content_fit(ContentFit::Contain)
                    .filter_method(FilterMethod::Linear),
                text(label.to_owned()).size(FONT_BODY),
            ]
            .spacing(SPACE_XS)
            .align_y(iced::Alignment::Center),
            Some(msg),
            active,
        )
    };
    let view_toggle = card(
        tab_list([
            segment_label(
                assets::icon_gallery(dark),
                app.i18n.text(TextKey::GalleryTab),
                is_grid,
                Message::OpenGallery,
            )
            .into(),
            segment_label(
                assets::icon_timeline(dark),
                app.i18n.text(TextKey::TimelineTab),
                !is_grid,
                Message::OpenTimeline,
            )
            .into(),
        ]),
        CardVariant::Default,
    )
    .padding(SPACE_2XS as u16);

    let content_header = row![
        text(app.i18n.text(TextKey::LibraryTab))
            .size(FONT_TITLE)
            .style(text_primary),
        view_toggle,
        Space::new().width(Length::Fill),
        icon_button(
            (image(assets::icon_refresh(app.is_dark_theme()))
                .width(Length::Fixed(ICON_MD))
                .height(Length::Fixed(ICON_MD))
                .content_fit(ContentFit::Contain)
                .filter_method(FilterMethod::Linear))
            .into(),
            app.i18n.text(TextKey::RootRefreshButton),
            Some(run_msg),
            ButtonVariant::Ghost
        ),
        icon_button(
            (image(assets::icon_filter(app.is_dark_theme()))
                .width(Length::Fixed(ICON_MD))
                .height(Length::Fixed(ICON_MD))
                .content_fit(ContentFit::Contain)
                .filter_method(FilterMethod::Linear))
            .into(),
            app.i18n.text(TextKey::FiltersButtonLabel),
            Some(Message::ToggleFilterDialog),
            ButtonVariant::Ghost
        ),
        text(format!(
            "{}: {} \u{00B7} {}: {} \u{00B7} {}: {}",
            app.i18n.text(TextKey::StatsTotalLabel),
            stats.shown_items,
            app.i18n.text(TextKey::StatsImagesLabel),
            stats.image_count,
            app.i18n.text(TextKey::StatsVideosLabel),
            stats.video_count
        ))
        .size(FONT_BODY)
        .style(text_secondary),
    ]
    .spacing(SPACE_SM)
    .align_y(iced::Alignment::Center);

    let header: Element<'_, Message> = content_header.into();

    let search_section: Element<'_, Message> = if !app.state.search_query.trim().is_empty() {
        if app.search_items.is_empty() {
            render_empty_state(
                assets::icon_search(app.is_dark_theme()),
                app.i18n.text(TextKey::EmptySearchResultsLabel),
            )
        } else {
            let search_header = row![
                text(app.i18n.text(TextKey::SearchResultLabel))
                    .size(FONT_SUBTITLE)
                    .style(text_primary),
                text(format!("({})", app.search_items.len()))
                    .size(FONT_BODY)
                    .style(text_secondary),
            ]
            .spacing(SPACE_SM);

            column![
                search_header,
                super::gallery::view(
                    app,
                    &app.search_items,
                    app.state.selected_media_id,
                    app.media_scroll_absolute_y,
                    app.media_viewport_height,
                ),
            ]
            .spacing(SPACE_SM)
            .into()
        }
    } else {
        column![].into()
    };

    let empty_label = match app.state.active_route {
        Route::Gallery => app.i18n.text(TextKey::EmptyGalleryLabel),
        Route::Timeline => app.i18n.text(TextKey::EmptyTimelineLabel),
    };

    let browse_content: Element<'_, Message> = if browse_items.is_empty() {
        render_empty_state(assets::icon_gallery(app.is_dark_theme()), empty_label)
    } else {
        match app.state.active_route {
            Route::Gallery => super::gallery::view(
                app,
                browse_items,
                app.state.selected_media_id,
                app.media_scroll_absolute_y,
                app.media_viewport_height,
            ),
            Route::Timeline => super::timeline::view(
                app,
                browse_items,
                app.state.selected_media_id,
                app.media_scroll_absolute_y,
                app.media_viewport_height,
            ),
        }
    };

    let scrollable_content: Element<'_, Message> = column![search_section, browse_content]
        .spacing(SPACE_LG)
        .into();

    (header, scrollable_content)
}

pub(crate) fn media_card(
    item: &BrowseItem,
    selected: bool,
    height: f32,
    dark: bool,
) -> Element<'_, Message> {
    let thumb: Element<'_, Message> = if let Some(path) = &item.thumbnail_path {
        image(image::Handle::from_path(path))
            .width(Length::Fill)
            .height(Length::Fixed(height))
            .content_fit(ContentFit::Cover)
            .into()
    } else {
        container(
            column![
                Space::new().height(Length::Fill),
                text(item.title.clone())
                    .size(FONT_CAPTION)
                    .style(text_tertiary),
            ]
            .padding(SPACE_XS as u16),
        )
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(thumb_placeholder_style)
        .into()
    };

    let kind_icon_handle = if item.media_kind.eq_ignore_ascii_case("video") {
        assets::icon_type_video(dark)
    } else {
        assets::icon_type_image(dark)
    };
    let kind_badge = image(kind_icon_handle)
        .width(Length::Fixed(ICON_SM))
        .height(Length::Fixed(ICON_SM))
        .content_fit(ContentFit::Contain)
        .filter_method(FilterMethod::Linear);

    let selection_badge: Element<'_, Message> = if selected {
        selection_check().into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    let thumb_overlay: Element<'_, Message> = container(
        row![
            selection_badge,
            Space::new().width(Length::Fill),
            kind_badge
        ]
        .align_y(iced::Alignment::Start),
    )
    .width(Length::Fill)
    .height(Length::Fixed(height))
    .padding([SPACE_XS as u16, SPACE_XS as u16])
    .into();

    // Top scrim keeps the badges legible over bright images.
    let scrim: Element<'_, Message> = container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .style(thumb_scrim_style)
        .into();

    let thumb_with_badge: Element<'_, Message> = stack([thumb, scrim, thumb_overlay])
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .clip(true)
        .into();

    let card_content = column![
        thumb_with_badge,
        container(
            text(item.metadata_line.clone())
                .size(FONT_CAPTION)
                .style(text_secondary)
        )
        .padding([SPACE_XS as u16, SPACE_SM as u16]),
    ];

    interactive_card(
        card_content,
        Some(Message::SelectMedia(item.media_id)),
        selected,
    )
    .width(Length::Fill)
    .into()
}

fn render_empty_state(icon: image::Handle, message: &str) -> Element<'_, Message> {
    empty_state(
        image(icon)
            .width(ICON_XL)
            .height(ICON_XL)
            .content_fit(ContentFit::Contain)
            .filter_method(FilterMethod::Linear),
        message,
        None,
        None,
    )
    .into()
}
