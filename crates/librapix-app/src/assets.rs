//! Centralized asset path resolution for branding and icons.
//!
//! Resolves assets from executable-relative paths in packaged builds, and
//! falls back to workspace-relative paths for development.

use std::path::{Path, PathBuf};

/// Project repository URL for the GitHub link in the app header.
pub const REPO_URL: &str = "https://github.com/asad-albadi/LibraPix";

/// Returns the assets directory (workspace root / assets).
/// Resolution order:
/// 1) `LIBRAPIX_ASSETS_DIR` env override.
/// 2) Executable-relative candidates (for packaged artifacts).
/// 3) Workspace-relative fallback for development.
fn assets_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var("LIBRAPIX_ASSETS_DIR") {
        let path = PathBuf::from(override_dir);
        if path.is_dir() {
            return path;
        }
    }

    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let packaged_candidates = [
            exe_dir.join("assets"),
            exe_dir.join("../assets"),
            exe_dir.join("../../assets"),
        ];

        for candidate in packaged_candidates {
            if candidate.is_dir() {
                return candidate;
            }
        }
    }

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
