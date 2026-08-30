## Context

`EpisodeCard.vue` already renders several conditional icon indicators (favorite star, listened mark, playlist membership) following a consistent `v-if` pattern keyed off episode fields. Episodes gain a `chapters` field from `01-add-chapter-capture-and-embed`.

## Goals / Non-Goals

**Goals:**
- Keep the indicator visually lightweight (small icon, no text label) so it doesn't compete with existing badges/controls across the card's default, compact, and playlist presentations.

**Non-Goals:**
- Any interaction (tapping the badge to open a chapter list is out of scope — see `03-add-player-chapter-list` for the actual list, which lives in the player, not the card).
- Chapter tick marks on the card's progress strip (tracked separately in `08-add-episode-card-chapter-marks`).

## Decisions

**Use a Phosphor icon already available in the project's icon set** (e.g., a list/bookmark-style icon distinct from existing favorite/playlist icons) rendered conditionally via `v-if="episode.chapters && episode.chapters.length > 0"`, matching the existing conditional-icon pattern used for other badges.

**Place the indicator near existing compact metadata** (e.g., next to the duration label) rather than introducing a new layout region, to minimize footprint growth across the three card presentations (default, compact, playlist).

## Risks / Trade-offs

- [Risk] Adding another icon to an already busy card (especially the playlist mobile presentation, which already has several status icons) could clutter the layout → Mitigation: keep the icon small and place it only where existing metadata already lives; if it proves crowded in the playlist presentation specifically, a follow-up can move it to the overflow menu for that presentation only.
