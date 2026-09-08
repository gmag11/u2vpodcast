## Why

Listeners browsing episode lists have no way to tell which episodes have chapters without opening the player. A small "has chapters" indicator on the card lets them notice this at a glance, consistent with how other card indicators (favorite star, played mark) already surface episode metadata compactly.

## What Changes

- Add a small icon indicator to `EpisodeCard` shown only when the episode has stored chapters, placed alongside the existing favorite and playlist icons (not competing with the primary play/pause/stop controls).
- Add a localized tooltip that identifies the indicator on hover or keyboard focus.
- The indicator is purely informational in this change (no click behavior); it does not open the player or a chapter list.
- Episodes without stored chapters render no indicator. The mobile playlist status row keeps a fixed third icon slot so favorite, playlist, and chapter statuses remain aligned between rows; other presentations reserve no space for an absent indicator.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `episode-cards`: adds a tooltip-enabled has-chapters indicator to the card's existing status/action icon group.

## Impact

- Frontend only: `frontend/src/components/EpisodeCard.vue`.
- Depends on `01-add-chapter-capture-and-embed` for the `chapters` field on episodes.
