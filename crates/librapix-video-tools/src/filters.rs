use crate::models::{CropPosition, Effect, ShortGenerationOptions};

pub fn crop_x_expression(position: CropPosition) -> &'static str {
    match position {
        CropPosition::Left => "0",
        CropPosition::Right => "(iw-1080)",
        CropPosition::Center => "(iw-1080)/2",
    }
}

pub fn video_filter(options: &ShortGenerationOptions, duration_seconds: f64) -> String {
    let mut filters: Vec<String> = vec![
        "scale=-1:1920".to_owned(),
        format!(
            "crop=1080:1920:{}:(ih-1920)",
            crop_x_expression(options.crop_position)
        ),
    ];

    if options.effects.contains(&Effect::Smooth) {
        filters.push(
            "minterpolate=fps=120:mi_mode=mci:mc_mode=aobmc:me_mode=bidir:vsbmc=1".to_owned(),
        );
    }

    if !options.effects.contains(&Effect::Clean) {
        if options.effects.contains(&Effect::Enhanced) {
            add_unique(
                &mut filters,
                "eq=brightness=0.03:contrast=1.08:saturation=1.08:gamma=1.06",
            );
            add_unique(&mut filters, "unsharp=5:5:0.4:5:5:0.0");
        }

        if options.effects.contains(&Effect::Cinematic) {
            add_unique(
                &mut filters,
                "eq=brightness=0.03:contrast=1.08:saturation=1.08:gamma=1.06",
            );
            add_unique(&mut filters, "hqdn3d=1.2:1.2:6:6");
            add_unique(&mut filters, "unsharp=5:5:0.4:5:5:0.0");
        }

        if options.effects.contains(&Effect::Night) {
            add_unique(
                &mut filters,
                "eq=brightness=0.05:contrast=1.10:saturation=1.10:gamma=1.12",
            );
            add_unique(&mut filters, "hqdn3d=1.4:1.4:6:6");
            add_unique(&mut filters, "unsharp=5:5:0.35:5:5:0.0");
        }

        if options.effects.contains(&Effect::Scenic) {
            add_unique(
                &mut filters,
                "eq=brightness=0.03:contrast=1.08:saturation=1.12:gamma=1.08",
            );
            add_unique(&mut filters, "hqdn3d=1.0:1.0:4:4");
            add_unique(&mut filters, "gradfun=1.2");
            add_unique(&mut filters, "unsharp=5:5:0.35:5:5:0.0");
        }
    }

    if (options.speed - 1.0).abs() > f64::EPSILON {
        let speed_value = 1.0 / options.speed;
        filters.push(format!("setpts={}*PTS", format_decimal(speed_value, 6)));
    }

    if options.add_fade {
        let fade_in = 0.35;
        let fade_out_duration = 0.40;
        let adjusted_duration = duration_seconds / options.speed;
        let fade_out_start = (adjusted_duration - fade_out_duration).max(0.0);

        filters.push(format!("fade=t=in:st=0:d={}", format_decimal(fade_in, 3)));
        filters.push(format!(
            "fade=t=out:st={}:d={}",
            format_decimal(fade_out_start, 3),
            format_decimal(fade_out_duration, 3)
        ));
    }

    filters.push("format=yuv420p".to_owned());

    filters.join(",")
}

pub fn audio_filter(speed: f64) -> Option<String> {
    if (speed - 1.0).abs() < f64::EPSILON {
        return None;
    }

    let mut remaining = speed;
    let mut parts: Vec<String> = Vec::new();

    while remaining > 2.0 {
        parts.push("atempo=2.0".to_owned());
        remaining /= 2.0;
    }

    while remaining < 0.5 {
        parts.push("atempo=0.5".to_owned());
        remaining /= 0.5;
    }

    parts.push(format!("atempo={}", format_decimal(remaining, 3)));
    Some(parts.join(","))
}

fn add_unique(filters: &mut Vec<String>, value: &str) {
    if !filters.iter().any(|existing| existing == value) {
        filters.push(value.to_owned());
    }
}

fn format_decimal(value: f64, max_digits: usize) -> String {
    let mut text = format!("{value:.max_digits$}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CropPosition, Effect, Preset, ShortGenerationOptions};

    #[test]
    fn crop_mapping_matches_script() {
        assert_eq!(crop_x_expression(CropPosition::Left), "0");
        assert_eq!(crop_x_expression(CropPosition::Right), "(iw-1080)");
        assert_eq!(crop_x_expression(CropPosition::Center), "(iw-1080)/2");
    }

    #[test]
    fn audio_filter_builds_atempo_chain_for_high_speed() {
        let filter = audio_filter(4.0).expect("filter");
        assert_eq!(filter, "atempo=2.0,atempo=2");
    }

    #[test]
    fn audio_filter_builds_atempo_chain_for_low_speed() {
        let filter = audio_filter(0.25).expect("filter");
        assert_eq!(filter, "atempo=0.5,atempo=0.5");
    }

    #[test]
    fn video_filter_includes_fade_and_format() {
        let options = ShortGenerationOptions {
            effects: vec![Effect::Enhanced],
            crop_position: CropPosition::Center,
            add_fade: true,
            speed: 2.0,
            crf: 18,
            preset: Preset::Medium,
        };

        let filter = video_filter(&options, 10.0);
        assert!(filter.contains("setpts=0.5*PTS"));
        assert!(filter.contains("fade=t=in:st=0:d=0.35"));
        assert!(filter.contains("fade=t=out:st=4.6:d=0.4"));
        assert!(filter.ends_with("format=yuv420p"));
    }
}
