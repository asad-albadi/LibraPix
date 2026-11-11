//! Viewport-bounded gallery composition; layout and cache ownership stay in the app.
use super::library::media_card;
use crate::ui::tokens::GALLERY_GAP;
use crate::{
    BrowseItem, Librapix, Message, SurfaceRenderMetrics, gallery_layout_for_width,
    log_large_surface_render_once, media_virtual_overscan_px,
};
use iced::{
    Element, Length, Size,
    widget::{Space, column, container, responsive, row},
};
use std::time::Instant;

pub(super) fn view<'a>(
    app: &'a Librapix,
    items: &'a [BrowseItem],
    selected_id: Option<i64>,
    viewport_absolute_y: f32,
    viewport_height: f32,
) -> Element<'a, Message> {
    let total_items = items.len();
    if total_items == 0 {
        return column![].into();
    }

    responsive(move |size: Size| {
        let render_started_at = Instant::now();
        let layout = gallery_layout_for_width(app, items, size.width);
        let overscan_px = media_virtual_overscan_px(app.viewport_drag.active);
        let viewport_height = if viewport_height > 0.0 {
            viewport_height
        } else {
            size.height.clamp(600.0, 900.0)
        };
        let window_top = (viewport_absolute_y - overscan_px).max(0.0);
        let window_bottom = viewport_absolute_y + viewport_height + overscan_px;
        let mut grid = column![];
        let mut pending_spacer = 0.0f32;
        let mut cursor_y = 0.0f32;
        let mut visible_rows = 0usize;

        for (index, row_layout) in layout.rows.iter().enumerate() {
            let row_top = cursor_y;
            let row_bottom = row_top + row_layout.height;
            let gap_after = if index + 1 < layout.rows.len() {
                GALLERY_GAP as f32
            } else {
                0.0
            };
            let segment_height = row_layout.height + gap_after;
            let intersects_window = row_bottom >= window_top && row_top <= window_bottom;

            if !intersects_window {
                pending_spacer += segment_height;
                cursor_y += segment_height;
                continue;
            }

            if pending_spacer > 0.0 {
                grid = grid.push(Space::new().height(Length::Fixed(pending_spacer)));
                pending_spacer = 0.0;
            }

            let mut row_widget = row![].spacing(GALLERY_GAP);
            for item in &items[row_layout.start..row_layout.end] {
                let portion = (item.aspect_ratio * 1000.0).max(1.0) as u16;
                let card = media_card(
                    item,
                    selected_id == Some(item.media_id),
                    row_layout.height,
                    app.is_dark_theme(),
                );
                row_widget = row_widget.push(container(card).width(Length::FillPortion(portion)));
            }
            visible_rows = visible_rows.saturating_add(1);
            grid = grid.push(row_widget);
            if gap_after > 0.0 {
                grid = grid.push(Space::new().height(Length::Fixed(gap_after)));
            }
            cursor_y += segment_height;
        }

        if pending_spacer > 0.0 {
            grid = grid.push(Space::new().height(Length::Fixed(pending_spacer)));
        }

        log_large_surface_render_once(SurfaceRenderMetrics {
            surface: "gallery",
            total_items,
            total_rows: layout.rows.len(),
            visible_rows,
            viewport_absolute_y,
            viewport_height,
            overscan_px,
            drag_active: app.viewport_drag.active,
            render_elapsed: render_started_at.elapsed(),
        });

        grid.into()
    })
    .into()
}
