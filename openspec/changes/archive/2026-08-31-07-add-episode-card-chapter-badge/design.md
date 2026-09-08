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

**Place the indicator in the existing favorite and playlist icon group** rather than beside the duration or in a new layout region. In the mobile playlist presentation, use three fixed status slots and widen the row only enough to fit them. The empty chapter slot remains present for chapterless episodes so the other status icons do not shift between rows; standard and compact cards do not reserve this space.

**Use the existing visual tooltip pattern** with localized text, hover and keyboard-focus visibility, an accessible relationship via `aria-describedby`, and a native `title` fallback. The indicator remains informational and has no click behavior.

## Risks / Trade-offs

- [Risk] Adding another icon to an already busy card (especially the playlist mobile presentation, which already has several status icons) could clutter the layout → Mitigation: keep the icon small and place it only where existing metadata already lives; if it proves crowded in the playlist presentation specifically, a follow-up can move it to the overflow menu for that presentation only.
