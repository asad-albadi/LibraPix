use crate::error::VideoShortError;
use crate::ffmpeg::{resolve_ffmpeg, resolve_ffprobe};
use crate::models::{Effect, ShortGenerationRequest};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn validate_request(request: &ShortGenerationRequest) -> Result<(), VideoShortError> {
    let _ = resolve_ffmpeg()?;
    let _ = resolve_ffprobe()?;
    validate_semantics(request)
}

pub fn validate_semantics(request: &ShortGenerationRequest) -> Result<(), VideoShortError> {
    if !request.input_file.exists() {
        return Err(VideoShortError::InputFileMissing(
            request.input_file.clone(),
        ));
    }

    if request.options.speed <= 0.0 {
        return Err(VideoShortError::InvalidSpeed(request.options.speed));
    }

    if request.options.effects.contains(&Effect::Clean) && request.options.effects.len() > 1 {
        return Err(VideoShortError::CleanEffectExclusive);
    }

    ensure_output_path_writable(&request.output_file)
}

pub fn ensure_output_path_writable(path: &Path) -> Result<(), VideoShortError> {
    let parent = path
        .parent()
        .ok_or_else(|| VideoShortError::OutputPathInvalid(path.to_path_buf()))?;

    fs::create_dir_all(parent)
        .map_err(|_| VideoShortError::OutputPathInvalid(path.to_path_buf()))?;

    let probe_file = parent.join(format!(
        ".librapix-write-probe-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|v| v.as_nanos())
            .unwrap_or(0)
    ));

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&probe_file)
        .map_err(|_| VideoShortError::OutputPathNotWritable(PathBuf::from(parent)))?;

    file.write_all(b"ok")
        .map_err(|_| VideoShortError::OutputPathNotWritable(PathBuf::from(parent)))?;

    let _ = fs::remove_file(probe_file);

    Ok(())
}
