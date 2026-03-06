use chrono::{DateTime, Local, Utc};

pub fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let size = bytes as f64;
    if size < KB {
        format!("{bytes} B")
    } else if size < MB {
        format!("{:.1} KB", size / KB)
    } else if size < GB {
        format!("{:.1} MB", size / MB)
    } else {
        format!("{:.2} GB", size / GB)
    }
}

pub fn format_timestamp(unix_seconds: Option<i64>) -> String {
    let Some(ts) = unix_seconds else {
        return "\u{2014}".to_owned();
    };
    DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|dt| {
            let local = dt.with_timezone(&Local);
            local.format("%b %d, %Y  %I:%M %p").to_string()
        })
        .unwrap_or_else(|| "\u{2014}".to_owned())
}

pub fn format_dimensions(width: Option<u32>, height: Option<u32>) -> String {
    match (width, height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => format!("{w} \u{00D7} {h}"),
        _ => "\u{2014}".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_file_sizes_correctly() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(843_776), "824.0 KB");
        assert_eq!(format_file_size(14_888_960), "14.2 MB");
        assert_eq!(format_file_size(1_395_864_371), "1.30 GB");
    }

    #[test]
    fn formats_timestamps_as_nonempty() {
        let result = format_timestamp(Some(1_700_000_000));
        assert!(!result.is_empty());
        assert!(!result.contains("1700000000"));
    }

    #[test]
    fn formats_none_timestamp_as_dash() {
        assert_eq!(format_timestamp(None), "\u{2014}");
    }

    #[test]
    fn formats_dimensions() {
        assert_eq!(
            format_dimensions(Some(1920), Some(1080)),
            "1920 \u{00D7} 1080"
        );
        assert_eq!(format_dimensions(None, None), "\u{2014}");
        assert_eq!(format_dimensions(Some(0), Some(0)), "\u{2014}");
    }
}
