//! One localized word rotated by Iced's native geometry pipeline, not a renderer.

use crate::ui::tokens::FONT_CAPTION;
use iced::{
    Element, Length, Point, Radians, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Layout, Widget,
        graphics::geometry::Renderer as _,
        layout, mouse, renderer,
        text::{self, Paragraph as _, Renderer as _},
        widget::{Operation, Tree},
    },
    widget::canvas,
};

pub(super) fn rotated_label<Message>(label: &str) -> Element<'_, Message> {
    Element::new(RotatedLabel {
        label,
        natural: Size::ZERO,
    })
}

struct RotatedLabel<'a> {
    label: &'a str,
    natural: Size,
}

impl<Message> Widget<Message, Theme, Renderer> for RotatedLabel<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let paragraph = <Renderer as text::Renderer>::Paragraph::with_text(text::Text {
            content: self.label,
            bounds: Size::INFINITE,
            size: FONT_CAPTION.into(),
            line_height: text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            wrapping: text::Wrapping::None,
        });
        self.natural = paragraph.min_bounds();
        layout::Node::new(limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(self.natural.height, self.natural.width),
        ))
    }

    fn operate(
        &mut self,
        _tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), self.label);
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        if self.natural.width <= 0.0 || self.natural.height <= 0.0 {
            return;
        }
        // Use one absolute coordinate space for geometry and clipping.
        let mut frame = canvas::Frame::with_bounds(renderer, clip);
        frame.translate(Vector::new(bounds.center().x, bounds.center().y));
        frame.rotate(Radians(std::f32::consts::FRAC_PI_2));
        // Preserve the whole word even in unusually short panes/localizations.
        frame.scale(
            (bounds.height / self.natural.width)
                .min(bounds.width / self.natural.height)
                .min(1.0),
        );
        frame.fill_text(canvas::Text {
            content: self.label.to_owned(),
            position: Point::ORIGIN,
            color: style.text_color,
            size: FONT_CAPTION.into(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            ..canvas::Text::default()
        });
        renderer.draw_geometry(frame.into_geometry());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::renderer::{Headless, Renderer as _};

    #[test]
    fn rotated_label_pixels_match_a_quarter_turn_of_one_native_word() {
        let mut renderer = iced_futures::futures::executor::block_on(Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(13.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let size = Size::new(128.0, 128.0);
        let viewport = Rectangle::with_size(size);
        let mut label: Element<'_, ()> = iced::widget::container(rotated_label("Details"))
            .center(Length::Fill)
            .into();
        let mut tree = Tree::new(&label);
        let node = label.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        renderer.reset(viewport);
        label.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &renderer::Style {
                text_color: iced::Color::WHITE,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let actual = renderer.screenshot(Size::new(128, 128), 1.0, iced::Color::BLACK);

        // Independent horizontal native reference, quarter-turned in pixel space.
        renderer.reset(viewport);
        let mut frame = canvas::Frame::new(&renderer, size);
        let reference = canvas::Text {
            content: "Details".into(),
            position: Point::new(64.0, 64.0),
            color: iced::Color::WHITE,
            size: FONT_CAPTION.into(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: iced::alignment::Vertical::Center,
            shaping: text::Shaping::Advanced,
            ..canvas::Text::default()
        };
        // Rotation uses native glyph outlines, so compare the same native path
        // representation rather than differently hinted horizontal glyph bitmaps.
        reference.draw_with(|path, color| frame.fill(&path, color));
        renderer.draw_geometry(frame.into_geometry());
        let horizontal = renderer.screenshot(Size::new(128, 128), 1.0, iced::Color::BLACK);
        let mut intersection = 0;
        let mut union = 0;
        for y in 0..128 {
            for x in 0..128 {
                let a = actual[(y * 128 + x) * 4] > 80;
                let b = horizontal[((127 - x) * 128 + y) * 4] > 80;
                intersection += usize::from(a && b);
                union += usize::from(a || b);
            }
        }
        assert!(union > 40, "word must actually render");
        let overlap = intersection as f32 / union as f32;
        assert!(
            overlap > 0.70,
            "expected rotated whole-word glyphs, overlap={overlap}"
        );
    }
}
