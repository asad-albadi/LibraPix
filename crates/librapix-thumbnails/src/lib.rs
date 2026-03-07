use image::ImageFormat;
use image::ImageReader;
use image::imageops::FilterType;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
    max_edge: u32,
) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(source_path.to_string_lossy().as_bytes());
    hasher.update(file_size_bytes.to_le_bytes());
    hasher.update(modified_unix_seconds.unwrap_or_default().to_le_bytes());
    hasher.update(max_edge.to_le_bytes());
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
        max_edge,
    );
    if output.exists() {
        return Ok(ThumbnailOutcome {
            thumbnail_path: output,
            generated: false,
        });
    }

    let image = ImageReader::open(source_path)?.decode()?;
    let thumbnail = image.resize(max_edge, max_edge, FilterType::Lanczos3);
    thumbnail.save_with_format(&output, ImageFormat::Png)?;
    Ok(ThumbnailOutcome {
        thumbnail_path: output,
        generated: true,
    })
}

/// Normalize path for ffmpeg subprocess. On Windows, use forward slashes
/// since ffmpeg accepts them and they avoid backslash escaping issues.
fn path_for_ffmpeg(p: &Path) -> String {
    let s = p.display().to_string();
    #[cfg(windows)]
    {
        s.replace('\\', "/")
    }
    #[cfg(not(windows))]
    {
        s
    }
}

pub fn ensure_video_thumbnail(
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
        max_edge,
    );
    if output.exists() {
        return Ok(ThumbnailOutcome {
            thumbnail_path: output,
            generated: false,
        });
    }

    let scale_filter = format!("scale={max_edge}:{max_edge}:force_original_aspect_ratio=decrease");
    let source_str = path_for_ffmpeg(source_path);
    let output_str = path_for_ffmpeg(&output);

    #[cfg(windows)]
    let ffmpeg_cmd = "ffmpeg.exe";
    #[cfg(not(windows))]
    let ffmpeg_cmd = "ffmpeg";

    let mut command = std::process::Command::new(ffmpeg_cmd);
    command
        .args([
            "-y",
            "-i",
            &source_str,
            "-ss",
            "00:00:01",
            "-frames:v",
            "1",
            "-vf",
            &scale_filter,
            &output_str,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let status = command.status();

    match status {
        Ok(s) if s.success() && output.exists() => Ok(ThumbnailOutcome {
            thumbnail_path: output,
            generated: true,
        }),
        _ => Err(ThumbnailError::Io(std::io::Error::other(
            "video thumbnail extraction failed (ffmpeg may not be installed or in PATH)",
        ))),
    }
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
            256,
        );
        let b = thumbnail_path(
            Path::new("/tmp/thumbs"),
            Path::new("/tmp/a.png"),
            10,
            Some(123),
            256,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn different_max_edge_produces_different_path() {
        let a = thumbnail_path(
            Path::new("/tmp/thumbs"),
            Path::new("/tmp/a.png"),
            10,
            Some(123),
            256,
        );
        let b = thumbnail_path(
            Path::new("/tmp/thumbs"),
            Path::new("/tmp/a.png"),
            10,
            Some(123),
            400,
        );
        assert_ne!(a, b);
    }
}
