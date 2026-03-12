use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    Clean,
    Enhanced,
    Cinematic,
    Night,
    Scenic,
    Smooth,
}

impl Effect {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Enhanced => "enhanced",
            Self::Cinematic => "cinematic",
            Self::Night => "night",
            Self::Scenic => "scenic",
            Self::Smooth => "smooth",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CropPosition {
    Center,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Fast,
    Medium,
    Slow,
}

impl Preset {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Medium => "medium",
            Self::Slow => "slow",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShortGenerationOptions {
    pub effects: Vec<Effect>,
    pub crop_position: CropPosition,
    pub add_fade: bool,
    pub speed: f64,
    pub crf: i32,
    pub preset: Preset,
}

impl Default for ShortGenerationOptions {
    fn default() -> Self {
        Self {
            effects: vec![Effect::Enhanced],
            crop_position: CropPosition::Center,
            add_fade: false,
            speed: 1.0,
            crf: 18,
            preset: Preset::Medium,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShortGenerationRequest {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub options: ShortGenerationOptions,
}

#[derive(Debug, Clone)]
pub struct FfmpegArgs {
    pub ffmpeg_path: PathBuf,
    pub ffprobe_path: PathBuf,
    pub args: Vec<String>,
    pub video_filter: String,
    pub audio_filter: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationStage {
    Preparing,
    Probing,
    BuildingFilters,
    Generating,
    Finalizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ShortGenerationResult {
    pub output_file: PathBuf,
    pub ffmpeg_exit_code: Option<i32>,
    pub ffmpeg_stderr: String,
}
