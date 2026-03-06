use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionMedia {
    pub media_id: i64,
    pub absolute_path: String,
    pub media_kind: String,
    pub modified_unix_seconds: Option<i64>,
    pub tags: Vec<String>,
}

pub mod gallery;
pub mod timeline;

fn compare_optional_i64_desc(left: Option<i64>, right: Option<i64>) -> Ordering {
    match (left, right) {
        (Some(a), Some(b)) => b.cmp(&a),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}
