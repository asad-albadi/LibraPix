#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionMedia {
    pub media_id: i64,
    pub absolute_path: String,
    pub media_kind: String,
    pub modified_unix_seconds: Option<i64>,
    pub tags: Vec<String>,
}
