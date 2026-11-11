use super::{
    dialog::{DialogParts, DialogSpec, dialog},
    field::{FieldParts, FieldSpec, field},
    status::{StatusTone, status_row},
};
use iced::{
    Element, Size,
    widget::{Space, column, text},
};

#[test]
fn dialog_keeps_owned_dynamic_title_after_source_is_dropped() {
    let element = {
        let title = format!("Library statistics: {}", "Screenshots");
        dialog(DialogSpec {
            title: title.into(),
            close_message: None,
            body: text("Summary").into(),
            footer: None,
            max_width: 560.0,
            max_height: 640.0,
        })
    };
    assert_eq!(
        inspect(element, Size::new(800.0, 700.0)).0,
        ["Library statistics: Screenshots", "Summary"]
    );
}

#[test]
fn dialog_parts_keep_close_and_footer_independent() {
    for close in [false, true] {
        for footer in [false, true] {
            let spec = DialogSpec {
                title: "About".into(),
                close_message: close.then_some(("Dismiss", 7_u8)),
                body: text("Body").into(),
                footer: footer.then(|| text("Actions").into()),
                max_width: 460.0,
                max_height: 440.0,
            };
            let parts = DialogParts::from_spec(&spec);
            assert_eq!(parts.has_close, close);
            assert_eq!(parts.has_footer, footer);
            let (texts, _) = inspect(dialog(spec), Size::new(600.0, 500.0));
            let mut expected = vec!["About"];
            if close {
                expected.push("Dismiss");
            }
            expected.push("Body");
            if footer {
                expected.push("Actions");
            }
            assert_eq!(texts, expected);
        }
    }
}

#[test]
fn field_parts_show_error_instead_of_help() {
    for (help, error, expected, invalid) in [
        (None, None, None, false),
        (Some("Hint"), None, Some("Hint"), false),
        (None, Some("Invalid"), Some("Invalid"), true),
        (Some("Hint"), Some("Invalid"), Some("Invalid"), true),
    ] {
        let spec = FieldSpec::<u8> {
            label: "Speed",
            control: text("Input").into(),
            help,
            error,
        };
        let parts = FieldParts::from_spec(&spec);
        assert_eq!(parts.supporting_text, expected);
        assert_eq!(parts.invalid, invalid);
        let (texts, _) = inspect(field(spec), Size::new(400.0, 200.0));
        let mut expected_texts = vec!["Speed", "Input"];
        expected_texts.extend(expected);
        assert_eq!(texts, expected_texts);
    }
}

#[derive(Default)]
struct Texts(Vec<String>);

impl iced_futures::core::widget::Operation for Texts {
    fn traverse(
        &mut self,
        operate: &mut dyn FnMut(&mut dyn iced_futures::core::widget::Operation),
    ) {
        operate(self);
    }
    fn text(&mut self, _: Option<&iced::widget::Id>, _: iced::Rectangle, value: &str) {
        self.0.push(value.to_owned());
    }
}

fn inspect(
    mut element: Element<'_, u8>,
    size: Size,
) -> (Vec<String>, iced_futures::core::layout::Node) {
    use iced_futures::core::{Layout, layout, renderer::Headless, widget::Tree};
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
    let mut texts = Texts::default();
    element
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut texts);
    (texts.0, node)
}

#[test]
fn dialog_keeps_header_and_footer_visible_when_body_overflows() {
    for size in [Size::new(1000.0, 800.0), Size::new(320.0, 260.0)] {
        let (_, node) = inspect(
            dialog(DialogSpec {
                title: "Title".into(),
                close_message: Some(("Dismiss", 7)),
                body: column![text("Body"), Space::new().height(2000)].into(),
                footer: Some(text("Actions").into()),
                max_width: 460.0,
                max_height: 440.0,
            }),
            size,
        );
        assert!(node.size().width <= size.width.min(460.0));
        assert!(node.size().height <= size.height.min(440.0));
        let children = node.children()[0].children();
        let header = children.first().unwrap().bounds();
        let footer = children.last().unwrap().bounds();
        assert!(header.height > 0.0);
        assert!(footer.height > 0.0);
        assert!(footer.y + footer.height <= node.size().height);
        // Only the body scrolls; its content is taller than its viewport.
        let body = &children[2];
        assert!(body.children()[0].size().height > body.size().height);
        assert!(body.bounds().y + body.size().height <= footer.y);
    }
}

#[test]
fn dialog_bounds_stay_stable_for_short_and_overflowing_content() {
    let mut sizes = Vec::new();
    for body_height in [20, 2000] {
        let (_, node) = inspect(
            dialog(DialogSpec {
                title: "Title".into(),
                close_message: None,
                body: Space::new().height(body_height).into(),
                footer: None,
                max_width: 460.0,
                max_height: 440.0,
            }),
            Size::new(1000.0, 800.0),
        );
        sizes.push(node.size());
    }
    assert_eq!(sizes[0], sizes[1]);
}

#[test]
fn working_status_has_one_stage_line_and_no_numeric_progress() {
    let mut sizes = Vec::new();
    for phase in 0..4 {
        let (texts, node) = inspect(
            status_row("Generating short", StatusTone::Working, phase),
            Size::new(400.0, 100.0),
        );
        assert_eq!(texts, ["Generating short"]);
        sizes.push(node.size());
        // The working row has a three-dot indicator and one stage label, not a bar.
        let row = &node.children()[0];
        assert_eq!(row.children().len(), 2);
        assert_eq!(row.children()[0].children().len(), 3);
    }
    assert!(sizes.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn all_status_tones_keep_caller_localized_text() {
    for tone in [
        StatusTone::Neutral,
        StatusTone::Working,
        StatusTone::Success,
        StatusTone::Warning,
        StatusTone::Error,
    ] {
        let (texts, _) = inspect(
            status_row("Localized status", tone, 0),
            Size::new(400.0, 100.0),
        );
        assert_eq!(texts, ["Localized status"]);
    }
}
