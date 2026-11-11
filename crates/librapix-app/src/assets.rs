//! Centralized embedded assets for branding and icons.
//!
//! Assets are compiled into the binary so release executables do not depend
//! on an external `assets/` folder for UI icon rendering. UI icons ship in two
//! tints — white (for dark surfaces) and black (for light surfaces) — and the
//! accessors pick the variant from the active theme via a `dark` flag.
use iced::widget::image;
use std::sync::LazyLock;

/// Project repository URL for the GitHub link in the app header.
pub const REPO_URL: &str = "https://github.com/asad-albadi/LibraPix";

// ── Canonical brand ──

/// INDEX mark, with a light or dark foreground chosen for the active surface.
pub fn logo(dark: bool) -> image::Handle {
    pick(&LOGO_LIGHT, &LOGO_DARK, dark)
}

/// Approved custom wordmark, not a font approximation of the brand lettering.
pub fn wordmark(dark: bool) -> image::Handle {
    pick(&WORDMARK_LIGHT, &WORDMARK_DARK, dark)
}

/// OS window / taskbar icon (brand blue). `None` if the embedded PNG fails to
/// decode, in which case iced falls back to the platform default.
pub fn window_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(
        include_bytes!("../../../assets/logo/index/icon-256.png"),
        None,
    )
    .ok()
}

// ── UI icons (white for dark surfaces, black for light surfaces) ──

fn pick(
    white: &LazyLock<image::Handle>,
    black: &LazyLock<image::Handle>,
    dark: bool,
) -> image::Handle {
    if dark {
        (**white).clone()
    } else {
        (**black).clone()
    }
}

pub fn icon_gallery(dark: bool) -> image::Handle {
    pick(&ICON_GALLERY_W, &ICON_GALLERY_B, dark)
}

pub fn icon_timeline(dark: bool) -> image::Handle {
    pick(&ICON_TIMELINE_W, &ICON_TIMELINE_B, dark)
}

pub fn icon_search(dark: bool) -> image::Handle {
    pick(&ICON_SEARCH_W, &ICON_SEARCH_B, dark)
}

pub fn icon_refresh(dark: bool) -> image::Handle {
    pick(&ICON_REFRESH_W, &ICON_REFRESH_B, dark)
}

pub fn icon_github(dark: bool) -> image::Handle {
    pick(&ICON_GITHUB_W, &ICON_GITHUB_B, dark)
}

pub fn icon_open(dark: bool) -> image::Handle {
    pick(&ICON_OPEN_W, &ICON_OPEN_B, dark)
}

pub fn icon_show_in_folder(dark: bool) -> image::Handle {
    pick(&ICON_SHOW_IN_FOLDER_W, &ICON_SHOW_IN_FOLDER_B, dark)
}

pub fn icon_copy_file(dark: bool) -> image::Handle {
    pick(&ICON_COPY_FILE_W, &ICON_COPY_FILE_B, dark)
}

pub fn icon_copy_path(dark: bool) -> image::Handle {
    pick(&ICON_COPY_PATH_W, &ICON_COPY_PATH_B, dark)
}

pub fn icon_filter(dark: bool) -> image::Handle {
    pick(&ICON_FILTER_W, &ICON_FILTER_B, dark)
}

pub fn icon_index(dark: bool) -> image::Handle {
    pick(&ICON_INDEX_W, &ICON_INDEX_B, dark)
}

pub fn icon_browse(dark: bool) -> image::Handle {
    pick(&ICON_BROWSE_W, &ICON_BROWSE_B, dark)
}

pub fn icon_save(dark: bool) -> image::Handle {
    pick(&ICON_SAVE_W, &ICON_SAVE_B, dark)
}

pub fn icon_youtube(dark: bool) -> image::Handle {
    pick(&ICON_YOUTUBE_W, &ICON_YOUTUBE_B, dark)
}

pub fn icon_generate(dark: bool) -> image::Handle {
    pick(&ICON_GENERATE_W, &ICON_GENERATE_B, dark)
}

pub fn icon_type_image(dark: bool) -> image::Handle {
    pick(&ICON_TYPE_IMAGE_W, &ICON_TYPE_IMAGE_B, dark)
}

pub fn icon_type_video(dark: bool) -> image::Handle {
    pick(&ICON_TYPE_VIDEO_W, &ICON_TYPE_VIDEO_B, dark)
}

fn make_image_handle(bytes: &'static [u8]) -> image::Handle {
    image::Handle::from_bytes(bytes.to_vec())
}

static LOGO_LIGHT: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!(
        "../../../assets/logo/index/librapix-icon-light.png"
    ))
});
static LOGO_DARK: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!(
        "../../../assets/logo/index/librapix-icon-dark.png"
    ))
});
static WORDMARK_LIGHT: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!(
        "../../../assets/logo/index/librapix-wordmark-light.png"
    ))
});
static WORDMARK_DARK: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!(
        "../../../assets/logo/index/librapix-wordmark-dark.png"
    ))
});

macro_rules! icon_pair {
    ($white:ident, $black:ident, $file:literal) => {
        static $white: LazyLock<image::Handle> = LazyLock::new(|| {
            make_image_handle(include_bytes!(concat!(
                "../../../assets/icons/white/",
                $file
            )))
        });
        static $black: LazyLock<image::Handle> = LazyLock::new(|| {
            make_image_handle(include_bytes!(concat!(
                "../../../assets/icons/black/",
                $file
            )))
        });
    };
}

icon_pair!(ICON_GALLERY_W, ICON_GALLERY_B, "gallary.png");
icon_pair!(ICON_TIMELINE_W, ICON_TIMELINE_B, "timeline.png");
icon_pair!(ICON_SEARCH_W, ICON_SEARCH_B, "search.png");
icon_pair!(ICON_REFRESH_W, ICON_REFRESH_B, "refresh.png");
icon_pair!(ICON_GITHUB_W, ICON_GITHUB_B, "github.png");
icon_pair!(ICON_OPEN_W, ICON_OPEN_B, "open.png");
icon_pair!(
    ICON_SHOW_IN_FOLDER_W,
    ICON_SHOW_IN_FOLDER_B,
    "show-in-folder.png"
);
icon_pair!(ICON_COPY_FILE_W, ICON_COPY_FILE_B, "copy-file.png");
icon_pair!(ICON_COPY_PATH_W, ICON_COPY_PATH_B, "copy-path.png");
icon_pair!(ICON_FILTER_W, ICON_FILTER_B, "filter.png");
icon_pair!(ICON_INDEX_W, ICON_INDEX_B, "index.png");
icon_pair!(ICON_BROWSE_W, ICON_BROWSE_B, "browse.png");
icon_pair!(ICON_SAVE_W, ICON_SAVE_B, "save.png");
icon_pair!(ICON_YOUTUBE_W, ICON_YOUTUBE_B, "youtube.png");
icon_pair!(ICON_GENERATE_W, ICON_GENERATE_B, "generate.png");
icon_pair!(ICON_TYPE_IMAGE_W, ICON_TYPE_IMAGE_B, "type-image.png");
icon_pair!(ICON_TYPE_VIDEO_W, ICON_TYPE_VIDEO_B, "type-video.png");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_brand_assets_are_theme_specific_and_window_icon_decodes() {
        assert_ne!(logo(true).id(), logo(false).id());
        assert_ne!(wordmark(true).id(), wordmark(false).id());
        assert_eq!(logo(true).id(), logo(true).id());
        assert_eq!(wordmark(false).id(), wordmark(false).id());
        assert!(window_icon().is_some());
    }
}
