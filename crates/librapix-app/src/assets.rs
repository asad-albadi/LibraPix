//! Centralized embedded assets for branding and icons.
//!
//! Assets are compiled into the binary so release executables do not depend
//! on an external `assets/` folder for UI icon rendering. UI icons ship in two
//! tints — white (for dark surfaces) and black (for light surfaces) — and the
//! accessors pick the variant from the active theme via a `dark` flag.
use iced::widget::{image, svg};
use std::sync::LazyLock;

/// Project repository URL for the GitHub link in the app header.
pub const REPO_URL: &str = "https://github.com/asad-albadi/LibraPix";

// ── Canonical brand ──

#[allow(dead_code)]
pub fn logo_icon_64() -> image::Handle {
    LOGO_ICON_64.clone()
}

/// Logo as SVG for scalable display (e.g. header): white on dark, black on light.
pub fn logo_svg(dark: bool) -> svg::Handle {
    if dark {
        LOGO_SVG_WHITE.clone()
    } else {
        LOGO_SVG_BLACK.clone()
    }
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

#[allow(dead_code)]
pub fn icon_filter_remove(dark: bool) -> image::Handle {
    pick(&ICON_FILTER_REMOVE_W, &ICON_FILTER_REMOVE_B, dark)
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

fn make_svg_handle(bytes: &'static [u8]) -> svg::Handle {
    svg::Handle::from_memory(bytes.to_vec())
}

static LOGO_ICON_64: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/logo/blue/icon-64.png")));
static LOGO_SVG_WHITE: LazyLock<svg::Handle> =
    LazyLock::new(|| make_svg_handle(include_bytes!("../../../assets/logo/white/logo-white.svg")));
static LOGO_SVG_BLACK: LazyLock<svg::Handle> =
    LazyLock::new(|| make_svg_handle(include_bytes!("../../../assets/logo/black/logo-black.svg")));

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
icon_pair!(
    ICON_FILTER_REMOVE_W,
    ICON_FILTER_REMOVE_B,
    "filter-remove.png"
);
icon_pair!(ICON_INDEX_W, ICON_INDEX_B, "index.png");
icon_pair!(ICON_BROWSE_W, ICON_BROWSE_B, "browse.png");
icon_pair!(ICON_SAVE_W, ICON_SAVE_B, "save.png");
icon_pair!(ICON_YOUTUBE_W, ICON_YOUTUBE_B, "youtube.png");
icon_pair!(ICON_GENERATE_W, ICON_GENERATE_B, "generate.png");
icon_pair!(ICON_TYPE_IMAGE_W, ICON_TYPE_IMAGE_B, "type-image.png");
icon_pair!(ICON_TYPE_VIDEO_W, ICON_TYPE_VIDEO_B, "type-video.png");
