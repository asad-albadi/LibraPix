use image::ImageFormat;
use image::ImageReader;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ThumbnailError {
    Io(std::io::Error),
    Image(image::ImageError),
}

impl Display for ThumbnailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbnailError::Io(error) => write!(f, "{error}"),
            ThumbnailError::Image(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ThumbnailError {}

impl From<std::io::Error> for ThumbnailError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<image::ImageError> for ThumbnailError {
    fn from(value: image::ImageError) -> Self {
        Self::Image(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailOutcome {
    pub thumbnail_path: PathBuf,
    pub generated: bool,
}

pub fn thumbnail_path(
    thumbnails_dir: &Path,
    source_path: &Path,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(source_path.to_string_lossy().as_bytes());
    hasher.update(file_size_bytes.to_le_bytes());
    hasher.update(modified_unix_seconds.unwrap_or_default().to_le_bytes());
    let digest = hasher.finalize();
    let filename = digest
        .iter()
        .map(|value| format!("{value:02x}"))
        .collect::<String>();
    thumbnails_dir.join(format!("{filename}.png"))
}

pub fn ensure_image_thumbnail(
    thumbnails_dir: &Path,
    source_path: &Path,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    max_edge: u32,
) -> Result<ThumbnailOutcome, ThumbnailError> {
    fs::create_dir_all(thumbnails_dir)?;
    let output = thumbnail_path(
        thumbnails_dir,
        source_path,
        file_size_bytes,
        modified_unix_seconds,
    );
    if output.exists() {
        return Ok(ThumbnailOutcome {
            thumbnail_path: output,
            generated: false,
        });
    }

    let image = ImageReader::open(source_path)?.decode()?;
    let thumbnail = image.thumbnail(max_edge, max_edge);
    thumbnail.save_with_format(&output, ImageFormat::Png)?;
    Ok(ThumbnailOutcome {
        thumbnail_path: output,
        generated: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnail_path_is_deterministic() {
        let a = thumbnail_path(
            Path::new("/tmp/thumbs"),
            Path::new("/tmp/a.png"),
            10,
            Some(123),
        );
        let b = thumbnail_path(
            Path::new("/tmp/thumbs"),
            Path::new("/tmp/a.png"),
            10,
            Some(123),
        );
        assert_eq!(a, b);
    }
}
