use crate::{ProjectionMedia, compare_optional_i64_desc};
use chrono::{DateTime, Datelike, Utc};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelineBucket {
    pub label: String,
    pub item_count: usize,
    pub items: Vec<TimelineItem>,
}

pub fn project_timeline(
    media: &[ProjectionMedia],
    granularity: TimelineGranularity,
) -> Vec<TimelineBucket> {
    let mut grouped: BTreeMap<TimelineKey, Vec<&ProjectionMedia>> = BTreeMap::new();
    let mut unknown_items: Vec<&ProjectionMedia> = Vec::new();

    for item in media {
        let Some(timestamp) = item.modified_unix_seconds else {
            unknown_items.push(item);
            continue;
        };
        let Some(datetime) = DateTime::<Utc>::from_timestamp(timestamp, 0) else {
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
            item_count: rows.len(),
            items: rows,
        });
    }

    buckets
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

    fn sample() -> Vec<ProjectionMedia> {
        vec![
            ProjectionMedia {
                media_id: 1,
                absolute_path: "/a/one.png".to_owned(),
                media_kind: "image".to_owned(),
                modified_unix_seconds: Some(1_700_000_000),
                tags: vec![],
            },
            ProjectionMedia {
                media_id: 2,
                absolute_path: "/a/two.mp4".to_owned(),
                media_kind: "video".to_owned(),
                modified_unix_seconds: Some(1_700_000_100),
                tags: vec![],
            },
            ProjectionMedia {
                media_id: 3,
                absolute_path: "/a/unknown.png".to_owned(),
                media_kind: "image".to_owned(),
                modified_unix_seconds: None,
                tags: vec![],
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
}
