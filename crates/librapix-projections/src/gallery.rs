use crate::{ProjectionMedia, compare_optional_i64_desc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GallerySort {
    ModifiedDesc,
    PathAsc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalleryQuery {
    pub media_kind: Option<String>,
    pub extension: Option<String>,
    pub tag: Option<String>,
    pub sort: GallerySort,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GalleryItem {
    pub media_id: i64,
    pub absolute_path: String,
    pub media_kind: String,
    pub modified_unix_seconds: Option<i64>,
}

pub fn project_gallery(media: &[ProjectionMedia], query: &GalleryQuery) -> Vec<GalleryItem> {
    let mut filtered = media
        .iter()
        .filter(|item| {
            query
                .media_kind
                .as_ref()
                .is_none_or(|kind| item.media_kind.eq_ignore_ascii_case(kind))
        })
        .filter(|item| {
            query.extension.as_ref().is_none_or(|ext| {
                item.absolute_path
                    .rsplit('.')
                    .next()
                    .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            })
        })
        .filter(|item| {
            query
                .tag
                .as_ref()
                .is_none_or(|tag| item.tags.iter().any(|item_tag| item_tag == tag))
        })
        .collect::<Vec<_>>();

    match query.sort {
        GallerySort::ModifiedDesc => filtered.sort_by(|a, b| {
            compare_optional_i64_desc(a.modified_unix_seconds, b.modified_unix_seconds)
                .then_with(|| a.absolute_path.cmp(&b.absolute_path))
        }),
        GallerySort::PathAsc => filtered.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path)),
    }

    filtered
        .into_iter()
        .skip(query.offset)
        .take(query.limit)
        .map(|item| GalleryItem {
            media_id: item.media_id,
            absolute_path: item.absolute_path.clone(),
            media_kind: item.media_kind.clone(),
            modified_unix_seconds: item.modified_unix_seconds,
        })
        .collect()
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
                modified_unix_seconds: Some(200),
                tags: vec!["kind:image".to_owned()],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
            ProjectionMedia {
                media_id: 2,
                absolute_path: "/a/two.mp4".to_owned(),
                media_kind: "video".to_owned(),
                modified_unix_seconds: Some(100),
                tags: vec!["kind:video".to_owned()],
                timeline_day_key: None,
                timeline_month_key: None,
                timeline_year_key: None,
            },
        ]
    }

    #[test]
    fn filters_by_kind_and_sorts_desc() {
        let query = GalleryQuery {
            media_kind: Some("image".to_owned()),
            extension: None,
            tag: None,
            sort: GallerySort::ModifiedDesc,
            limit: 10,
            offset: 0,
        };
        let rows = project_gallery(&sample(), &query);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].media_id, 1);
    }

    #[test]
    fn filters_by_extension() {
        let query = GalleryQuery {
            media_kind: None,
            extension: Some("mp4".to_owned()),
            tag: None,
            sort: GallerySort::ModifiedDesc,
            limit: 10,
            offset: 0,
        };
        let rows = project_gallery(&sample(), &query);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].media_id, 2);
    }

    #[test]
    fn filters_by_tag() {
        let query = GalleryQuery {
            media_kind: None,
            extension: None,
            tag: Some("kind:video".to_owned()),
            sort: GallerySort::PathAsc,
            limit: 10,
            offset: 0,
        };
        let rows = project_gallery(&sample(), &query);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].media_id, 2);
    }
}
