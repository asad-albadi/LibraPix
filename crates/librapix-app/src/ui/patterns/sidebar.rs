use crate::ui::{styles::container::sidebar_style, tokens::*};
use iced::{
    Element, Length,
    widget::{Container, column, container},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarMode {
    Expanded,
    Rail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalRailAction {
    Library,
    AddLibrary,
    Refresh,
    Expand,
}

impl SidebarMode {
    pub fn from_collapsed(collapsed: bool) -> Self {
        if collapsed {
            Self::Rail
        } else {
            Self::Expanded
        }
    }

    pub fn global_actions(self) -> &'static [GlobalRailAction] {
        match self {
            Self::Expanded => &[],
            Self::Rail => &[
                GlobalRailAction::Library,
                GlobalRailAction::AddLibrary,
                GlobalRailAction::Refresh,
                GlobalRailAction::Expand,
            ],
        }
    }
}

/// Expanded content is built only when visible; rail inventory is always fixed.
pub fn sidebar<'a, Message: 'a>(
    mode: SidebarMode,
    expanded_content: impl FnOnce() -> Element<'a, Message>,
    collapse_control: Element<'a, Message>,
    rail_action: impl FnMut(GlobalRailAction) -> Element<'a, Message>,
) -> Container<'a, Message> {
    let (content, width): (Element<'a, Message>, _) = match mode {
        SidebarMode::Expanded => (
            column![
                container(collapse_control)
                    .align_right(Length::Fill)
                    .padding([SPACE_XS as u16, SPACE_SM as u16]),
                expanded_content(),
            ]
            .height(Length::Fill)
            .into(),
            SIDEBAR_WIDTH,
        ),
        SidebarMode::Rail => (
            column(mode.global_actions().iter().copied().map(rail_action))
                .spacing(SPACE_SM)
                .padding([SPACE_SM as u16, SPACE_XS as u16])
                .align_x(iced::Alignment::Center)
                .width(Length::Fill)
                .into(),
            SIDEBAR_RAIL_WIDTH,
        ),
    };
    container(content)
        .width(width)
        .height(Length::Fill)
        .style(sidebar_style)
}
