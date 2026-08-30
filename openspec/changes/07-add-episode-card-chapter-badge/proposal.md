## Why

Listeners browsing episode lists have no way to tell which episodes have chapters without opening the player. A small "has chapters" indicator on the card lets them notice this at a glance, consistent with how other card indicators (favorite star, played mark) already surface episode metadata compactly.

## What Changes

- Add a small icon indicator to `EpisodeCard` shown only when the episode has stored chapters, placed alongside existing compact metadata (not competing with the primary play/pause/stop controls).
- The indicator is purely informational in this change (no click behavior); it does not open the player or a chapter list.
- Episodes without stored chapters render no indicator and no reserved space for it.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `episode-cards`: adds a has-chapters indicator to the card's compact metadata.

## Impact

- Frontend only: `frontend/src/components/EpisodeCard.vue`.
- Depends on `01-add-chapter-capture-and-embed` for the `chapters` field on episodes.
