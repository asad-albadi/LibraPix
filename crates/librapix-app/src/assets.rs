//! Centralized embedded assets for branding and icons.
//!
//! Assets are compiled into the binary so release executables do not depend
//! on an external `assets/` folder for UI icon rendering.
use iced::widget::{image, svg};
use std::sync::LazyLock;

/// Project repository URL for the GitHub link in the app header.
pub const REPO_URL: &str = "https://github.com/asad-albadi/LibraPix";

// ── Canonical brand (blue) ──

#[allow(dead_code)]
pub fn logo_icon_64() -> image::Handle {
    LOGO_ICON_64.clone()
}

/// Blue logo as SVG for scalable display (e.g. header).
pub fn logo_svg() -> svg::Handle {
    LOGO_SVG.clone()
}

// ── UI icons (white for dark surfaces) ──

pub fn icon_gallery() -> image::Handle {
    ICON_GALLERY.clone()
}

pub fn icon_timeline() -> image::Handle {
    ICON_TIMELINE.clone()
}

pub fn icon_search() -> image::Handle {
    ICON_SEARCH.clone()
}

pub fn icon_refresh() -> image::Handle {
    ICON_REFRESH.clone()
}

pub fn icon_github() -> image::Handle {
    ICON_GITHUB.clone()
}

pub fn icon_open() -> image::Handle {
    ICON_OPEN.clone()
}

pub fn icon_show_in_folder() -> image::Handle {
    ICON_SHOW_IN_FOLDER.clone()
}

pub fn icon_copy_file() -> image::Handle {
    ICON_COPY_FILE.clone()
}

pub fn icon_copy_path() -> image::Handle {
    ICON_COPY_PATH.clone()
}

pub fn icon_filter() -> image::Handle {
    ICON_FILTER.clone()
}

#[allow(dead_code)]
pub fn icon_filter_remove() -> image::Handle {
    ICON_FILTER_REMOVE.clone()
}

pub fn icon_index() -> image::Handle {
    ICON_INDEX.clone()
}

pub fn icon_type_image() -> image::Handle {
    ICON_TYPE_IMAGE.clone()
}

pub fn icon_type_video() -> image::Handle {
    ICON_TYPE_VIDEO.clone()
}

fn make_image_handle(bytes: &'static [u8]) -> image::Handle {
    image::Handle::from_bytes(bytes.to_vec())
}

fn make_svg_handle(bytes: &'static [u8]) -> svg::Handle {
    svg::Handle::from_memory(bytes.to_vec())
}

static LOGO_ICON_64: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/logo/blue/icon-64.png")));
static LOGO_SVG: LazyLock<svg::Handle> =
    LazyLock::new(|| make_svg_handle(include_bytes!("../../../assets/logo/blue/logo-blue.svg")));

static ICON_GALLERY: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/gallary.png")));
static ICON_TIMELINE: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/timeline.png")));
static ICON_SEARCH: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/search.png")));
static ICON_REFRESH: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/refresh.png")));
static ICON_GITHUB: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/github.png")));
static ICON_OPEN: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/open.png")));
static ICON_SHOW_IN_FOLDER: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!(
        "../../../assets/icons/white/show-in-folder.png"
    ))
});
static ICON_COPY_FILE: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!("../../../assets/icons/white/copy-file.png"))
});
static ICON_COPY_PATH: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!("../../../assets/icons/white/copy-path.png"))
});
static ICON_FILTER: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/filter.png")));
static ICON_FILTER_REMOVE: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!(
        "../../../assets/icons/white/filter-remove.png"
    ))
});
static ICON_INDEX: LazyLock<image::Handle> =
    LazyLock::new(|| make_image_handle(include_bytes!("../../../assets/icons/white/index.png")));
static ICON_TYPE_IMAGE: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!("../../../assets/icons/white/type-image.png"))
});
static ICON_TYPE_VIDEO: LazyLock<image::Handle> = LazyLock::new(|| {
    make_image_handle(include_bytes!("../../../assets/icons/white/type-video.png"))
});
