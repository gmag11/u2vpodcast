## Why

`02-add-player-chapter-markers` adds chapter tick marks to the persistent player's timeline; the episode card's own bottom progress strip already shows an analogous overlay for SponsorBlock segments (`data-testid="episode-progress"` in `EpisodeCard.vue`), so extending the same overlay technique to chapters keeps the two progress visualizations (card and player) consistent with each other.

## What Changes

- Render chapter tick marks on the episode card's bottom progress strip, using the same overlay technique already used there for SponsorBlock segment markers, positioned by the episode's original chapter times against its total duration.
- Marks are visual-only in the card (no click-to-seek — the card's progress strip already has no interaction), consistent with the strip's existing read-only behavior.
- An episode with no stored chapters renders no chapter marks on its strip; behavior is otherwise unchanged.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `episode-cards`: extends the existing progress-strip requirement to also render chapter marks.

## Impact

- Frontend only: `frontend/src/components/EpisodeCard.vue` (reuses `chapterTimelineMarkers()` introduced by `02-add-player-chapter-markers`).
- Depends on `01-add-chapter-capture-and-embed` for the `chapters` field and `02-add-player-chapter-markers` for the `chapterTimelineMarkers()` helper (reused, not reimplemented).

## Note

Given the progress strip is only ~1px tall and roughly card-width wide, densely spaced chapters may render as visual noise rather than useful information. This proposal proceeds as scoped, but the design explicitly flags this as a risk worth validating against real chapter data before or during implementation — it may turn out that this indicator is not worth keeping on the card and is better left to the player alone (where `03`/`04`/`05` already provide richer chapter affordances).
