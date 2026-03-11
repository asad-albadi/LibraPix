use crate::{ProjectionMedia, compare_optional_i64_desc};
use chrono::{Datelike, Local, TimeZone};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineGranularity {
    Day,
    Month,
    Year,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TimelineKey {
    year: i32,
    month: Option<u32>,
    day: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineItem {
    pub media_id: i64,
    pub absolute_path: String,
    pub media_kind: String,
    pub modified_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineDateParts {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineBucket {
    pub label: String,
    pub date: TimelineDateParts,
    pub item_count: usize,
    pub items: Vec<TimelineItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAnchor {
    pub group_index: usize,
    pub label: String,
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub item_count: usize,
    pub normalized_position: f32,
}

pub fn project_timeline(
    media: &[ProjectionMedia],
    granularity: TimelineGranularity,
) -> Vec<TimelineBucket> {
    project_timeline_with_timezone(media, granularity, &Local)
}

fn project_timeline_with_timezone<Tz: TimeZone>(
    media: &[ProjectionMedia],
    granularity: TimelineGranularity,
    time_zone: &Tz,
) -> Vec<TimelineBucket> {
    let mut grouped: BTreeMap<TimelineKey, Vec<&ProjectionMedia>> = BTreeMap::new();
    let mut unknown_items: Vec<&ProjectionMedia> = Vec::new();

    for item in media {
        if let Some(key) = persisted_timeline_key(item, granularity) {
            grouped.entry(key).or_default().push(item);
            continue;
        }

        let Some(timestamp) = item.modified_unix_seconds else {
            unknown_items.push(item);
            continue;
        };
        let Some(datetime) = time_zone.timestamp_opt(timestamp, 0).single() else {
            unknown_items.push(item);
            continue;
        };
        let key = match granularity {
            TimelineGranularity::Day => TimelineKey {
                year: datetime.year(),
                month: Some(datetime.month()),
                day: Some(datetime.day()),
            },
            TimelineGranularity::Month => TimelineKey {
                year: datetime.year(),
                month: Some(datetime.month()),
                day: None,
            },
            TimelineGranularity::Year => TimelineKey {
                year: datetime.year(),
                month: None,
                day: None,
            },
        };
        grouped.entry(key).or_default().push(item);
    }

    let mut buckets = grouped
        .into_iter()
        .rev()
        .map(|(key, mut items)| {
            items.sort_by(|a, b| {
                compare_optional_i64_desc(a.modified_unix_seconds, b.modified_unix_seconds)
                    .then_with(|| a.absolute_path.cmp(&b.absolute_path))
            });
            let rows = items
                .into_iter()
                .map(|item| TimelineItem {
                    media_id: item.media_id,
                    absolute_path: item.absolute_path.clone(),
                    media_kind: item.media_kind.clone(),
                    modified_unix_seconds: item.modified_unix_seconds,
                })
                .collect::<Vec<_>>();
            TimelineBucket {
                label: format_key(&key),
                date: TimelineDateParts {
                    year: Some(key.year),
                    month: key.month,
                    day: key.day,
                },
                item_count: rows.len(),
                items: rows,
            }
        })
        .collect::<Vec<_>>();

    if !unknown_items.is_empty() {
        let rows = unknown_items
            .into_iter()
            .map(|item| TimelineItem {
                media_id: item.media_id,
                absolute_path: item.absolute_path.clone(),
                media_kind: item.media_kind.clone(),
                modified_unix_seconds: item.modified_unix_seconds,
            })
            .collect::<Vec<_>>();
        buckets.push(TimelineBucket {
            label: "unknown".to_owned(),
            date: TimelineDateParts {
                year: None,
                month: None,
                day: None,
            },
            item_count: rows.len(),
            items: rows,
        });
    }

    buckets
}

fn persisted_timeline_key(
    item: &ProjectionMedia,
    granularity: TimelineGranularity,
) -> Option<TimelineKey> {
    let raw = match granularity {
        TimelineGranularity::Day => item.timeline_day_key.as_deref(),
        TimelineGranularity::Month => item.timeline_month_key.as_deref(),
        TimelineGranularity::Year => item.timeline_year_key.as_deref(),
    }?;
    parse_persisted_timeline_key(raw, granularity)
}

fn parse_persisted_timeline_key(
    value: &str,
    granularity: TimelineGranularity,
) -> Option<TimelineKey> {
    let mut parts = value.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;

    match granularity {
        TimelineGranularity::Day => {
            let month = parts.next()?.parse::<u32>().ok()?;
            let day = parts.next()?.parse::<u32>().ok()?;
            Some(TimelineKey {
                year,
                month: Some(month),
                day: Some(day),
            })
        }
        TimelineGranularity::Month => {
            let month = parts.next()?.parse::<u32>().ok()?;
            Some(TimelineKey {
                year,
                month: Some(month),
                day: None,
            })
        }
        TimelineGranularity::Year => Some(TimelineKey {
            year,
            month: None,
            day: None,
        }),
    }
}

pub fn build_timeline_anchors(buckets: &[TimelineBucket]) -> Vec<TimelineAnchor> {
    if buckets.is_empty() {
        return Vec::new();
    }

    let has_multiple = buckets.len() > 1;
    let weights = buckets
        .iter()
        .map(|bucket| bucket.item_count.max(1) as f32 + 1.0)
        .collect::<Vec<_>>();
    let max_prefix_weight = weights
        .iter()
        .take(weights.len().saturating_sub(1))
        .sum::<f32>();

    let mut prefix_weight = 0.0f32;
    buckets
        .iter()
        .enumerate()
        .map(|(group_index, bucket)| {
            let normalized_position = if has_multiple && max_prefix_weight > 0.0 {
                (prefix_weight / max_prefix_weight).clamp(0.0, 1.0)
            } else {
                0.0
            };

            prefix_weight += weights[group_index];

            TimelineAnchor {
                group_index,
                label: bucket.label.clone(),
                year: bucket.date.year,
                month: bucket.date.month,
                day: bucket.date.day,
                item_count: bucket.item_count,
                normalized_position,
            }
        })
        .collect()
}

fn format_key(key: &TimelineKey) -> String {
    match (key.month, key.day) {
        (Some(month), Some(day)) => format!("{:04}-{:02}-{:02}", key.year, month, day),
        (Some(month), None) => format!("{:04}-{:02}", key.year, month),
        (None, None) => format!("{:04}", key.year),
        (None, Some(_)) => format!("{:04}", key.year),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    fn sample() -> Vec<ProjectionMedia> {
        vec![
            ProjectionMedia {
                media_id: 1,
                absolute_path: "/a/one.png".to_owned(),
                media_kind: "image".to_owned(),
                modified_unix_seconds: Some(1_700_000_000),
                tags: vec![],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
            ProjectionMedia {
                media_id: 2,
                absolute_path: "/a/two.mp4".to_owned(),
                media_kind: "video".to_owned(),
                modified_unix_seconds: Some(1_700_000_100),
                tags: vec![],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
            ProjectionMedia {
                media_id: 3,
                absolute_path: "/a/unknown.png".to_owned(),
                media_kind: "image".to_owned(),
                modified_unix_seconds: None,
                tags: vec![],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
        ]
    }

    #[test]
    fn groups_by_day_and_adds_unknown_bucket() {
        let buckets = project_timeline(&sample(), TimelineGranularity::Day);
        assert!(buckets.iter().any(|bucket| bucket.label == "unknown"));
        let known_total = buckets
            .iter()
            .filter(|bucket| bucket.label != "unknown")
            .map(|bucket| bucket.item_count)
            .sum::<usize>();
        assert_eq!(known_total, 2);
    }

    #[test]
    fn groups_by_month() {
        let buckets = project_timeline(&sample(), TimelineGranularity::Month);
        assert!(
            buckets
                .iter()
                .any(|bucket| bucket.label.len() == 7 || bucket.label == "unknown")
        );
    }

    #[test]
    fn builds_monotonic_anchors_with_date_parts() {
        let buckets = project_timeline(&sample(), TimelineGranularity::Day);
        let anchors = build_timeline_anchors(&buckets);

        assert_eq!(anchors.len(), buckets.len());
        assert!(
            anchors
                .windows(2)
                .all(|w| { w[0].normalized_position <= w[1].normalized_position })
        );
        assert_eq!(anchors.first().map(|a| a.normalized_position), Some(0.0));
        assert_eq!(anchors.last().map(|a| a.normalized_position), Some(1.0));

        let known_anchor = anchors.iter().find(|a| a.label != "unknown").unwrap();
        assert!(known_anchor.year.is_some());
        assert!(known_anchor.month.is_some());
        assert!(known_anchor.day.is_some());
    }

    #[test]
    fn unknown_bucket_anchor_has_no_date_parts() {
        let buckets = project_timeline(&sample(), TimelineGranularity::Day);
        let anchors = build_timeline_anchors(&buckets);
        let unknown = anchors.iter().find(|a| a.label == "unknown").unwrap();

        assert_eq!(unknown.year, None);
        assert_eq!(unknown.month, None);
        assert_eq!(unknown.day, None);
    }

    #[test]
    fn anchor_positions_reflect_bucket_sizes() {
        let buckets = vec![
            TimelineBucket {
                label: "2026-03-01".to_owned(),
                date: TimelineDateParts {
                    year: Some(2026),
                    month: Some(3),
                    day: Some(1),
                },
                item_count: 1,
                items: Vec::new(),
            },
            TimelineBucket {
                label: "2026-02-01".to_owned(),
                date: TimelineDateParts {
                    year: Some(2026),
                    month: Some(2),
                    day: Some(1),
                },
                item_count: 120,
                items: Vec::new(),
            },
            TimelineBucket {
                label: "2025-12-01".to_owned(),
                date: TimelineDateParts {
                    year: Some(2025),
                    month: Some(12),
                    day: Some(1),
                },
                item_count: 1,
                items: Vec::new(),
            },
        ];

        let anchors = build_timeline_anchors(&buckets);
        assert_eq!(anchors.len(), 3);
        assert_eq!(anchors[0].normalized_position, 0.0);
        assert_eq!(anchors[2].normalized_position, 1.0);
        assert!(anchors[1].normalized_position < 0.1);
    }

    #[test]
    fn groups_using_localized_day_boundaries() {
        let media = vec![
            ProjectionMedia {
                media_id: 1,
                absolute_path: "/a/night.png".to_owned(),
                media_kind: "image".to_owned(),
                modified_unix_seconds: Some(84_600), // 1970-01-01 23:30:00 UTC
                tags: vec![],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
            ProjectionMedia {
                media_id: 2,
                absolute_path: "/a/morning.png".to_owned(),
                media_kind: "image".to_owned(),
                modified_unix_seconds: Some(88_200), // 1970-01-02 00:30:00 UTC
                tags: vec![],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
        ];

        let utc = FixedOffset::east_opt(0).expect("valid offset");
        let local_plus_two = FixedOffset::east_opt(2 * 3_600).expect("valid offset");

        let utc_buckets = project_timeline_with_timezone(&media, TimelineGranularity::Day, &utc);
        let local_buckets =
            project_timeline_with_timezone(&media, TimelineGranularity::Day, &local_plus_two);

        assert_eq!(utc_buckets.len(), 2);
        assert_eq!(local_buckets.len(), 1);
        assert_eq!(local_buckets[0].label, "1970-01-02");
    }

    #[test]
    fn uses_persisted_timeline_keys_when_available() {
        let media = vec![ProjectionMedia {
            media_id: 1,
            absolute_path: "/a/one.png".to_owned(),
            media_kind: "image".to_owned(),
            modified_unix_seconds: Some(1_700_000_000),
            tags: vec![],
            timeline_day_key: Some("2026-03-10".to_owned()),
            timeline_month_key: Some("2026-03".to_owned()),
            timeline_year_key: Some("2026".to_owned()),
        }];

        let buckets = project_timeline(&media, TimelineGranularity::Day);
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].label, "2026-03-10");
        assert_eq!(buckets[0].date.year, Some(2026));
        assert_eq!(buckets[0].date.month, Some(3));
        assert_eq!(buckets[0].date.day, Some(10));
    }
}
