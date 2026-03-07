//! Centralized embedded assets for branding and icons.
//!
//! Assets are compiled into the binary so release executables do not depend
//! on an external `assets/` folder for UI icon rendering.
use iced::widget::{image, svg};

/// Project repository URL for the GitHub link in the app header.
pub const REPO_URL: &str = "https://github.com/asad-albadi/LibraPix";

// ── Canonical brand (blue) ──

#[allow(dead_code)]
pub fn logo_icon_64() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/logo/blue/icon-64.png").to_vec())
}

/// Blue logo as SVG for scalable display (e.g. header).
pub fn logo_svg() -> svg::Handle {
    svg::Handle::from_memory(include_bytes!("../../../assets/logo/blue/logo-blue.svg").to_vec())
}

// ── UI icons (white for dark surfaces) ──

pub fn icon_gallery() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/gallary.png").to_vec())
}

pub fn icon_timeline() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/timeline.png").to_vec())
}

pub fn icon_search() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/search.png").to_vec())
}

pub fn icon_refresh() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/refresh.png").to_vec())
}

pub fn icon_github() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/github.png").to_vec())
}

pub fn icon_open() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/open.png").to_vec())
}

pub fn icon_show_in_folder() -> image::Handle {
    image::Handle::from_bytes(
        include_bytes!("../../../assets/icons/white/show-in-folder.png").to_vec(),
    )
}

pub fn icon_copy_file() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/copy-file.png").to_vec())
}

pub fn icon_copy_path() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/copy-path.png").to_vec())
}

pub fn icon_filter() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/filter.png").to_vec())
}

#[allow(dead_code)]
pub fn icon_filter_remove() -> image::Handle {
    image::Handle::from_bytes(
        include_bytes!("../../../assets/icons/white/filter-remove.png").to_vec(),
    )
}

pub fn icon_index() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/index.png").to_vec())
}

pub fn icon_type_image() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/type-image.png").to_vec())
}

pub fn icon_type_video() -> image::Handle {
    image::Handle::from_bytes(include_bytes!("../../../assets/icons/white/type-video.png").to_vec())
}
