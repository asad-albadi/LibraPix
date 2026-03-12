use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum VideoShortError {
    FfmpegNotFound,
    FfprobeNotFound,
    InputFileMissing(PathBuf),
    InvalidSpeed(f64),
    CleanEffectExclusive,
    OutputPathInvalid(PathBuf),
    OutputPathNotWritable(PathBuf),
    ProbeFailed(String),
    ProbeParseFailed(String),
    FfmpegSpawnFailed(String),
    FfmpegFailed {
        exit_code: Option<i32>,
        stderr: String,
    },
}

impl Display for VideoShortError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FfmpegNotFound => write!(f, "ffmpeg was not found in PATH."),
            Self::FfprobeNotFound => write!(f, "ffprobe was not found in PATH."),
            Self::InputFileMissing(path) => write!(f, "Input file not found: {}", path.display()),
            Self::InvalidSpeed(speed) => write!(f, "Speed must be greater than 0. (got {speed})"),
            Self::CleanEffectExclusive => {
                write!(f, "'clean' cannot be combined with other effects.")
            }
            Self::OutputPathInvalid(path) => {
                write!(
                    f,
                    "Output path is invalid or cannot be resolved: {}",
                    path.display()
                )
            }
            Self::OutputPathNotWritable(path) => {
                write!(f, "Output path is not writable: {}", path.display())
            }
            Self::ProbeFailed(detail) => {
                write!(f, "Could not read video duration with ffprobe: {detail}")
            }
            Self::ProbeParseFailed(value) => {
                write!(f, "Could not parse ffprobe duration as number: {value}")
            }
            Self::FfmpegSpawnFailed(detail) => write!(f, "ffmpeg could not start: {detail}"),
            Self::FfmpegFailed { exit_code, stderr } => {
                write!(f, "ffmpeg failed (exit_code={exit_code:?}): {stderr}")
            }
        }
    }
}

impl std::error::Error for VideoShortError {}
