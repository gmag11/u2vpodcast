## Context

`EpisodeCard.vue`'s bottom progress strip already overlays SponsorBlock segment markers using the same absolutely-positioned-`<div>` technique as the persistent player (`sponsorBlockTimelineMarkers`). `02-add-player-chapter-markers` introduces `chapterTimelineMarkers(duration, chapters)` for the player; this change reuses it verbatim for the card.

## Goals / Non-Goals

**Goals:**
- Reuse `chapterTimelineMarkers()` exactly as `02-add-player-chapter-markers` defines it — no card-specific variant.
- Keep the strip fully read-only, matching its existing behavior.

**Non-Goals:**
- Any click-to-seek from the card (the strip has never supported interaction; this change does not add any).
- Deciding now whether marks are visually useful at this scale — see the proposal's Note; if validation during implementation shows they're too dense to read, that's a legitimate reason to descope at that time rather than force it in.

## Decisions

**Reuse the player's `chapterTimelineMarkers()` helper unchanged** rather than duplicating the clamping/filtering logic in `EpisodeCard.vue`, keeping a single source of truth for how chapter times map to `left%` positions.

**Render markers with the same visual treatment chosen in `02-add-player-chapter-markers`** (same marker color/class) so the two surfaces (card and player) are visually consistent rather than introducing a third distinct style.

## Risks / Trade-offs

- [Risk] The strip is very thin (~1px tall) and card-width wide; densely spaced chapters could render as a solid smear rather than distinguishable marks → Mitigation: no de-duplication/minimum-spacing logic is in scope; if real chapter data shows this is unreadable, treat that as grounds to reconsider keeping this indicator on the card at all (per the proposal's Note) rather than adding ad hoc spacing logic.
