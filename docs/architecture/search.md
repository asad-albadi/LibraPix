# Search Architecture

Search is implemented as a replaceable subsystem.

## Boundary

- Core search logic lives in `librapix-search`.
- Storage (`librapix-storage`) provides read-model rows.
- App orchestration maps read-model rows into search documents and executes a search strategy.
- UI integrates search in the app header and renders resulting media cards; it does not implement ranking.

## Baseline contracts

- `SearchDocument`: normalized searchable fields (`path`, `filename`, `media_kind`, `tags`)
- `SearchQuery`: text + limit
- `SearchHit`: `media_id` + relevance score
- `SearchStrategy` trait: replaceable search behavior
- `FuzzySearchStrategy`: default baseline strategy

## Baseline ranking policy

- Query is normalized to lowercase terms.
- All terms must match (exact, partial, or fuzzy) for a document to be included.
- Term scoring priority:
  1. exact match
  2. partial/contains match
  3. fuzzy similarity (`strsim::normalized_levenshtein`) above threshold
- Final score is average term score; results sorted by descending score.

## Current scope

- Supports path/filename, tags, and media-kind terms.
- Search results respect active type and extension filters applied at the app layer.
- Designed to be replaced later with richer ranking or index-backed search.
