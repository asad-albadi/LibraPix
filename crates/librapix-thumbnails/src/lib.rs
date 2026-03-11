use image::ImageFormat;
use image::ImageReader;
use image::imageops::FilterType;
use sha2::{Digest, Sha256};
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::fs;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;
const VIDEO_PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);
const STDERR_SUMMARY_LIMIT: usize = 240;
pub const DEFAULT_VIDEO_THUMBNAIL_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoThumbnailErrorKind {
    FfmpegNotFound,
    SpawnFailed,
    TimedOut,
    ExitNonZero,
    MissingOutput,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct VideoThumbnailError {
    pub kind: VideoThumbnailErrorKind,
    pub ffmpeg_path: Option<PathBuf>,
    pub command_line: String,
    pub exit_code: Option<i32>,
    pub stderr_summary: Option<String>,
    pub timeout_ms: Option<u128>,
}

#[derive(Debug)]
pub enum ThumbnailError {
    Io(std::io::Error),
    Image(image::ImageError),
    Video(Box<VideoThumbnailError>),
}

impl Display for ThumbnailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbnailError::Io(error) => write!(f, "{error}"),
            ThumbnailError::Image(error) => write!(f, "{error}"),
            ThumbnailError::Video(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ThumbnailError {}

impl Display for VideoThumbnailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.kind {
            VideoThumbnailErrorKind::FfmpegNotFound => {
                "video thumbnail extraction failed: ffmpeg executable not found"
            }
            VideoThumbnailErrorKind::SpawnFailed => {
                "video thumbnail extraction failed: ffmpeg could not start"
            }
            VideoThumbnailErrorKind::TimedOut => "video thumbnail extraction failed: timed out",
            VideoThumbnailErrorKind::ExitNonZero => {
                "video thumbnail extraction failed: ffmpeg exited with an error"
            }
            VideoThumbnailErrorKind::MissingOutput => {
                "video thumbnail extraction failed: ffmpeg produced no output thumbnail"
            }
            VideoThumbnailErrorKind::Cancelled => {
                "video thumbnail extraction cancelled before completion"
            }
        };
        write!(f, "{prefix}")?;
        if let Some(path) = &self.ffmpeg_path {
            write!(f, " ffmpeg={}", path.display())?;
        }
        if let Some(code) = self.exit_code {
            write!(f, " exit_code={code}")?;
        }
        if let Some(timeout_ms) = self.timeout_ms {
            write!(f, " timeout_ms={timeout_ms}")?;
        }
        if let Some(stderr_summary) = &self.stderr_summary {
            write!(f, " stderr={stderr_summary}")?;
        }
        if !self.command_line.is_empty() {
            write!(f, " command={}", self.command_line)?;
        }
        Ok(())
    }
}

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

#[derive(Debug, Clone)]
pub struct ThumbnailCancellation {
    token: Arc<AtomicU64>,
    expected_generation: u64,
}

impl ThumbnailCancellation {
    pub fn new(token: Arc<AtomicU64>, expected_generation: u64) -> Self {
        Self {
            token,
            expected_generation,
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.load(Ordering::Relaxed) != self.expected_generation
    }
}

#[derive(Debug, Clone)]
pub struct VideoThumbnailOptions {
    pub timeout: Duration,
    pub cancellation: Option<ThumbnailCancellation>,
}

impl Default for VideoThumbnailOptions {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_VIDEO_THUMBNAIL_TIMEOUT,
            cancellation: None,
        }
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
    ensure_video_thumbnail_with_options(
        thumbnails_dir,
        source_path,
        file_size_bytes,
        modified_unix_seconds,
        max_edge,
        VideoThumbnailOptions::default(),
    )
}

pub fn ensure_video_thumbnail_with_options(
    thumbnails_dir: &Path,
    source_path: &Path,
    file_size_bytes: u64,
    modified_unix_seconds: Option<i64>,
    max_edge: u32,
    options: VideoThumbnailOptions,
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

    if options
        .cancellation
        .as_ref()
        .is_some_and(ThumbnailCancellation::is_cancelled)
    {
        return Err(ThumbnailError::Video(Box::new(VideoThumbnailError {
            kind: VideoThumbnailErrorKind::Cancelled,
            ffmpeg_path: None,
            command_line: String::new(),
            exit_code: None,
            stderr_summary: None,
            timeout_ms: None,
        })));
    }

    let scale_filter = format!("scale={max_edge}:{max_edge}:force_original_aspect_ratio=decrease");
    let ffmpeg_path = resolve_ffmpeg_binary().map_err(|message| {
        ThumbnailError::Video(Box::new(VideoThumbnailError {
            kind: VideoThumbnailErrorKind::FfmpegNotFound,
            ffmpeg_path: None,
            command_line: message,
            exit_code: None,
            stderr_summary: None,
            timeout_ms: None,
        }))
    })?;
    let source_str = path_for_ffmpeg(source_path);
    let output_str = path_for_ffmpeg(&output);
    let args = vec![
        "-nostdin".to_owned(),
        "-hide_banner".to_owned(),
        "-loglevel".to_owned(),
        "error".to_owned(),
        "-y".to_owned(),
        "-ss".to_owned(),
        "00:00:01".to_owned(),
        "-i".to_owned(),
        source_str,
        "-frames:v".to_owned(),
        "1".to_owned(),
        "-vf".to_owned(),
        scale_filter,
        output_str,
    ];
    let command_line = format_command_line(&ffmpeg_path, &args);
    let mut command = Command::new(&ffmpeg_path);
    command
        .args(args.iter().map(String::as_str))
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = command.spawn().map_err(|error| {
        ThumbnailError::Video(Box::new(VideoThumbnailError {
            kind: VideoThumbnailErrorKind::SpawnFailed,
            ffmpeg_path: Some(ffmpeg_path.clone()),
            command_line: command_line.clone(),
            exit_code: None,
            stderr_summary: Some(error.to_string()),
            timeout_ms: None,
        }))
    })?;
    let started_at = Instant::now();

    loop {
        if options
            .cancellation
            .as_ref()
            .is_some_and(ThumbnailCancellation::is_cancelled)
        {
            let _ = child.kill();
            let output_capture = child.wait_with_output().ok();
            return Err(ThumbnailError::Video(Box::new(VideoThumbnailError {
                kind: VideoThumbnailErrorKind::Cancelled,
                ffmpeg_path: Some(ffmpeg_path.clone()),
                command_line,
                exit_code: output_capture
                    .as_ref()
                    .and_then(|capture| capture.status.code()),
                stderr_summary: output_capture
                    .as_ref()
                    .and_then(|capture| summarize_stderr(&capture.stderr)),
                timeout_ms: Some(started_at.elapsed().as_millis()),
            })));
        }

        match child.try_wait() {
            Ok(Some(_)) => {
                let output_capture = child.wait_with_output().map_err(|error| {
                    ThumbnailError::Video(Box::new(VideoThumbnailError {
                        kind: VideoThumbnailErrorKind::SpawnFailed,
                        ffmpeg_path: Some(ffmpeg_path.clone()),
                        command_line: command_line.clone(),
                        exit_code: None,
                        stderr_summary: Some(error.to_string()),
                        timeout_ms: Some(started_at.elapsed().as_millis()),
                    }))
                })?;
                return finalize_video_thumbnail(
                    output,
                    output_capture,
                    ffmpeg_path,
                    command_line,
                    started_at.elapsed(),
                );
            }
            Ok(None) => {
                if started_at.elapsed() >= options.timeout {
                    let _ = child.kill();
                    let output_capture = child.wait_with_output().ok();
                    return Err(ThumbnailError::Video(Box::new(VideoThumbnailError {
                        kind: VideoThumbnailErrorKind::TimedOut,
                        ffmpeg_path: Some(ffmpeg_path.clone()),
                        command_line,
                        exit_code: output_capture
                            .as_ref()
                            .and_then(|capture| capture.status.code()),
                        stderr_summary: output_capture
                            .as_ref()
                            .and_then(|capture| summarize_stderr(&capture.stderr)),
                        timeout_ms: Some(started_at.elapsed().as_millis()),
                    })));
                }
                thread::sleep(VIDEO_PROCESS_POLL_INTERVAL);
            }
            Err(error) => {
                let _ = child.kill();
                let output_capture = child.wait_with_output().ok();
                return Err(ThumbnailError::Video(Box::new(VideoThumbnailError {
                    kind: VideoThumbnailErrorKind::SpawnFailed,
                    ffmpeg_path: Some(ffmpeg_path),
                    command_line,
                    exit_code: output_capture
                        .as_ref()
                        .and_then(|capture| capture.status.code()),
                    stderr_summary: Some(error.to_string()),
                    timeout_ms: Some(started_at.elapsed().as_millis()),
                })));
            }
        }
    }
}

fn resolve_ffmpeg_binary() -> Result<PathBuf, String> {
    static FFMPEG_PATH: OnceLock<Result<PathBuf, String>> = OnceLock::new();

    FFMPEG_PATH.get_or_init(locate_ffmpeg_binary).clone()
}

fn locate_ffmpeg_binary() -> Result<PathBuf, String> {
    let path_var = env::var_os("PATH");
    let current_dir = env::current_dir().ok();
    locate_binary_with_path_fallback(ffmpeg_executable_name(), path_var, current_dir.as_deref())
}

fn ffmpeg_executable_name() -> &'static str {
    #[cfg(windows)]
    {
        "ffmpeg.exe"
    }
    #[cfg(not(windows))]
    {
        "ffmpeg"
    }
}

fn locate_binary_with_path_fallback(
    executable: &str,
    path_var: Option<OsString>,
    current_dir: Option<&Path>,
) -> Result<PathBuf, String> {
    if let Some(candidate) = locate_binary_on_path(executable, path_var.as_deref()) {
        return Ok(candidate);
    }

    if let Some(current_dir) = current_dir {
        let local_candidate = current_dir.join(executable);
        if local_candidate.is_file() {
            return Ok(local_candidate);
        }
    }

    if path_var.is_none() {
        Err(format!("{executable} not found because PATH is empty"))
    } else {
        Err(format!("{executable} not found on PATH"))
    }
}

fn locate_binary_on_path(executable: &str, path_var: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
    let path_var = path_var?;
    for directory in env::split_paths(path_var) {
        let candidate = directory.join(executable);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn format_command_line(program: &Path, args: &[String]) -> String {
    let mut parts = Vec::with_capacity(args.len() + 1);
    parts.push(quote_shell_token(&program.display().to_string()));
    parts.extend(args.iter().map(|arg| quote_shell_token(arg)));
    parts.join(" ")
}

fn quote_shell_token(token: &str) -> String {
    if token.chars().any(|ch| ch.is_whitespace() || ch == '"') {
        format!("\"{}\"", token.replace('"', "\\\""))
    } else {
        token.to_owned()
    }
}

fn summarize_stderr(stderr: &[u8]) -> Option<String> {
    let summary = String::from_utf8_lossy(stderr)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" / ");
    if summary.is_empty() {
        return None;
    }
    if summary.len() <= STDERR_SUMMARY_LIMIT {
        Some(summary)
    } else {
        Some(format!("{}...", &summary[..STDERR_SUMMARY_LIMIT]))
    }
}

fn finalize_video_thumbnail(
    output_path: PathBuf,
    output_capture: Output,
    ffmpeg_path: PathBuf,
    command_line: String,
    elapsed: Duration,
) -> Result<ThumbnailOutcome, ThumbnailError> {
    if output_capture.status.success() && output_path.exists() {
        return Ok(ThumbnailOutcome {
            thumbnail_path: output_path,
            generated: true,
        });
    }

    let kind = if output_capture.status.success() {
        VideoThumbnailErrorKind::MissingOutput
    } else {
        VideoThumbnailErrorKind::ExitNonZero
    };
    Err(ThumbnailError::Video(Box::new(VideoThumbnailError {
        kind,
        ffmpeg_path: Some(ffmpeg_path),
        command_line,
        exit_code: output_capture.status.code(),
        stderr_summary: summarize_stderr(&output_capture.stderr),
        timeout_ms: Some(elapsed.as_millis()),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "librapix-thumbnails-{name}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be after epoch")
                .as_nanos()
        ))
    }

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

    #[test]
    fn summarize_stderr_collapses_whitespace() {
        let summary = summarize_stderr(b"\nfirst line\n\nsecond line\n").expect("summary exists");
        assert_eq!(summary, "first line / second line");
    }

    #[test]
    fn cancellation_token_detects_generation_change() {
        let token = Arc::new(AtomicU64::new(7));
        let cancellation = ThumbnailCancellation::new(token.clone(), 7);
        assert!(!cancellation.is_cancelled());
        token.store(8, Ordering::Relaxed);
        assert!(cancellation.is_cancelled());
    }

    #[test]
    fn locate_binary_prefers_path_match_over_current_directory_file() {
        let executable = ffmpeg_executable_name();
        let path_dir = temp_dir("path");
        let cwd_dir = temp_dir("cwd");
        fs::create_dir_all(&path_dir).expect("path dir should exist");
        fs::create_dir_all(&cwd_dir).expect("cwd dir should exist");
        let path_candidate = path_dir.join(executable);
        let cwd_candidate = cwd_dir.join(executable);
        fs::write(&path_candidate, b"path").expect("path candidate should be created");
        fs::write(&cwd_candidate, b"cwd").expect("cwd candidate should be created");

        let resolved = locate_binary_with_path_fallback(
            executable,
            Some(env::join_paths([path_dir.as_path()]).expect("path should join")),
            Some(&cwd_dir),
        )
        .expect("binary should resolve");

        assert_eq!(resolved, path_candidate);

        let _ = fs::remove_dir_all(path_dir);
        let _ = fs::remove_dir_all(cwd_dir);
    }

    #[test]
    fn locate_binary_falls_back_to_current_directory_when_path_misses() {
        let executable = ffmpeg_executable_name();
        let path_dir = temp_dir("empty-path");
        let cwd_dir = temp_dir("fallback-cwd");
        fs::create_dir_all(&path_dir).expect("path dir should exist");
        fs::create_dir_all(&cwd_dir).expect("cwd dir should exist");
        let cwd_candidate = cwd_dir.join(executable);
        fs::write(&cwd_candidate, b"cwd").expect("cwd candidate should be created");

        let resolved = locate_binary_with_path_fallback(
            executable,
            Some(env::join_paths([path_dir.as_path()]).expect("path should join")),
            Some(&cwd_dir),
        )
        .expect("binary should resolve");

        assert_eq!(resolved, cwd_candidate);

        let _ = fs::remove_dir_all(path_dir);
        let _ = fs::remove_dir_all(cwd_dir);
    }
}
