## Why

Tick marks (from `02-add-player-chapter-markers`) show *where* chapters fall on the timeline but not *what* they're called without hovering/tapping each one individually. A dedicated chapter list lets a listener scan all chapter titles at once and jump directly to any of them — a pattern already familiar from mainstream podcast/audio apps (Apple Podcasts, Overcast).

## What Changes

- Add a "Chapters" section to the expanded "now playing" view (`PersistentPlayerExpanded.vue`), shown only when the current episode has stored chapters, listing every chapter's title and start time in order.
- Tapping a chapter row seeks playback to that chapter's start time (subject to existing SponsorBlock rejected-interval skip behavior).
- The row corresponding to the chapter containing the current playback position is visually highlighted, and updates live as playback progresses.
- An episode with no stored chapters shows no Chapters section; nothing else in the expanded view changes.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `persistent-audio-player`: adds a chapter list requirement to the expanded now-playing view.

## Impact

- Frontend only: `frontend/src/components/PersistentPlayerExpanded.vue` (new Chapters section), `frontend/src/stores/player.ts` (a `currentChapter` computed or equivalent helper identifying which chapter contains `currentTime`, reusable by `04-add-player-current-chapter-label` if that change lands first or after).
- Depends on `01-add-chapter-capture-and-embed` for the `chapters` field on episodes. Does not depend on `02-add-player-chapter-markers` functionally, though both touch the same components.
