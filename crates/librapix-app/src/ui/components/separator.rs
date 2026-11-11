use crate::ui::styles::container::separator_style;
use iced::widget::{Rule, rule};

pub fn horizontal<'a>() -> Rule<'a> {
    rule::horizontal(1).style(separator_style)
}
pub fn vertical<'a>() -> Rule<'a> {
    rule::vertical(1).style(separator_style)
}
