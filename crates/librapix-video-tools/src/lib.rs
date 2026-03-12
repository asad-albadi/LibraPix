pub mod error;
pub mod ffmpeg;
pub mod filters;
pub mod models;
pub mod paths;
pub mod probe;
pub mod runner;
pub mod validate;

pub use error::VideoShortError;
pub use models::{
    CropPosition, Effect, FfmpegArgs, GenerationStage, Preset, ShortGenerationOptions,
    ShortGenerationRequest, ShortGenerationResult,
};
pub use paths::default_shorts_output_dir;
pub use runner::{prepare_generation, run_generation};
