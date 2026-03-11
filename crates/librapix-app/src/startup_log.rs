use chrono::Local;
use librapix_config::default_paths;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const LOG_FILE_PREFIX: &str = "librapix-startup";
const SLOW_STEP_THRESHOLD: Duration = Duration::from_millis(150);

struct StartupLogger {
    path: PathBuf,
    file: Mutex<File>,
    launched_at: Instant,
}

static LOGGER: OnceLock<StartupLogger> = OnceLock::new();

pub fn init_process_logging() -> Option<PathBuf> {
    if let Some(path) = active_log_path() {
        return Some(path);
    }

    let (file, path) = create_log_file()?;
    let logger = StartupLogger {
        path: path.clone(),
        file: Mutex::new(file),
        launched_at: Instant::now(),
    };
    let _ = LOGGER.set(logger);
    log_info("logging.initialized", &format!("path={}", path.display()));
    eprintln!("Librapix log: {}", path.display());
    Some(path)
}

pub fn active_log_path() -> Option<PathBuf> {
    LOGGER.get().map(|logger| logger.path.clone())
}

pub fn elapsed_since_launch() -> Option<Duration> {
    LOGGER.get().map(|logger| logger.launched_at.elapsed())
}

pub fn log_info(event: &str, detail: &str) {
    log("INFO", event, detail);
}

pub fn log_warn(event: &str, detail: &str) {
    log("WARN", event, detail);
}

pub fn log_error(event: &str, detail: &str) {
    log("ERROR", event, detail);
}

pub fn log_duration(event: &str, duration: Duration, detail: &str) {
    let detail = if detail.trim().is_empty() {
        format!("elapsed_ms={}", duration.as_millis())
    } else {
        format!("{detail} elapsed_ms={}", duration.as_millis())
    };
    if duration >= SLOW_STEP_THRESHOLD {
        log_warn(event, &detail);
    } else {
        log_info(event, &detail);
    }
}

fn log(level: &str, event: &str, detail: &str) {
    let Some(logger) = LOGGER.get() else {
        return;
    };
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f %:z");
    let since_launch_ms = logger.launched_at.elapsed().as_millis();
    let mut line = format!("{timestamp} | +{since_launch_ms}ms | {level} | {event}");
    if !detail.trim().is_empty() {
        line.push_str(" | ");
        line.push_str(detail);
    }
    line.push('\n');

    if let Ok(mut file) = logger.file.lock() {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

fn create_log_file() -> Option<(File, PathBuf)> {
    for directory in candidate_log_dirs() {
        if let Some(file) = create_log_file_in_dir(&directory) {
            return Some(file);
        }
    }
    None
}

fn candidate_log_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("LIBRAPIX_LOG_DIR") {
        push_candidate(&mut candidates, PathBuf::from(path));
    }

    if let Ok(current_dir) = env::current_dir()
        && is_dev_or_portable_dir(&current_dir)
    {
        push_candidate(&mut candidates, current_dir.join("logs"));
    }

    if let Ok(current_exe) = env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        push_candidate(&mut candidates, parent.join("logs"));
    }

    if let Ok(paths) = default_paths() {
        push_candidate(&mut candidates, paths.logs_dir);
    }

    candidates
}

fn push_candidate(candidates: &mut Vec<PathBuf>, candidate: PathBuf) {
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn is_dev_or_portable_dir(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
        || path.join("librapix.db").exists()
        || path.join("config.toml").exists()
}

fn create_log_file_in_dir(directory: &Path) -> Option<(File, PathBuf)> {
    fs::create_dir_all(directory).ok()?;

    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let pid = std::process::id();
    for index in 0..100u32 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("-{index}")
        };
        let path = directory.join(format!("{LOG_FILE_PREFIX}-{timestamp}-{pid}{suffix}.log"));
        match OpenOptions::new().create_new(true).append(true).open(&path) {
            Ok(file) => return Some((file, path)),
            Err(_) => continue,
        }
    }

    None
}
