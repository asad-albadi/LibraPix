use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchDocument {
    pub media_id: i64,
    pub absolute_path: String,
    pub file_name: String,
    pub media_kind: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchHit {
    pub media_id: i64,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchQuery {
    pub text: String,
    pub limit: usize,
}

pub trait SearchStrategy {
    fn search(&self, documents: &[SearchDocument], query: &SearchQuery) -> Vec<SearchHit>;
}

#[derive(Debug, Clone, Copy)]
pub struct FuzzySearchStrategy {
    pub fuzzy_threshold: f64,
}

impl Default for FuzzySearchStrategy {
    fn default() -> Self {
        Self {
            fuzzy_threshold: 0.70,
        }
    }
}

impl SearchStrategy for FuzzySearchStrategy {
    fn search(&self, documents: &[SearchDocument], query: &SearchQuery) -> Vec<SearchHit> {
        let normalized = normalize(&query.text);
        if normalized.is_empty() {
            return documents
                .iter()
                .take(query.limit)
                .map(|doc| SearchHit {
                    media_id: doc.media_id,
                    score: 0.0,
                })
                .collect();
        }

        let terms = normalized.split_whitespace().collect::<Vec<_>>();
        let mut hits = Vec::new();

        for doc in documents {
            let mut total_score = 0.0;
            let mut matched_all_terms = true;
            for term in &terms {
                let term_score = score_term(doc, term, self.fuzzy_threshold);
                if term_score <= 0.0 {
                    matched_all_terms = false;
                    break;
                }
                total_score += term_score;
            }

            if matched_all_terms {
                hits.push(SearchHit {
                    media_id: doc.media_id,
                    score: total_score / (terms.len() as f64),
                });
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.media_id.cmp(&b.media_id))
        });
        hits.truncate(query.limit);
        hits
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

fn score_term(doc: &SearchDocument, term: &str, fuzzy_threshold: f64) -> f64 {
    let path = normalize(&doc.absolute_path);
    let file_name = normalize(&doc.file_name);
    let media_kind = normalize(&doc.media_kind);
    let tags = doc
        .tags
        .iter()
        .map(|tag| normalize(tag))
        .collect::<Vec<_>>();

    if path == term || file_name == term || media_kind == term || tags.iter().any(|tag| tag == term)
    {
        return 1.0;
    }
    if path.contains(term)
        || file_name.contains(term)
        || media_kind.contains(term)
        || tags.iter().any(|tag| tag.contains(term))
    {
        return 0.86;
    }

    let path_terms = path_tokens(&path);
    let file_terms = path_tokens(&file_name);

    let fuzzy_candidates = std::iter::once(file_name.as_str())
        .chain(std::iter::once(path.as_str()))
        .chain(std::iter::once(media_kind.as_str()))
        .chain(tags.iter().map(String::as_str))
        .chain(path_terms.iter().map(String::as_str))
        .chain(file_terms.iter().map(String::as_str));
    let best = fuzzy_candidates
        .map(|candidate| strsim::normalized_levenshtein(candidate, term))
        .fold(0.0_f64, f64::max);

    if best >= fuzzy_threshold {
        best * 0.75
    } else {
        0.0
    }
}

fn path_tokens(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs() -> Vec<SearchDocument> {
        vec![
            SearchDocument {
                media_id: 1,
                absolute_path: "/shots/game-a/victory_screen.png".to_owned(),
                file_name: "victory_screen.png".to_owned(),
                media_kind: "image".to_owned(),
                tags: vec!["kind:image".to_owned(), "game-a".to_owned()],
            },
            SearchDocument {
                media_id: 2,
                absolute_path: "/videos/game-b/highlight_clip.mp4".to_owned(),
                file_name: "highlight_clip.mp4".to_owned(),
                media_kind: "video".to_owned(),
                tags: vec!["kind:video".to_owned(), "game-b".to_owned()],
            },
        ]
    }

    #[test]
    fn exact_match_finds_document() {
        let strategy = FuzzySearchStrategy::default();
        let query = SearchQuery {
            text: "victory_screen.png".to_owned(),
            limit: 10,
        };
        let hits = strategy.search(&docs(), &query);
        assert_eq!(hits.first().map(|hit| hit.media_id), Some(1));
    }

    #[test]
    fn partial_match_finds_document() {
        let strategy = FuzzySearchStrategy::default();
        let query = SearchQuery {
            text: "highlight".to_owned(),
            limit: 10,
        };
        let hits = strategy.search(&docs(), &query);
        assert_eq!(hits.first().map(|hit| hit.media_id), Some(2));
    }

    #[test]
    fn fuzzy_match_finds_document() {
        let strategy = FuzzySearchStrategy::default();
        let query = SearchQuery {
            text: "victroy".to_owned(),
            limit: 10,
        };
        let hits = strategy.search(&docs(), &query);
        assert_eq!(hits.first().map(|hit| hit.media_id), Some(1));
    }

    #[test]
    fn combined_tag_and_path_terms_must_match() {
        let strategy = FuzzySearchStrategy::default();
        let query = SearchQuery {
            text: "game-a victory".to_owned(),
            limit: 10,
        };
        let hits = strategy.search(&docs(), &query);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].media_id, 1);
    }
}
