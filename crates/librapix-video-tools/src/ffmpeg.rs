use crate::error::VideoShortError;
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn locate_binary(executable: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for entry in env::split_paths(&path_var) {
        let candidate = entry.join(executable);
        if is_executable(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            if Path::new(executable).extension().is_none() {
                let with_exe = entry.join(format!("{executable}.exe"));
                if is_executable(&with_exe) {
                    return Some(with_exe);
                }
            }
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

pub fn resolve_ffmpeg() -> Result<PathBuf, VideoShortError> {
    locate_binary("ffmpeg").ok_or(VideoShortError::FfmpegNotFound)
}

pub fn resolve_ffprobe() -> Result<PathBuf, VideoShortError> {
    locate_binary("ffprobe").ok_or(VideoShortError::FfprobeNotFound)
}

pub fn path_for_ffmpeg(path: &Path) -> String {
    #[cfg(windows)]
    {
        path.as_os_str()
            .to_string_lossy()
            .replace('\\', "/")
            .to_string()
    }
    #[cfg(not(windows))]
    {
        path.as_os_str().to_string_lossy().to_string()
    }
}

pub fn command_line(executable: &Path, args: &[String]) -> String {
    let mut parts = vec![shell_escape(executable.as_os_str())];
    parts.extend(args.iter().map(|v| shell_escape(OsStr::new(v))));
    parts.join(" ")
}

pub fn configure_background_command(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn shell_escape(value: &OsStr) -> String {
    let raw = value.to_string_lossy();
    if raw.chars().any(|c| c.is_whitespace()) {
        format!("\"{raw}\"")
    } else {
        raw.to_string()
    }
}
