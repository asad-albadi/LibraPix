use crate::error::VideoShortError;
use crate::ffmpeg::{path_for_ffmpeg, resolve_ffprobe};
use std::process::Command;

pub fn read_video_duration_seconds(path: &std::path::Path) -> Result<f64, VideoShortError> {
    let ffprobe = resolve_ffprobe()?;
    let input = path_for_ffmpeg(path);

    let output = Command::new(ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            &input,
        ])
        .output()
        .map_err(|e| VideoShortError::ProbeFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(VideoShortError::ProbeFailed(stderr));
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if raw.is_empty() {
        return Err(VideoShortError::ProbeFailed(
            "empty duration output".to_owned(),
        ));
    }

    raw.parse::<f64>()
        .map_err(|_| VideoShortError::ProbeParseFailed(raw))
}
