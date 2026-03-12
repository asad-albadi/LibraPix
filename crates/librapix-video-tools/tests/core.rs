use librapix_video_tools::models::{
    CropPosition, Effect, Preset, ShortGenerationOptions, ShortGenerationRequest,
};
use librapix_video_tools::paths::default_output_file_path;
use librapix_video_tools::runner::build_ffmpeg_args_for_request;
use librapix_video_tools::validate::validate_semantics;
use std::path::{Path, PathBuf};

fn temp_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("librapix-video-tools-tests");
    let _ = std::fs::create_dir_all(&dir);
    let file = dir.join(name);
    let _ = std::fs::write(&file, b"stub");
    file
}

fn request_with_options(options: ShortGenerationOptions) -> ShortGenerationRequest {
    let input = temp_file("input.mp4");
    let out_dir = std::env::temp_dir().join("librapix-video-tools-tests-out");
    let _ = std::fs::create_dir_all(&out_dir);

    ShortGenerationRequest {
        input_file: input,
        output_file: out_dir.join("output-short.mp4"),
        options,
    }
}

#[test]
fn clean_effect_is_exclusive() {
    let request = request_with_options(ShortGenerationOptions {
        effects: vec![Effect::Clean, Effect::Enhanced],
        crop_position: CropPosition::Center,
        add_fade: false,
        speed: 1.0,
        crf: 18,
        preset: Preset::Medium,
    });

    let error = validate_semantics(&request).expect_err("clean + enhanced should fail");
    assert!(error.to_string().contains("cannot be combined"));
}

#[test]
fn speed_must_be_positive() {
    let request = request_with_options(ShortGenerationOptions {
        effects: vec![Effect::Enhanced],
        crop_position: CropPosition::Center,
        add_fade: false,
        speed: 0.0,
        crf: 18,
        preset: Preset::Medium,
    });

    let error = validate_semantics(&request).expect_err("speed zero should fail");
    assert!(error.to_string().contains("greater than 0"));
}

#[test]
fn default_output_path_uses_short_suffix() {
    let input = PathBuf::from("C:/captures/highlight.mp4");
    let out = default_output_file_path(&input, Some(Path::new("C:/Videos/LibraPix-Shorts")));
    assert_eq!(
        out,
        PathBuf::from("C:/Videos/LibraPix-Shorts/highlight-short.mp4")
    );
}

#[test]
fn ffmpeg_arguments_include_script_core_options() {
    let request = request_with_options(ShortGenerationOptions {
        effects: vec![Effect::Enhanced, Effect::Smooth],
        crop_position: CropPosition::Right,
        add_fade: true,
        speed: 1.5,
        crf: 20,
        preset: Preset::Slow,
    });

    let built = build_ffmpeg_args_for_request(
        &request,
        24.0,
        PathBuf::from("ffmpeg"),
        PathBuf::from("ffprobe"),
    );

    assert!(built.args.iter().any(|value| value == "-vf"));
    assert!(
        built
            .args
            .iter()
            .any(|value| value.contains("format=yuv420p"))
    );
    assert!(built.args.iter().any(|value| value == "-af"));
    assert!(built.args.iter().any(|value| value.contains("atempo=")));
    assert!(built.args.iter().any(|value| value == "-c:v"));
    assert!(built.args.iter().any(|value| value == "libx264"));
    assert!(built.args.iter().any(|value| value == "-c:a"));
    assert!(built.args.iter().any(|value| value == "aac"));
    assert!(built.args.iter().any(|value| value == "+faststart"));
    assert!(built.args.iter().any(|value| value == "120"));
}
