use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Image,
    Video,
}

impl MediaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MediaKind::Image => "image",
            MediaKind::Video => "video",
        }
    }
}

pub fn classify_media_kind(path: &Path) -> Option<MediaKind> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "tif" | "tiff" => Some(MediaKind::Image),
        "mp4" | "mov" | "mkv" | "webm" | "avi" => Some(MediaKind::Video),
        _ => None,
    }
}

pub fn extract_image_dimensions(path: &Path) -> Option<(u32, u32)> {
    let size = imagesize::size(path).ok()?;
    Some((size.width as u32, size.height as u32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_supported_extensions() {
        assert_eq!(
            classify_media_kind(&PathBuf::from("/tmp/a.PNG")),
            Some(MediaKind::Image)
        );
        assert_eq!(
            classify_media_kind(&PathBuf::from("/tmp/a.mp4")),
            Some(MediaKind::Video)
        );
        assert_eq!(classify_media_kind(&PathBuf::from("/tmp/a.txt")), None);
    }

    #[test]
    fn returns_none_for_non_image_dimensions() {
        assert_eq!(extract_image_dimensions(&PathBuf::from("/tmp/a.mp4")), None);
    }
}
