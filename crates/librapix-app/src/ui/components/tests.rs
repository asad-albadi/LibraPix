use super::{
    badge::BadgeVariant,
    button::{ButtonSize, ButtonVariant, app_button},
    card::CardVariant,
};
use crate::ui::{styles, tokens::palette};
use iced::widget::{button, text_input};
use iced::{Background, Color, Theme};

fn luminance(color: Color) -> f32 {
    let linear = |c: f32| {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

fn contrast(a: Color, b: Color) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn fill(background: Option<Background>, fallback: Color) -> Color {
    match background {
        Some(Background::Color(c)) if c.a > 0.0 => c,
        None | Some(Background::Color(_)) => fallback,
        _ => panic!("component surfaces must use semantic solid colors"),
    }
}

#[test]
fn buttons_remain_legible_across_variants_and_interactions() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
        ] {
            for status in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
                button::Status::Disabled,
            ] {
                let style = styles::button::button_style(&theme, status, variant);
                if status == button::Status::Disabled {
                    assert_eq!(style.text_color, p.text_disabled);
                } else {
                    assert!(
                        contrast(style.text_color, fill(style.background, p.bg_surface)) >= 4.5,
                        "{theme:?} {variant:?} {status:?}"
                    );
                }
            }
        }
        let outline =
            styles::button::button_style(&theme, button::Status::Active, ButtonVariant::Outline);
        assert_eq!(outline.border.width, 1.0);
        assert_eq!(outline.border.color, p.border_strong);
    }
}

#[test]
fn badges_use_legible_semantic_tones_in_both_themes() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        for variant in [
            BadgeVariant::Secondary,
            BadgeVariant::Success,
            BadgeVariant::Destructive,
        ] {
            let style = styles::container::badge_style(&theme, variant);
            assert!(
                contrast(
                    style.text_color.unwrap(),
                    fill(style.background, p.bg_surface)
                ) >= 4.5,
                "{theme:?} {variant:?}"
            );
            assert_eq!(style.border.width, 1.0);
        }
        for (variant, color) in [
            (BadgeVariant::Success, p.success),
            (BadgeVariant::Destructive, p.destructive),
        ] {
            assert_eq!(
                styles::container::badge_style(&theme, variant).border.color,
                color
            );
        }
    }
}

#[test]
fn selected_cards_and_hovered_cards_keep_a_visible_selection_edge() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        for status in [
            button::Status::Active,
            button::Status::Hovered,
            button::Status::Pressed,
        ] {
            let style = styles::button::card_action_style(&theme, status, CardVariant::Selected);
            assert_eq!(style.border.color, p.focus_ring);
            assert_eq!(style.border.width, 2.0);
            assert_eq!(style.background, Some(Background::Color(p.bg_selected)));
        }
        let hovered = styles::button::card_action_style(
            &theme,
            button::Status::Hovered,
            CardVariant::Interactive,
        );
        assert_eq!(hovered.background, Some(Background::Color(p.bg_hover)));
        assert_eq!(hovered.border.color, p.border_strong);
    }
}

#[test]
fn inputs_distinguish_focus_invalid_and_disabled_without_layout_shift() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        for status in [
            text_input::Status::Active,
            text_input::Status::Hovered,
            text_input::Status::Focused { is_hovered: false },
            text_input::Status::Disabled,
        ] {
            let valid = styles::input::input_style(&theme, status, false);
            let invalid = styles::input::input_style(&theme, status, true);
            assert_eq!(valid.border.width, 1.0);
            assert_eq!(invalid.border.width, 1.0);
            if status != text_input::Status::Disabled {
                assert_eq!(invalid.border.color, p.destructive);
            } else {
                assert_eq!(valid.value, p.text_disabled);
                assert_eq!(invalid.border.color, valid.border.color);
            }
        }
        assert_eq!(
            styles::input::input_style(
                &theme,
                text_input::Status::Focused { is_hovered: false },
                false
            )
            .border
            .color,
            p.focus_ring
        );
    }
}

#[test]
fn semantic_text_contrast_and_surface_hierarchy_hold() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        assert!(luminance(p.bg_base) < luminance(p.bg_surface));
        for surface in [
            p.bg_base,
            p.bg_layer,
            p.bg_surface,
            p.bg_card,
            p.bg_hover,
            p.bg_selected,
        ] {
            for text in [p.text_primary, p.text_secondary, p.text_tertiary] {
                assert!(
                    contrast(text, surface) >= 4.5,
                    "{theme:?}: text {text:?} on {surface:?}"
                );
            }
        }
        assert!(contrast(p.focus_ring, p.bg_surface) >= 3.0);
    }
}

#[test]
fn optional_press_really_disables_buttons_and_sizes_stay_consistent() {
    use iced::{Event, Font, Pixels, Point, Rectangle, Size};
    use iced_futures::core::{
        Layout, Shell, clipboard, layout, mouse, renderer::Headless, widget::Tree,
    };
    let renderer = iced_futures::futures::executor::block_on(iced::Renderer::new(
        Font::DEFAULT,
        Pixels(13.0),
        Some("tiny-skia"),
    ))
    .expect("Iced's built-in software renderer");
    for (size, height) in [
        (ButtonSize::Small, 28.0),
        (ButtonSize::Default, 32.0),
        (ButtonSize::Large, 36.0),
        (ButtonSize::Icon, 32.0),
    ] {
        for enabled in [false, true] {
            let mut element: iced::Element<'_, u8> = app_button(
                iced::widget::Space::new().width(10),
                enabled.then_some(7),
                ButtonVariant::Primary,
                size,
            )
            .into();
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(200.0, 100.0)),
            );
            assert_eq!(node.size().height, height);
            if size == ButtonSize::Icon {
                assert_eq!(node.size().width, height);
                let content = &node.children()[0];
                assert_eq!(content.bounds().x + content.children()[0].bounds().x, 11.0);
            }
            let mut messages = Vec::new();
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                element.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(Point::new(4.0, 4.0)),
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &Rectangle::with_size(Size::new(200.0, 100.0)),
                );
            }
            assert_eq!(messages, if enabled { vec![7] } else { vec![] });
        }
    }
}

#[test]
fn tabs_preserve_active_selection_in_both_themes() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        assert_eq!(
            styles::button::tab_style(&theme, button::Status::Active, true)
                .border
                .color,
            p.focus_ring
        );
        assert_ne!(
            styles::button::tab_style(&theme, button::Status::Active, false)
                .border
                .color,
            p.focus_ring
        );
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
    fn text(&mut self, _: Option<&iced::widget::Id>, _: iced::Rectangle, text: &str) {
        self.0.push(text.to_owned());
    }
}

fn rendered_texts(mut element: iced::Element<'_, u8>) -> (Vec<String>, iced::Size) {
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
        &layout::Limits::new(iced::Size::ZERO, iced::Size::new(500.0, 300.0)),
    );
    let mut texts = Texts::default();
    element
        .as_widget_mut()
        .operate(&mut tree, Layout::new(&node), &renderer, &mut texts);
    (texts.0, node.size())
}

#[test]
fn loading_shows_one_stage_and_keeps_the_same_bounds_across_phases() {
    let mut bounds = None;
    for phase in 0..7 {
        let (texts, size) =
            rendered_texts(super::loading::working_row("Probing source video", phase).into());
        assert_eq!(texts, ["Probing source video"]);
        assert_eq!(*bounds.get_or_insert(size), size);
    }
}

#[test]
fn notices_show_tone_summary_and_only_the_requested_dismiss_action() {
    use super::notice::{NoticeTone, ShellNotice, notice};
    {
        let tone = NoticeTone::Error;
        let state = ShellNotice {
            tone,
            summary: "Could not save layout preference.".to_owned(),
        };
        let (texts, _) = rendered_texts(notice(&state, "Error", Some(("Dismiss", 1))).into());
        assert_eq!(
            texts,
            ["Error", "Dismiss", "Could not save layout preference."]
        );
        let (texts, _) = rendered_texts(notice(&state, "Error", None).into());
        assert_eq!(texts, ["Error", "Could not save layout preference."]);
    }
}

#[test]
fn empty_state_preserves_optional_help_and_action() {
    use iced::widget::{Space, text};
    let (texts, _) = rendered_texts(
        super::empty_state::empty_state(
            Space::new(),
            "No media",
            Some("Add a library"),
            Some(text("Add").into()),
        )
        .into(),
    );
    assert_eq!(texts, ["No media", "Add a library", "Add"]);
    let (texts, _) = rendered_texts(
        super::empty_state::empty_state(Space::new(), "No media", None, None).into(),
    );
    assert_eq!(texts, ["No media"]);
}

#[test]
fn icon_buttons_expose_their_tooltip_even_when_disabled() {
    use iced::{Event, Font, Pixels, Point, Rectangle, Size, Vector};
    use iced_futures::core::{
        Layout, Shell, clipboard, layout, mouse, renderer::Headless, widget::Tree,
    };
    let renderer = iced_futures::futures::executor::block_on(iced::Renderer::new(
        Font::DEFAULT,
        Pixels(13.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    for enabled in [false, true] {
        let mut element = super::icon_button::icon_button(
            iced::widget::Space::new().width(16).height(16).into(),
            "Expand Libraries",
            enabled.then_some(1_u8),
            ButtonVariant::Ghost,
        );
        let mut tree = Tree::new(&element);
        let size = Size::new(300.0, 100.0);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        let mut messages = Vec::new();
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(5.0, 5.0),
            }),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(5.0, 5.0)),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(size),
        );
        let mut overlay = element
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &Rectangle::with_size(size),
                Vector::ZERO,
            )
            .expect("icon-only controls always have a tooltip");
        let overlay_node = overlay.as_overlay_mut().layout(&renderer, size);
        // Iced's tooltip overlay does not expose text through Operation. Its
        // descriptive label must still produce more than an empty padded box.
        assert!(overlay_node.children()[0].size().width > 32.0);
        assert!(messages.is_empty());
    }
}

#[test]
fn indeterminate_indicator_changes_pixels_without_changing_size() {
    use iced::{Font, Pixels, Rectangle, Size};
    use iced_futures::core::{
        Layout, Renderer, layout, mouse,
        renderer::{Headless, Style},
        widget::Tree,
    };
    let mut renderer = iced_futures::futures::executor::block_on(iced::Renderer::new(
        Font::DEFAULT,
        Pixels(13.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    for theme in [Theme::Light, Theme::Dark] {
        let mut frames = Vec::new();
        for phase in 0..4 {
            let mut element: iced::Element<'_, u8> =
                super::loading::loading_indicator(phase).into();
            let mut tree = Tree::new(&element);
            let size = Size::new(26.0, 6.0);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, size),
            );
            assert_eq!(node.size(), size);
            renderer.reset(Rectangle::with_size(size));
            element.as_widget().draw(
                &tree,
                &mut renderer,
                &theme,
                &Style::default(),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &Rectangle::with_size(size),
            );
            frames.push(renderer.screenshot(Size::new(26, 6), 1.0, palette(&theme).bg_surface));
        }
        assert_ne!(frames[0], frames[1]);
        assert_ne!(frames[1], frames[2]);
        assert_eq!(frames[0], frames[3]);
    }
}

#[test]
fn shared_accent_text_remains_readable_on_neutral_surfaces() {
    for theme in [Theme::Light, Theme::Dark] {
        let p = palette(&theme);
        for surface in [p.bg_base, p.bg_layer, p.bg_surface] {
            assert!(
                contrast(styles::text::text_accent(&theme).color.unwrap(), surface) >= 4.5,
                "{theme:?}"
            );
        }
    }
}
