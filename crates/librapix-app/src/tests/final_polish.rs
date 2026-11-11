use super::*;
use iced::{Point, Rectangle, Vector};
use iced_futures::core::{
    Layout, Shell, clipboard, layout, mouse,
    renderer::Headless,
    widget::{Operation, Tree},
};

#[derive(Default)]
struct Labels(Vec<(String, Rectangle)>, Vec<Rectangle>);
impl Operation for Labels {
    fn container(&mut self, _: Option<&Id>, bounds: Rectangle) {
        self.1.push(bounds);
    }
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self);
    }
    fn text(&mut self, _: Option<&Id>, bounds: Rectangle, value: &str) {
        self.0.push((value.into(), bounds));
    }
}
struct Harness<'a> {
    element: Element<'a, Message>,
    tree: Tree,
    node: layout::Node,
    renderer: iced::Renderer,
    size: Size,
}
impl<'a> Harness<'a> {
    fn new(mut element: Element<'a, Message>, size: Size) -> Self {
        let renderer = iced_futures::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(13.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        Self {
            element,
            tree,
            node,
            renderer,
            size,
        }
    }
    fn labels(&mut self) -> Labels {
        let mut labels = Labels::default();
        self.element.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &mut labels,
        );
        labels
    }
    fn event(&mut self, event: mouse::Event, point: Point) -> Vec<Message> {
        let mut messages = Vec::new();
        self.element.as_widget_mut().update(
            &mut self.tree,
            &iced::Event::Mouse(event),
            Layout::new(&self.node),
            mouse::Cursor::Available(point),
            &self.renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(self.size),
        );
        messages
    }
    fn click(&mut self, point: Point) -> Vec<Message> {
        self.event(mouse::Event::ButtonPressed(mouse::Button::Left), point);
        self.event(mouse::Event::ButtonReleased(mouse::Button::Left), point)
    }
    fn tooltip(&mut self, point: Point) {
        self.event(mouse::Event::CursorMoved { position: point }, point);
        let mut overlay = self
            .element
            .as_widget_mut()
            .overlay(
                &mut self.tree,
                Layout::new(&self.node),
                &self.renderer,
                &Rectangle::with_size(self.size),
                Vector::ZERO,
            )
            .expect("localized hover help");
        let node = overlay.as_overlay_mut().layout(&self.renderer, self.size);
        assert!(node.children()[0].size().width > 40.0);
    }
}

#[test]
fn final_details_edge_centers_vertical_label_and_expands() {
    use ui::patterns::details_pane::{DetailsPaneMode, details_pane};
    let app = test_app();
    for height in [300.0, 900.0] {
        let mut harness = Harness::new(
            details_pane(
                DetailsPaneMode::Edge,
                app.i18n.text(TextKey::DetailsLabel),
                app.i18n.text(TextKey::ExpandDetails),
                Message::ToggleDetailsSidebar,
                || panic!("hidden"),
            )
            .into(),
            Size::new(72.0, height),
        );
        let labels = harness.labels();
        let label = labels
            .0
            .iter()
            .find(|(s, _)| s.replace('\n', "") == "Details")
            .unwrap();
        assert!(
            label.1.height > label.1.width * 2.0,
            "Details must read vertically"
        );
        assert!(
            (label.1.center().x - harness.node.size().width / 2.0).abs() < 1.0,
            "label horizontally centered on the rail"
        );
        assert!(
            (label.1.center().y - height / 2.0).abs() < 1.0,
            "label centered on the rail"
        );
        assert!(matches!(
            harness.click(label.1.center()).as_slice(),
            [Message::ToggleDetailsSidebar]
        ));
        harness.tooltip(label.1.center());
    }
}

#[test]
fn final_browse_header_shares_collapse_control_row() {
    let app = test_app();
    let mut harness = Harness::new(views::shell::view(&app), Size::new(1400.0, 900.0));
    let labels = harness.labels();
    let browse = labels
        .0
        .iter()
        .find(|(s, _)| s == app.i18n.text(TextKey::BrowseSectionLabel))
        .unwrap()
        .1;
    let arrow = labels.0.iter().find(|(s, _)| s == "\u{2039}").unwrap().1;
    assert!(
        (browse.center().y - arrow.center().y).abs() < 1.0,
        "collapse shares Browse header, not a separate top band"
    );
    assert!(browse.y < HEADER_HEIGHT + 42.0);
    assert!(
        harness
            .click(arrow.center())
            .iter()
            .any(|m| matches!(m, Message::ToggleLeftSidebar))
    );
    harness.tooltip(arrow.center());
}

#[test]
fn final_about_is_product_focused_and_compact() {
    let app = test_app();
    let mut harness = Harness::new(render_about_dialog(&app), Size::new(800.0, 600.0));
    let labels = harness.labels();
    let frame = labels
        .1
        .iter()
        .find(|b| (b.width - 460.0).abs() < 0.5)
        .unwrap();
    assert!(
        frame.height < 350.0,
        "About must fit its content, not its 440px cap"
    );
    let copy = labels
        .0
        .iter()
        .map(|(s, _)| s.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    for phrase in [
        "cross-platform",
        "open-source",
        "non-destructive",
        "screenshots",
        "recordings",
    ] {
        assert!(copy.to_lowercase().contains(phrase), "missing {phrase}");
    }
    assert!(!copy.contains("weekend") && !copy.contains("I think"));
    assert!(
        harness
            .click(
                labels
                    .0
                    .iter()
                    .find(|(s, _)| s == "Dismiss")
                    .unwrap()
                    .1
                    .center()
            )
            .iter()
            .any(|m| matches!(m, Message::CloseAbout))
    );
    let mut narrow = Harness::new(render_about_dialog(&app), Size::new(360.0, 300.0));
    let labels = narrow.labels();
    let dismiss = labels.0.iter().find(|(s, _)| s == "Dismiss").unwrap().1;
    assert!(dismiss.y >= 0.0 && dismiss.y + dismiss.height <= 300.0);
    assert!(
        narrow
            .click(dismiss.center())
            .iter()
            .any(|m| matches!(m, Message::CloseAbout))
    );
}

#[test]
fn final_managed_chip_remove_has_localized_hover_help() {
    let app = test_app();
    let mut harness = Harness::new(
        ui::components::chip::managed_chip(
            "game",
            "game",
            None,
            None,
            None,
            Some((
                app.i18n.text(TextKey::RemoveTagHelp).to_owned(),
                Message::DetachTagByName("game".into()),
            )),
            true,
        ),
        Size::new(320.0, 100.0),
    );
    let point = harness
        .labels()
        .0
        .iter()
        .find(|(s, _)| s == "x")
        .unwrap()
        .1
        .center();
    harness.tooltip(point);
    assert!(
        matches!(harness.click(point).as_slice(), [Message::DetachTagByName(name)] if name == "game")
    );
    assert!(!app.i18n.text(TextKey::DetailsDetachTagButton).is_empty());
}

#[test]
fn final_settings_rules_are_compact_with_live_actions_at_narrow_widths() {
    let mut app = test_app();
    app.ignore_rules = vec![
        librapix_storage::IgnoreRuleRecord {
            id: 7,
            scope: "global".into(),
            pattern: "**/cache/**".into(),
            is_enabled: true,
        },
        librapix_storage::IgnoreRuleRecord {
            id: 8,
            scope: "global".into(),
            pattern: "**/a very long folder name/screenshots and recordings/cache/**".into(),
            is_enabled: false,
        },
    ];
    for width in [680.0, 320.0] {
        let mut harness = Harness::new(render_settings_dialog(&app), Size::new(width, 900.0));
        let labels = harness.labels();
        let pattern = labels.0.iter().find(|(s, _)| s == "**/cache/**").unwrap().1;
        let enabled = labels.0.iter().find(|(s, _)| s == "enabled").unwrap().1;
        if width > 600.0 {
            assert!(
                (pattern.center().y - enabled.center().y).abs() < 1.0,
                "short rules should not spend a second line on status"
            );
        }
        for (value, bounds) in &labels.0 {
            assert!(
                bounds.x >= 0.0 && bounds.x + bounds.width <= width + 0.5,
                "{value}: {bounds:?} exceeds {width}"
            );
        }
        let disable = labels.0.iter().find(|(s, _)| s == "Disable").unwrap().1;
        assert!(
            harness
                .click(disable.center())
                .iter()
                .any(|m| matches!(m, Message::IgnoreRuleToggleEnabled(7)))
        );
        let edit = labels.0.iter().find(|(s, _)| s == "Edit").unwrap().1;
        assert!(
            harness
                .click(edit.center())
                .iter()
                .any(|m| matches!(m, Message::IgnoreRuleStartEdit(7)))
        );
        let remove = labels.0.iter().find(|(s, _)| s == "x").unwrap().1;
        harness.tooltip(remove.center());
        assert!(
            harness
                .click(remove.center())
                .iter()
                .any(|m| matches!(m, Message::IgnoreRuleRemove(7)))
        );
        let long = labels
            .0
            .iter()
            .find(|(s, _)| s == &app.ignore_rules[1].pattern)
            .unwrap()
            .1;
        let mut full: Element<'_, Message> =
            text(&app.ignore_rules[1].pattern).size(FONT_BODY).into();
        let mut tree = Tree::new(&full);
        let natural = full.as_widget_mut().layout(
            &mut tree,
            &harness.renderer,
            &layout::Limits::new(Size::ZERO, Size::new(long.width + 0.5, 2000.0)),
        );
        assert!(long.height + 0.5 >= natural.size().height);
    }
}

#[test]
fn final_timeline_emphasizes_current_year_without_changing_actions() {
    let mut app = test_app();
    app.timeline_anchors = vec![
        anchor(0, Some(2026), "2026-03-01", 0.0),
        anchor(4, Some(2025), "2025-12-01", 0.5),
    ];
    app.timeline_scrub_anchor_index = Some(0);
    let first = dialog_frame_pixels(views::timeline::scrubber(&app), Theme::Dark);
    app.timeline_scrub_anchor_index = Some(1);
    let second = dialog_frame_pixels(views::timeline::scrubber(&app), Theme::Dark);
    assert!(
        first != second,
        "current year needs visible emphasis even outside a drag"
    );
    for height in [240.0, 900.0] {
        let mut h = Harness::new(
            views::timeline::scrubber(&app),
            Size::new(SCRUBBER_PANEL_WIDTH, height),
        );
        assert!(h.node.size().width <= SCRUBBER_PANEL_WIDTH && h.node.size().height <= height);
        for (year, index) in [("2026", 0), ("2025", 4)] {
            let bounds = h.labels().0.into_iter().find(|(s, _)| s == year).unwrap().1;
            assert!(bounds.y >= 0.0 && bounds.y + bounds.height <= height);
            assert!(
                matches!(h.click(bounds.center()).as_slice(),[Message::JumpToTimelineAnchor(actual)] if *actual==index)
            );
        }
        // Existing slider geometry: its pointer track is at the right of the rail.
        fn slider_bounds(layout: Layout<'_>) -> Option<Rectangle> {
            let b = layout.bounds();
            if (b.width - 12.0).abs() < 0.1 && b.height > 100.0 {
                return Some(b);
            }
            layout.children().find_map(slider_bounds)
        }
        let bounds = slider_bounds(Layout::new(&h.node)).expect("native slider");
        let point = Point::new(bounds.center().x, bounds.y + bounds.height * 0.75);
        let messages = h.event(mouse::Event::ButtonPressed(mouse::Button::Left), point);
        assert!(
            messages.iter().any(
                |m| matches!(m,Message::TimelineScrubChanged(value) if (*value-0.75).abs()<0.05)
            )
        );
        assert!(
            h.event(mouse::Event::ButtonReleased(mouse::Button::Left), point)
                .iter()
                .any(|m| matches!(m, Message::TimelineScrubReleased))
        );
    }
}
