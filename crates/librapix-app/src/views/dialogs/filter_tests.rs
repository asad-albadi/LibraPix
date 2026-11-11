use super::*;
use iced::{Point, Rectangle, Size};
use iced_futures::core::{
    Layout, Shell, clipboard, layout, mouse,
    renderer::Headless,
    widget::{Operation, Tree},
};

#[derive(Default)]
struct Labels(Vec<(String, Rectangle)>);
impl Operation for Labels {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self);
    }
    fn text(&mut self, _: Option<&iced::widget::Id>, bounds: Rectangle, value: &str) {
        self.0.push((value.into(), bounds));
    }
}

#[test]
fn adjacent_long_filter_labels_fit_their_hit_areas() {
    let renderer = iced_futures::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(13.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    for names in [
        [
            "Screenshots and recordings from an exceptionally long named library",
            "Another adjacent tag with a complete descriptive multi line name",
        ],
        ["Screenshots and recordings", "My secondary media library"],
    ] {
        for width in [180.0, 280.0] {
            for active in [false, true] {
                let mut element = render_wrapped_filter_chips(
                    names
                        .iter()
                        .map(|name| FilterChipSpec {
                            label: (*name).into(),
                            on_press: Message::SetFilterTag(Some((*name).into())),
                            active,
                        })
                        .collect(),
                );
                let size = Size::new(width, 800.0);
                let mut tree = Tree::new(&element);
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, size),
                );
                let mut labels = Labels::default();
                element.as_widget_mut().operate(
                    &mut tree,
                    Layout::new(&node),
                    &renderer,
                    &mut labels,
                );
                let mut hit_ranges = Vec::new();
                for name in names {
                    let bounds = labels.0.iter().find(|(label, _)| label == name).unwrap().1;
                    let mut full: Element<'_, Message> = text(name).size(FONT_BODY).into();
                    let mut full_tree = Tree::new(&full);
                    let full_node = full.as_widget_mut().layout(
                        &mut full_tree,
                        &renderer,
                        &layout::Limits::new(Size::ZERO, Size::new(bounds.width + 0.5, 800.0)),
                    );
                    if name.len() > 50 {
                        assert!(full_node.size().height > CONTROL_HEIGHT_SM);
                    }
                    assert!(
                        bounds.height + 0.5 >= full_node.size().height,
                        "complete label needs {} but has {}",
                        full_node.size().height,
                        bounds.height
                    );
                    let pointer_at = |y| {
                        element.as_widget().mouse_interaction(
                            &tree,
                            Layout::new(&node),
                            mouse::Cursor::Available(Point::new(bounds.center().x, y)),
                            &Rectangle::with_size(size),
                            &renderer,
                        ) == mouse::Interaction::Pointer
                    };
                    let mut top = bounds.center().y;
                    let mut bottom = top;
                    while top > 0.5 && pointer_at(top - 0.5) {
                        top -= 0.5;
                    }
                    while bottom < size.height && pointer_at(bottom + 0.5) {
                        bottom += 0.5;
                    }
                    assert!(top <= bounds.y + 0.5 && bottom >= bounds.y + bounds.height - 0.5);
                    hit_ranges.push((top, bottom));
                    for y in [bounds.y + 1.0, bounds.y + bounds.height - 1.0] {
                        let mut messages = Vec::new();
                        for event in [
                            mouse::Event::ButtonPressed(mouse::Button::Left),
                            mouse::Event::ButtonReleased(mouse::Button::Left),
                        ] {
                            element.as_widget_mut().update(
                                &mut tree,
                                &iced::Event::Mouse(event),
                                Layout::new(&node),
                                mouse::Cursor::Available(Point::new(bounds.center().x, y)),
                                &renderer,
                                &mut clipboard::Null,
                                &mut Shell::new(&mut messages),
                                &Rectangle::with_size(size),
                            );
                        }
                        assert_eq!(messages.len(), 1);
                        assert!(
                            matches!(&messages[0], Message::SetFilterTag(Some(value)) if value == name)
                        );
                    }
                }
                assert!(hit_ranges[0].1 < hit_ranges[1].0);
            }
        }
    }
}
