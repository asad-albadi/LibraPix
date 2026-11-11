//! Timeline and scrubber widgets; anchor/window calculations remain app-owned.
use super::library::media_card;
use crate::ui::styles::button::timeline_marker_style;
use crate::ui::{
    styles::{
        container::{scrubber_chip_style, scrubber_panel_style},
        text::*,
    },
    tokens::*,
};
use crate::{
    BrowseItem, Librapix, Message, SCRUBBER_CHIP_TRACK_WIDTH, SCRUBBER_PANEL_WIDTH,
    SurfaceRenderMetrics, TIMELINE_GROUP_HEADER_HEIGHT, TimelineRenderWindowMetrics,
    TimelineYearMarker, assets, compute_visible_row_window, log_large_surface_render_once,
    log_timeline_window_once, media_virtual_overscan_px, timeline_layout_for_width,
    timeline_year_markers,
};
use iced::widget::image::FilterMethod;
use iced::{
    ContentFit, Element, Length, Size,
    widget::{Space, button, column, container, image, responsive, row, text, vertical_slider},
};
use std::time::Instant;

pub(super) fn view<'a>(
    app: &'a Librapix,
    items: &'a [BrowseItem],
    selected_id: Option<i64>,
    viewport_absolute_y: f32,
    viewport_height: f32,
) -> Element<'a, Message> {
    responsive(move |size: Size| {
        let render_started_at = Instant::now();
        let layout = timeline_layout_for_width(app, items, size.width);
        let overscan_px = media_virtual_overscan_px(app.viewport_drag.active);
        let viewport_height = if viewport_height > 0.0 {
            viewport_height
        } else {
            size.height.clamp(600.0, 900.0)
        };
        let window_top = (viewport_absolute_y - overscan_px).max(0.0);
        let window_bottom = viewport_absolute_y + viewport_height + overscan_px;
        let mut sections = column![];
        let mut pending_spacer = 0.0f32;
        let mut cursor_y = 0.0f32;
        let mut visible_groups = 0usize;
        let mut visible_rows = 0usize;
        let mut global_row_index = 0usize;
        let mut first_visible_row = None;
        let mut last_visible_row = None;
        let mut top_spacer_height = 0.0f32;
        let mut bottom_spacer_height = 0.0f32;

        for (section_index, section_layout) in layout.sections.iter().enumerate() {
            let header_item = &items[section_layout.header_index];
            let row_layouts = &section_layout.row_layouts;
            let section_gap = if section_index + 1 < layout.sections.len() {
                SPACE_MD as f32
            } else {
                0.0
            };
            let section_height = section_layout.section_height;
            let section_top = cursor_y;
            let section_bottom = section_top + section_height;
            let intersects_window = section_bottom >= window_top && section_top <= window_bottom;

            if !intersects_window {
                pending_spacer += section_height + section_gap;
                if visible_rows == 0 {
                    top_spacer_height += section_height + section_gap;
                } else {
                    bottom_spacer_height += section_height + section_gap;
                }
                global_row_index += row_layouts.len();
                cursor_y += section_height + section_gap;
                continue;
            }

            if pending_spacer > 0.0 {
                sections = sections.push(Space::new().height(Length::Fixed(pending_spacer)));
                pending_spacer = 0.0;
            }

            visible_groups = visible_groups.saturating_add(1);
            let count_chips: Element<'_, Message> = if let (Some(img), Some(vid)) =
                (header_item.group_image_count, header_item.group_video_count)
            {
                row![
                    image(assets::icon_type_image(app.is_dark_theme()))
                        .width(Length::Fixed(ICON_XS))
                        .height(Length::Fixed(ICON_XS))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(format!("{img}"))
                        .size(FONT_CAPTION)
                        .style(text_secondary),
                    image(assets::icon_type_video(app.is_dark_theme()))
                        .width(Length::Fixed(ICON_XS))
                        .height(Length::Fixed(ICON_XS))
                        .content_fit(ContentFit::Contain)
                        .filter_method(FilterMethod::Linear),
                    text(format!("{vid}"))
                        .size(FONT_CAPTION)
                        .style(text_secondary),
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center)
                .into()
            } else {
                column![].into()
            };
            let group_header = container(
                row![
                    text(header_item.title.clone())
                        .size(FONT_SUBTITLE)
                        .style(text_primary),
                    Space::new().width(Length::Fill),
                    count_chips,
                ]
                .spacing(SPACE_SM)
                .align_y(iced::Alignment::Center),
            )
            .padding([SPACE_SM as u16, 0]);
            let body_spacing = if row_layouts.is_empty() {
                0.0
            } else {
                SPACE_XS as f32
            };
            let rows_top = section_top + TIMELINE_GROUP_HEADER_HEIGHT + body_spacing;
            let row_window = compute_visible_row_window(
                row_layouts,
                rows_top,
                window_top,
                window_bottom,
                GALLERY_GAP as f32,
            );
            let header_bottom = section_top + TIMELINE_GROUP_HEADER_HEIGHT;
            let header_visible = header_bottom >= window_top && section_top <= window_bottom;
            let mut section_pending_spacer = 0.0f32;

            if header_visible {
                sections = sections.push(group_header);
            } else {
                section_pending_spacer += TIMELINE_GROUP_HEADER_HEIGHT;
            }

            section_pending_spacer += body_spacing + row_window.top_spacer;

            if row_window.visible_rows > 0 {
                if first_visible_row.is_none() {
                    first_visible_row = Some(global_row_index + row_window.start_row);
                }
                last_visible_row = Some(global_row_index + row_window.end_row - 1);
            }

            if section_pending_spacer > 0.0 {
                sections =
                    sections.push(Space::new().height(Length::Fixed(section_pending_spacer)));
            }

            for (index, layout) in row_layouts
                .iter()
                .enumerate()
                .skip(row_window.start_row)
                .take(row_window.visible_rows)
            {
                let mut row_widget = row![].spacing(GALLERY_GAP);
                for item in &items[section_layout.media_start + layout.start
                    ..section_layout.media_start + layout.end]
                {
                    let portion = (item.aspect_ratio * 1000.0).max(1.0) as u16;
                    let card = media_card(
                        item,
                        selected_id == Some(item.media_id),
                        layout.height,
                        app.is_dark_theme(),
                    );
                    row_widget =
                        row_widget.push(container(card).width(Length::FillPortion(portion)));
                }
                visible_rows = visible_rows.saturating_add(1);
                sections = sections.push(row_widget);
                if index + 1 < row_window.end_row {
                    sections =
                        sections.push(Space::new().height(Length::Fixed(GALLERY_GAP as f32)));
                }
            }

            let trailing_spacer = row_window.bottom_spacer + section_gap;
            if trailing_spacer > 0.0 {
                sections = sections.push(Space::new().height(Length::Fixed(trailing_spacer)));
            }
            if visible_rows == 0 {
                top_spacer_height += section_pending_spacer + trailing_spacer;
            } else {
                bottom_spacer_height += trailing_spacer;
            }
            global_row_index += row_layouts.len();
            cursor_y += section_height + section_gap;
        }

        if pending_spacer > 0.0 {
            sections = sections.push(Space::new().height(Length::Fixed(pending_spacer)));
        }

        log_large_surface_render_once(SurfaceRenderMetrics {
            surface: "timeline",
            total_items: layout.total_items,
            total_rows: layout.total_rows,
            visible_rows,
            viewport_absolute_y,
            viewport_height,
            overscan_px,
            drag_active: app.viewport_drag.active,
            render_elapsed: render_started_at.elapsed(),
        });
        log_timeline_window_once(TimelineRenderWindowMetrics {
            total_items: layout.total_items,
            total_groups: layout.total_groups,
            total_rows: layout.total_rows,
            visible_groups,
            visible_rows,
            first_visible_row,
            last_visible_row,
            top_spacer: top_spacer_height,
            bottom_spacer: bottom_spacer_height,
            viewport_absolute_y,
            viewport_height,
            overscan_px,
            drag_active: app.viewport_drag.active,
        });

        sections.into()
    })
    .into()
}

pub(crate) fn scrubber(app: &Librapix) -> Element<'_, Message> {
    if app.timeline_anchors.is_empty() {
        return container(column![])
            .width(Length::Fixed(SCRUBBER_PANEL_WIDTH))
            .height(Length::Fill)
            .into();
    }

    let slider_value = (1.0 - app.timeline_scrub_value).clamp(0.0, 1.0);
    let slider = vertical_slider(0.0..=1.0, slider_value, |value| {
        Message::TimelineScrubChanged(1.0 - value)
    })
    .on_release(Message::TimelineScrubReleased)
    .step(0.001)
    .width(12.0)
    .height(Length::Fill);

    let active_anchor = app
        .timeline_scrub_anchor_index
        .and_then(|index| app.timeline_anchors.get(index));
    let chip_label = active_anchor
        .map(|anchor| format!("{} ({})", anchor.label, anchor.item_count))
        .unwrap_or_default();
    let chip_position = app.timeline_scrub_value.clamp(0.0, 1.0);

    let chip_track_content: Element<'_, Message> = if app.timeline_scrubbing {
        let top = ((chip_position * 1000.0).round() as u16).min(1000);
        let bottom = 1000u16.saturating_sub(top);
        column![
            Space::new().height(Length::FillPortion(top.max(1))),
            container(text(chip_label).size(FONT_CAPTION).style(text_primary))
                .padding([SPACE_2XS as u16, SPACE_SM as u16])
                .style(scrubber_chip_style),
            Space::new().height(Length::FillPortion(bottom.max(1))),
        ]
        .height(Length::Fill)
        .into()
    } else {
        Space::new().height(Length::Fill).into()
    };
    let chip_track = container(chip_track_content)
        .width(Length::Fixed(SCRUBBER_CHIP_TRACK_WIDTH))
        .height(Length::Fill);

    let year_markers = timeline_year_markers(&app.timeline_anchors);
    let marker_track = year_marker_track(&year_markers);
    let scrub_controls = row![
        marker_track,
        row![chip_track, slider]
            .spacing(SPACE_XS)
            .height(Length::Fill),
    ]
    .spacing(SPACE_SM)
    .height(Length::Fill);

    container(column![scrub_controls].height(Length::Fill))
        .width(Length::Fixed(SCRUBBER_PANEL_WIDTH))
        .height(Length::Fill)
        .padding([SPACE_SM as u16, SPACE_XS as u16])
        .style(scrubber_panel_style)
        .into()
}

fn year_marker_track(markers: &[TimelineYearMarker]) -> Element<'static, Message> {
    if markers.is_empty() {
        return Space::new().width(Length::Fixed(0.0)).into();
    }

    let mut track = column![].height(Length::Fill);
    let mut previous_position = 0.0f32;

    for marker in markers {
        let current = marker.normalized_position.clamp(0.0, 1.0);
        let spacer = ((current - previous_position).max(0.0) * 1000.0).round() as u16;
        if spacer > 0 {
            track = track.push(Space::new().height(Length::FillPortion(spacer)));
        }

        track = track.push(
            button(text(marker.label.clone()).size(FONT_CAPTION))
                .on_press(Message::JumpToTimelineAnchor(marker.group_index))
                .style(timeline_marker_style)
                .padding([SPACE_2XS as u16, SPACE_XS as u16]),
        );
        previous_position = current;
    }

    let trailing = ((1.0 - previous_position).max(0.0) * 1000.0).round() as u16;
    if trailing > 0 {
        track = track.push(Space::new().height(Length::FillPortion(trailing)));
    }

    track.into()
}
