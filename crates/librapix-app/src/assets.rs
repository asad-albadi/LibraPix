//! Centralized asset path resolution for branding and icons.
//!
//! Uses workspace-relative paths for development. Assets live at `assets/`
//! in the repository root.

use std::path::{Path, PathBuf};

/// Project repository URL for the GitHub link in the app header.
pub const REPO_URL: &str = "https://github.com/asad-albadi/librapix";

/// Returns the assets directory (workspace root / assets).
/// Resolves from CARGO_MANIFEST_DIR: crates/librapix-app -> workspace root.
fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap_or_else(|| Path::new("."))
        .join("assets")
}

// ── Canonical brand (blue) ──

#[allow(dead_code)]
pub fn logo_icon_64() -> PathBuf {
    assets_dir().join("logo/blue/icon-64.png")
}

/// Blue logo as SVG for scalable display (e.g. header).
pub fn logo_svg() -> PathBuf {
    assets_dir().join("logo/blue/logo-blue.svg")
}

// ── UI icons (white for dark surfaces) ──

pub fn icon_gallery() -> PathBuf {
    assets_dir().join("icons/white/gallary.png")
}

pub fn icon_timeline() -> PathBuf {
    assets_dir().join("icons/white/timeline.png")
}

pub fn icon_search() -> PathBuf {
    assets_dir().join("icons/white/search.png")
}

pub fn icon_refresh() -> PathBuf {
    assets_dir().join("icons/white/refresh.png")
}

pub fn icon_github() -> PathBuf {
    assets_dir().join("icons/white/github.png")
}

pub fn icon_open() -> PathBuf {
    assets_dir().join("icons/white/open.png")
}

pub fn icon_show_in_folder() -> PathBuf {
    assets_dir().join("icons/white/show-in-folder.png")
}

pub fn icon_copy_file() -> PathBuf {
    assets_dir().join("icons/white/copy-file.png")
}

pub fn icon_copy_path() -> PathBuf {
    assets_dir().join("icons/white/copy-path.png")
}

pub fn icon_filter() -> PathBuf {
    assets_dir().join("icons/white/filter.png")
}

#[allow(dead_code)]
pub fn icon_filter_remove() -> PathBuf {
    assets_dir().join("icons/white/filter-remove.png")
}

pub fn icon_index() -> PathBuf {
    assets_dir().join("icons/white/index.png")
}

pub fn icon_type_image() -> PathBuf {
    assets_dir().join("icons/white/type-image.png")
}

pub fn icon_type_video() -> PathBuf {
    assets_dir().join("icons/white/type-video.png")
}
