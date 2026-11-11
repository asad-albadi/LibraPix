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

fn leaf_bounds(layout: Layout<'_>, bounds: &mut Vec<Rectangle>) {
    if layout.children().next().is_none() {
        bounds.push(layout.bounds());
    } else {
        for child in layout.children() {
            leaf_bounds(child, bounds);
        }
    }
}

#[test]
fn brand_header_preserves_ratio_and_clearspace_in_both_themes() {
    for theme in [ThemePreference::Light, ThemePreference::Dark] {
        let mut app = test_app();
        app.theme_preference = theme;
        for width in [360.0, 640.0, 900.0, 1400.0] {
            let harness = Harness::new(views::shell::view(&app), Size::new(width, 900.0));
            let mut leaves = Vec::new();
            leaf_bounds(Layout::new(&harness.node), &mut leaves);
            // The product identity is the first leaf in the shell's header.
            let brand = leaves[0];
            assert!(brand.width > 0.0 && brand.height > 0.0);
            assert!((brand.width / brand.height - 1893.0 / 535.0).abs() < 0.001);
            assert!(brand.height <= 36.0);
            if width >= 900.0 {
                assert!((brand.height - 36.0).abs() < 0.01);
            }
            assert!(brand.y >= 8.0, "top clearspace: {brand:?}");
            assert!(
                52.0 - brand.y - brand.height >= 8.0,
                "bottom clearspace: {brand:?}"
            );
            assert!((brand.center().y - 26.0).abs() < 0.01);
            assert!((brand.x - 24.0).abs() < 0.01);
            assert!(brand.x + brand.width + 8.0 <= width);
            let neighbor = leaves
                .iter()
                .skip(1)
                .find(|b| b.width > 0.0 && b.height > 0.0)
                .unwrap();
            assert!(neighbor.x - brand.x - brand.width >= 8.0 - 0.01);
        }
    }
}

#[test]
fn brand_component_scales_to_narrow_limits_without_stretching() {
    use iced::advanced::image::Renderer as _;
    use ui::components::brand::{BrandPlacement, brand};

    for dark in [false, true] {
        for (placement, handle, dimensions, max_height) in [
            (
                BrandPlacement::Header,
                assets::wordmark(dark),
                Size::new(1893, 535),
                36.0,
            ),
            (
                BrandPlacement::AboutWordmark,
                assets::wordmark(dark),
                Size::new(1893, 535),
                32.0,
            ),
            (
                BrandPlacement::AboutMark,
                assets::logo(dark),
                Size::new(554, 833),
                44.0,
            ),
        ] {
            for width in [24.0, 64.0, 128.0, 360.0] {
                let harness = Harness::new(brand(dark, placement), Size::new(width, 100.0));
                assert_eq!(harness.renderer.measure_image(&handle).unwrap(), dimensions);
                let mut leaves = Vec::new();
                leaf_bounds(Layout::new(&harness.node), &mut leaves);
                assert_eq!(leaves.len(), 1);
                let artwork = leaves[0];
                let ratio = dimensions.width as f32 / dimensions.height as f32;
                assert!((artwork.width / artwork.height - ratio).abs() < 0.001);
                assert!(artwork.width > 0.0 && artwork.width <= width);
                assert!(artwork.height <= max_height + 0.01);
                if matches!(placement, BrandPlacement::Header) {
                    assert!(artwork.y >= 8.0);
                    assert!(52.0 - artwork.y - artwork.height >= 8.0 - 0.01);
                    assert!((artwork.center().y - 26.0).abs() < 0.01);
                }
            }
        }
    }
}

#[test]
fn brand_about_uses_balanced_aspect_preserving_artwork() {
    for theme in [ThemePreference::Light, ThemePreference::Dark] {
        let mut app = test_app();
        app.theme_preference = theme;
        for width in [360.0, 800.0] {
            let harness = Harness::new(render_about_dialog(&app), Size::new(width, 600.0));
            let mut leaves = Vec::new();
            leaf_bounds(Layout::new(&harness.node), &mut leaves);
            let wordmark = leaves
                .iter()
                .find(|b| (b.height - 32.0).abs() < 0.01 && (b.width - 113.22617).abs() < 0.01)
                .expect("32px proportional About wordmark");
            let mark = leaves
                .iter()
                .find(|b| (b.height - 44.0).abs() < 0.01 && (b.width - 29.262905).abs() < 0.01)
                .expect("44px proportional About mark");
            assert!(wordmark.x >= mark.x + mark.width + 8.0 - 0.01);
            assert!(wordmark.x + wordmark.width <= width);
        }
    }
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

pub(super) fn dialog_padding_point(element: Element<'_, Message>) -> Point {
    let mut harness = Harness::new(element, Size::new(1400.0, 900.0));
    let labels = harness.labels();
    let frame = labels
        .1
        .iter()
        .find(|bounds| bounds.width < 1300.0)
        .expect("dialog frame");
    Point::new(frame.x + 2.0, frame.y + 2.0)
}

#[test]
fn final_details_edge_centers_rotated_word_and_expands() {
    use ui::patterns::details_pane::{DetailsPaneMode, details_pane};
    let app = test_app();
    for height in [80.0, 300.0, 900.0] {
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
            .find(|(s, _)| s == "Details")
            .expect("one localized word, not stacked letters");
        let natural = Harness::new(
            text("Details")
                .size(FONT_CAPTION)
                .shaping(iced::widget::text::Shaping::Advanced)
                .into(),
            Size::new(200.0, 200.0),
        )
        .node
        .size();
        assert!((label.1.width - natural.height).abs() < 1.0);
        assert!((label.1.height - natural.width).abs() < 1.0);
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
        for y in [16.0, height - 16.0] {
            assert!(matches!(
                harness
                    .click(Point::new(harness.node.size().width / 2.0, y))
                    .as_slice(),
                [Message::ToggleDetailsSidebar]
            ));
        }
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
        let enable = labels
            .0
            .iter()
            .rev()
            .find(|(s, _)| s == app.i18n.text(TextKey::IgnoreRuleAddButton))
            .unwrap()
            .1;
        assert!(
            harness
                .click(enable.center())
                .iter()
                .any(|m| matches!(m, Message::IgnoreRuleToggleEnabled(8)))
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
fn final_settings_compact_rows_keep_distinct_deterministic_chip_tones() {
    let keys = ["landscape", "portrait", "gameplay"];
    for (theme, preference, dark) in [
        (Theme::Dark, ThemePreference::Dark, true),
        (Theme::Light, ThemePreference::Light, false),
    ] {
        let mut app = test_app();
        app.theme_preference = preference;
        app.ignore_rules = keys
            .iter()
            .enumerate()
            .map(|(id, key)| librapix_storage::IgnoreRuleRecord {
                id: id as i64,
                scope: "global".into(),
                pattern: (*key).into(),
                is_enabled: true,
            })
            .collect();
        let tones = keys.map(|key| chip_tone_for_key(key, dark));
        assert!(tones.iter().enumerate().all(|(i, tone)| {
            tones[..i]
                .iter()
                .all(|other| other.background != tone.background)
        }));
        for enabled in [true, false] {
            for rule in &mut app.ignore_rules {
                rule.is_enabled = enabled;
            }
            let pixels = dialog_frame_pixels(render_settings_dialog(&app), theme.clone());
            assert_eq!(
                pixels,
                dialog_frame_pixels(render_settings_dialog(&app), theme.clone()),
                "tones must not vary between frames"
            );
            for (key, tone) in keys.iter().zip(&tones) {
                let expected = tone.background.into_rgba8();
                let count = pixels
                    .chunks_exact(4)
                    .filter(|pixel| pixel.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 1))
                    .count();
                assert!(
                    count > 500,
                    "{theme:?} {key} enabled={enabled}: compact rule row lost its keyed fill ({count} pixels)"
                );
                for foreground in [tone.text, tone.accent_text] {
                    for background in [tone.background, tone.background_hover] {
                        let a = foreground.relative_luminance();
                        let b = background.relative_luminance();
                        assert!(
                            (a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5,
                            "{theme:?} {key}: unreadable chip text"
                        );
                    }
                }
            }
        }
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

fn new_media_spacing_fixture(long_path: bool) -> Librapix {
    let mut app = test_app();
    app.new_media_announcement = Some(NewMediaAnnouncement {
        media_id: 7,
        title: "capture.png".into(),
        metadata_line: "Image".into(),
        preview_path: None,
        media_kind: "image".into(),
        file_size_bytes: 4096,
        modified_unix_seconds: None,
        width_px: Some(3440),
        height_px: Some(1440),
        absolute_path: if long_path {
            PathBuf::from(format!(
                "/media/{}/capture.png",
                "nested directory/".repeat(100)
            ))
        } else {
            PathBuf::from("/media/capture.png")
        },
        additional_count: 0,
    });
    app
}

fn assert_new_media_actions(h: &mut Harness<'_>, app: &Librapix) {
    let labels = h.labels();
    for (key, expected) in [
        (TextKey::MediaSelectButton, "select"),
        (TextKey::DetailsOpenFileButton, "open"),
        (TextKey::DetailsCopyFileButton, "copy"),
        (TextKey::DismissButton, "dismiss"),
    ] {
        let bounds = labels
            .0
            .iter()
            .find(|(s, _)| s == app.i18n.text(key))
            .unwrap()
            .1;
        assert!(
            bounds.height > 0.0 && bounds.y >= 0.0 && bounds.y + bounds.height <= h.size.height
        );
        let messages = h.click(bounds.center());
        assert!(
            matches!(
                (expected, messages.as_slice()),
                ("select", [Message::SelectMedia(7)])
                    | ("open", [Message::OpenMediaById(7)])
                    | ("copy", [Message::CopyMediaFileById(7)])
                    | ("dismiss", [Message::DismissNewMediaAnnouncement])
            ),
            "{expected}: {messages:?}"
        );
    }
}

#[test]
fn dialog_spacing_new_media_short_content_has_no_blank_body_and_live_footer() {
    let app = new_media_spacing_fixture(false);
    for size in [Size::new(900.0, 900.0), Size::new(480.0, 720.0)] {
        let mut h = Harness::new(render_new_media_dialog(&app), size);
        let labels = h.labels();
        let frame = labels
            .1
            .iter()
            .find(|b| b.width <= 640.0 && b.height < size.height - 64.0)
            .unwrap();
        let path = labels
            .0
            .iter()
            .find(|(s, _)| s.starts_with("Path:"))
            .unwrap()
            .1;
        let footer = labels.0.iter().find(|(s, _)| s == "Dismiss").unwrap().1;
        let gap = footer.y - (path.y + path.height);
        eprintln!(
            "New Media at {size:?}: frame height {}, footer gap {gap}",
            frame.height
        );
        assert!(
            gap < 60.0,
            "short New Media has {gap}px of empty space before its footer"
        );
        assert!(frame.height < 600.0, "content must not fill the 680px cap");
        let inside = h.event(
            mouse::Event::ButtonPressed(mouse::Button::Left),
            Point::new(frame.x + 2.0, frame.y + 2.0),
        );
        assert!(matches!(inside.as_slice(), [Message::ModalContentClicked]));
        assert_new_media_actions(&mut h, &app);
        let messages = h.event(
            mouse::Event::ButtonPressed(mouse::Button::Left),
            Point::new(1.0, 1.0),
        );
        assert!(matches!(messages.as_slice(), [Message::CloseAllDialogs]));
    }
}

#[derive(Default)]
struct DialogScrolls(Vec<(Rectangle, Rectangle, Vector)>);
impl Operation for DialogScrolls {
    fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn Operation)) {
        f(self);
    }
    fn scrollable(
        &mut self,
        _: Option<&Id>,
        bounds: Rectangle,
        content: Rectangle,
        offset: Vector,
        _: &mut dyn iced_futures::core::widget::operation::scrollable::Scrollable,
    ) {
        self.0.push((bounds, content, offset));
    }
}

#[test]
fn dialog_spacing_new_media_overflow_scrolls_with_stationary_reachable_footer() {
    let app = new_media_spacing_fixture(true);
    for size in [Size::new(800.0, 720.0), Size::new(480.0, 400.0)] {
        let mut h = Harness::new(render_new_media_dialog(&app), size);
        let mut before = DialogScrolls::default();
        h.element.as_widget_mut().operate(
            &mut h.tree,
            Layout::new(&h.node),
            &h.renderer,
            &mut before,
        );
        assert_eq!(before.0.len(), 1);
        let (viewport, content, _) = before.0[0];
        assert!(content.height > viewport.height && viewport.height > 0.0);
        let footer_before = h
            .labels()
            .0
            .into_iter()
            .find(|(s, _)| s == "Dismiss")
            .unwrap()
            .1;
        assert!(viewport.y + viewport.height <= footer_before.y);
        assert_new_media_actions(&mut h, &app);
        h.event(
            mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1000.0 },
            },
            viewport.center(),
        );
        // Reconcile/layout the same native tree, as the live app does between frames.
        h.tree.diff(&h.element);
        h.node = h.element.as_widget_mut().layout(
            &mut h.tree,
            &h.renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        let mut after = DialogScrolls::default();
        h.element.as_widget_mut().operate(
            &mut h.tree,
            Layout::new(&h.node),
            &h.renderer,
            &mut after,
        );
        assert!(
            after.0[0].2.y > 0.0,
            "wheel input must reach the native scrollable"
        );
        let labels = h.labels();
        let footer_after = labels.0.iter().find(|(s, _)| s == "Dismiss").unwrap().1;
        assert_eq!(footer_before, footer_after);
        let path = labels
            .0
            .iter()
            .find(|(s, _)| s.starts_with("Path:"))
            .unwrap()
            .1;
        let visible_bottom = path.y + path.height - after.0[0].2.y;
        assert!(
            visible_bottom <= viewport.y + viewport.height + 1.0 && visible_bottom >= viewport.y
        );
        assert_new_media_actions(&mut h, &app);
    }
}
