use crate::error::VideoShortError;
use crate::ffmpeg::{
    configure_background_command, path_for_ffmpeg, resolve_ffmpeg, resolve_ffprobe,
};
use crate::filters::{audio_filter, video_filter};
use crate::models::{FfmpegArgs, GenerationStage, ShortGenerationRequest, ShortGenerationResult};
use crate::probe::read_video_duration_seconds;
use crate::validate::validate_request;
use std::process::{Command, Stdio};

pub fn prepare_generation(request: &ShortGenerationRequest) -> Result<FfmpegArgs, VideoShortError> {
    validate_request(request)?;

    let ffmpeg_path = resolve_ffmpeg()?;
    let ffprobe_path = resolve_ffprobe()?;
    let duration_seconds = read_video_duration_seconds(&request.input_file)?;
    Ok(build_ffmpeg_args_for_request(
        request,
        duration_seconds,
        ffmpeg_path,
        ffprobe_path,
    ))
}

pub fn build_ffmpeg_args_for_request(
    request: &ShortGenerationRequest,
    duration_seconds: f64,
    ffmpeg_path: std::path::PathBuf,
    ffprobe_path: std::path::PathBuf,
) -> FfmpegArgs {
    let video_filter_value = video_filter(&request.options, duration_seconds);
    let audio_filter_value = audio_filter(request.options.speed);

    let mut args: Vec<String> = vec![
        "-y".to_owned(),
        "-i".to_owned(),
        path_for_ffmpeg(&request.input_file),
        "-vf".to_owned(),
        video_filter_value.clone(),
    ];

    if let Some(af) = &audio_filter_value {
        args.push("-af".to_owned());
        args.push(af.clone());
    }

    args.extend([
        "-c:v".to_owned(),
        "libx264".to_owned(),
        "-crf".to_owned(),
        request.options.crf.to_string(),
        "-preset".to_owned(),
        request.options.preset.as_str().to_owned(),
    ]);

    if request
        .options
        .effects
        .iter()
        .any(|effect| effect.as_str() == "smooth")
    {
        args.push("-r".to_owned());
        args.push("120".to_owned());
    }

    args.extend([
        "-c:a".to_owned(),
        "aac".to_owned(),
        "-b:a".to_owned(),
        "192k".to_owned(),
        "-movflags".to_owned(),
        "+faststart".to_owned(),
        path_for_ffmpeg(&request.output_file),
    ]);

    FfmpegArgs {
        ffmpeg_path,
        ffprobe_path,
        args,
        video_filter: video_filter_value,
        audio_filter: audio_filter_value,
    }
}

pub fn run_generation(prepared: &FfmpegArgs) -> Result<ShortGenerationResult, VideoShortError> {
    let mut command = Command::new(&prepared.ffmpeg_path);
    command
        .args(prepared.args.iter())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    configure_background_command(&mut command);

    let output = command
        .output()
        .map_err(|e| VideoShortError::FfmpegSpawnFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(VideoShortError::FfmpegFailed {
            exit_code: output.status.code(),
            stderr,
        });
    }

    let output_path = prepared
        .args
        .last()
        .map(std::path::PathBuf::from)
        .unwrap_or_default();

    Ok(ShortGenerationResult {
        output_file: output_path,
        ffmpeg_exit_code: output.status.code(),
        ffmpeg_stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

pub fn stage_label(stage: GenerationStage) -> &'static str {
    match stage {
        GenerationStage::Preparing => "Preparing short",
        GenerationStage::Probing => "Probing source video",
        GenerationStage::BuildingFilters => "Building video filters",
        GenerationStage::Generating => "Generating short",
        GenerationStage::Finalizing => "Finalizing output",
        GenerationStage::Completed => "Completed",
        GenerationStage::Failed => "Failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        CropPosition, Effect, Preset, ShortGenerationOptions, ShortGenerationRequest,
    };
    use std::path::PathBuf;

    #[test]
    fn ffmpeg_args_include_expected_script_flags() {
        let request = ShortGenerationRequest {
            input_file: PathBuf::from("input.mp4"),
            output_file: PathBuf::from("output.mp4"),
            options: ShortGenerationOptions {
                effects: vec![Effect::Smooth, Effect::Enhanced],
                crop_position: CropPosition::Center,
                add_fade: false,
                speed: 1.0,
                crf: 21,
                preset: Preset::Fast,
            },
        };

        let built = build_ffmpeg_args_for_request(
            &request,
            3.0,
            PathBuf::from("ffmpeg"),
            PathBuf::from("ffprobe"),
        );

        assert!(built.args.iter().any(|v| v == "-movflags"));
        assert!(built.args.iter().any(|v| v == "+faststart"));
        assert!(built.args.iter().any(|v| v == "libx264"));
        assert!(built.args.iter().any(|v| v == "aac"));
        assert!(built.args.iter().any(|v| v == "120"));
        assert!(built.args.iter().any(|v| v == "21"));
    }
}
