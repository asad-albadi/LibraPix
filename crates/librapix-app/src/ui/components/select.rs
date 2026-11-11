use crate::ui::{
    styles::select::{select_menu_style, select_style},
    tokens::*,
};
use iced::widget::{PickList, pick_list};

pub fn app_select<'a, T, Message: Clone + 'a>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
) -> PickList<'a, T, &'a [T], T, Message>
where
    T: ToString + PartialEq + Clone + 'a,
{
    pick_list(options, selected, on_select)
        .text_size(FONT_BODY)
        .padding(SPACE_SM as u16)
        .style(select_style)
        .menu_style(select_menu_style)
}
