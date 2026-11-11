//! Labeled caller-owned controls with one supporting line: error before help.

use crate::ui::{
    styles::text::{text_error, text_primary, text_secondary},
    tokens::*,
};
use iced::{
    Element, Length,
    widget::{column, text},
};

pub struct FieldSpec<'a, Message> {
    pub label: &'a str,
    pub control: Element<'a, Message>,
    pub help: Option<&'a str>,
    pub error: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldParts<'a> {
    pub supporting_text: Option<&'a str>,
    pub invalid: bool,
}

impl<'a> FieldParts<'a> {
    pub fn from_spec<Message>(spec: &FieldSpec<'a, Message>) -> Self {
        Self {
            supporting_text: spec.error.or(spec.help),
            invalid: spec.error.is_some(),
        }
    }
}

pub fn field<'a, Message: 'a>(spec: FieldSpec<'a, Message>) -> Element<'a, Message> {
    let parts = FieldParts::from_spec(&spec);
    let mut content = column![
        text(spec.label).size(FONT_BODY).style(text_primary),
        spec.control
    ]
    .spacing(SPACE_XS)
    .width(Length::Fill);
    if let Some(label) = parts.supporting_text {
        content = content.push(text(label).size(FONT_CAPTION).style(if parts.invalid {
            text_error
        } else {
            text_secondary
        }));
    }
    content.into()
}
