use iced::widget::{Container, container};
use iced::{Background, Color, Element, Length, Theme, border};

pub const SPACE_XS: u32 = 6;
pub const SPACE_SM: u32 = 10;
pub const SPACE_MD: u32 = 14;

pub const FONT_TITLE: u32 = 28;
pub const FONT_SECTION: u32 = 18;
pub const FONT_BODY: u32 = 14;

pub const SIDEBAR_WIDTH: f32 = 300.0;
pub const DETAILS_WIDTH: f32 = 340.0;

pub fn panel<'a, Message>(content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content)
        .padding(SPACE_MD as u16)
        .width(Length::Fill)
        .style(panel_style)
}

pub fn subtle_panel<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    container(content)
        .padding(SPACE_MD as u16)
        .width(Length::Fill)
        .style(subtle_panel_style)
}

fn panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(30, 31, 36))),
        border: border::rounded(12),
        ..container::Style::default()
    }
}

fn subtle_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(24, 25, 30))),
        border: border::rounded(10),
        ..container::Style::default()
    }
}
