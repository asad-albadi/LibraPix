use directories::UserDirs;
use std::path::{Path, PathBuf};

pub fn default_shorts_output_dir() -> Option<PathBuf> {
    let user_dirs = UserDirs::new()?;
    let videos = user_dirs.video_dir()?;
    Some(videos.join("LibraPix-Shorts"))
}

pub fn default_output_file_path(input_file: &Path, default_dir: Option<&Path>) -> PathBuf {
    let stem = input_file
        .file_stem()
        .and_then(|v| v.to_str())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("output");
    let file_name = format!("{stem}-short.mp4");

    let base = default_dir
        .map(PathBuf::from)
        .or_else(default_shorts_output_dir)
        .or_else(|| input_file.parent().map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    base.join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_output_file_uses_given_directory() {
        let input = PathBuf::from("C:/clips/raid.mp4");
        let out = default_output_file_path(&input, Some(Path::new("C:/Videos/LibraPix-Shorts")));
        assert_eq!(
            out,
            PathBuf::from("C:/Videos/LibraPix-Shorts/raid-short.mp4")
        );
    }
}
