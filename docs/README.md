# Documentation Index

This repository is documentation-driven. Architecture and operational decisions must be documented in this directory as part of implementation.

## Core

- `docs/REPOSITORY_RULES.md`: repository-wide engineering rules.
- `docs/DEPENDENCIES.md`: major dependency decisions and official documentation references.
- `docs/TROUBLESHOOTING.md`: recurring issues, causes, and resolutions.

## Architecture

- `docs/architecture/overview.md`
- `docs/architecture/layers.md`
- `docs/architecture/message-flow.md`
- `docs/architecture/indexing.md`
- `docs/architecture/search.md`
- `docs/architecture/projections.md`
- `docs/architecture/thumbnails.md`
- `docs/architecture/actions.md`
- `docs/architecture/ui.md`
- `docs/architecture/storage.md`
- `docs/architecture/metadata.md`
- `docs/architecture/config.md`
- `docs/architecture/i18n.md`
- `docs/architecture/ignore-rules.md`
- `docs/architecture/non-destructive-guarantee.md`
- `docs/architecture/media-ui.md`

## Roadmap

- `docs/roadmap/mvp.md`
- `docs/roadmap/future.md`
- `docs/roadmap/bootstrap-checklist.md`
- `docs/roadmap/config-storage-checklist.md`
- `docs/roadmap/library-indexing-foundation-checklist.md`
- `docs/roadmap/metadata-incremental-search-checklist.md`
- `docs/roadmap/search-projections-checklist.md`
- `docs/roadmap/mvp-remaining-checklist.md`
- `docs/roadmap/ui-ux-redesign-checklist.md`
- `docs/roadmap/interaction-milestone-checklist.md`
- `docs/roadmap/quality-milestone-checklist.md`
- `docs/roadmap/media-view-milestone-checklist.md`
- `docs/roadmap/selection-dims-autotag-checklist.md`
- `docs/roadmap/startup-aggregation-ingestion-checklist.md`
- `docs/roadmap/all-videos-aggregation-thumbnails-checklist.md`
- `docs/roadmap/browse-model-correctness-checklist.md`
- `docs/roadmap/timeline-scrubber-checklist.md`
- `docs/roadmap/media-navigation-product-updates-checklist.md`
- `docs/roadmap/timeline-followup-correctness-checklist.md`
- `docs/roadmap/windows-copy-dialog-background-checklist.md`
- `docs/roadmap/windows-copy-timeline-scrubber-correctness-checklist.md`

## Packaging

- `packaging/windows/README.md`: Windows publisher/signing/distribution baseline.
- `packaging/windows/msix/AppxManifest.xml`: MSIX identity/publisher template (`CN=Asad`).
- `packaging/windows/scripts/`: PowerShell scripts for dev cert creation, signing, and MSIX packaging.

## ADRs

- `docs/adrs/0001-workspace-boundaries.md`
- `docs/adrs/0002-config-path-policy.md`
- `docs/adrs/0003-sqlite-storage-baseline.md`
- `docs/adrs/0004-source-root-lifecycle-policy.md`
- `docs/adrs/0005-indexing-ignore-baseline.md`
- `docs/adrs/0006-metadata-incremental-readmodels.md`
- `docs/adrs/0007-replaceable-search-baseline.md`
- `docs/adrs/0008-timeline-gallery-projections.md`
- `docs/adrs/0009-thumbnail-cache-baseline.md`
- `docs/adrs/0010-media-details-tags-actions-baseline.md`
- `docs/adrs/0011-ui-shell-and-design-tokens.md`
- `docs/adrs/0012-fluent-inspired-design-system.md`
- `docs/adrs/0013-interaction-milestone.md`
- `docs/adrs/0014-quality-filtering-exclusion.md`
- `docs/adrs/0015-media-view-architecture.md`
- `docs/adrs/0016-root-level-auto-tags.md`
- `docs/adrs/0017-async-startup-and-background-work.md`
- `docs/adrs/0018-timeline-scrubber-anchor-navigation.md`
- `docs/adrs/0019-windows-signing-and-publisher-baseline.md`
