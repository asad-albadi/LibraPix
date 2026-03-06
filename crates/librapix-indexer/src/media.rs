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
}
