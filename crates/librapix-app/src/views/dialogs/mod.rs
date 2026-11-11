//! Product dialogs. State, validation decisions, tasks and lifecycle remain app-owned.

pub(crate) mod about;
pub(crate) mod filters;
pub(crate) mod library;
pub(crate) mod library_statistics;
pub(crate) mod make_short;
pub(crate) mod new_media;
pub(crate) mod settings;

use crate::{Message, ui::*};
use iced::{
    Element, Length,
    widget::{container, mouse_area},
};

pub(super) fn modal_overlay(dialog: Element<'_, Message>) -> Element<'_, Message> {
    let dialog_surface = mouse_area(dialog).on_press(Message::ModalContentClicked);
    let centered_dialog = container(dialog_surface)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

    mouse_area(
        container(centered_dialog)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([SPACE_2XL as u16, SPACE_XL as u16])
            .style(modal_backdrop_style),
    )
    .on_press(Message::CloseAllDialogs)
    .into()
}
