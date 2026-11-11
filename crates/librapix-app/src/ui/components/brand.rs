//! Theme-aware brand placement; artwork geometry stays asset-owned.
use crate::assets;
use crate::ui::tokens::{HEADER_HEIGHT, SPACE_SM};
use iced::widget::{container, image};
use iced::{ContentFit, Element, Length};

#[derive(Clone, Copy)]
pub enum BrandPlacement {
    Header,
    AboutWordmark,
    AboutMark,
}

pub fn brand<'a, Message: 'a>(dark: bool, placement: BrandPlacement) -> Element<'a, Message> {
    // Ratios of the trimmed integration PNGs, not the generated master canvases.
    let (handle, height, ratio) = match placement {
        BrandPlacement::Header => (
            assets::wordmark(dark),
            HEADER_HEIGHT - 2.0 * SPACE_SM as f32,
            1893.0 / 535.0,
        ),
        BrandPlacement::AboutWordmark => (assets::wordmark(dark), 32.0, 1893.0 / 535.0),
        BrandPlacement::AboutMark => (assets::logo(dark), 44.0, 554.0 / 833.0),
    };
    let artwork = image(handle)
        .width(height * ratio)
        .height(Length::Shrink)
        .content_fit(ContentFit::Contain)
        .filter_method(image::FilterMethod::Linear);

    if matches!(placement, BrandPlacement::Header) {
        container(artwork)
            .padding([SPACE_SM as u16, 0])
            .center_y(HEADER_HEIGHT)
            .into()
    } else {
        artwork.into()
    }
}
