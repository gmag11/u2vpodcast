## Context

The seek bar in both `PersistentPlayer.vue` (wide and compact compositions) and `PersistentPlayerExpanded.vue` already overlays SponsorBlock range markers as absolutely-positioned `<div>`s inside the track, computed by `sponsorBlockTimelineMarkers(duration, segments)` (`frontend/src/stores/player.ts`), which converts `{start, end, category}` seconds into `{left%, width%, category}` for CSS positioning. `mediaUrl()` always requests the original `/media/{slug}/{yt_id}.mp3`, and `duration`/`currentTime` in the player store always reflect that original file, so chapter times need no coordinate translation — they're on the same timeline the player already uses.

## Goals / Non-Goals

**Goals:**
- Reuse the existing overlay technique and component structure exactly, adding a second marker type (points, not ranges).
- Keep chapter markers visually distinct from SponsorBlock markers.
- Wire marker activation to the existing `seek()` action so rejected-interval skip behavior applies uniformly.

**Non-Goals:**
- A dedicated chapter list/menu, current-chapter label, or prev/next-chapter transport controls (tracked separately in `03-add-player-chapter-list`, `04-add-player-current-chapter-label`, `05-add-player-chapter-navigation`).
- Any chapter markers on `EpisodeCard.vue` (tracked separately in `08-add-episode-card-chapter-marks`).
- Any backend change.

## Decisions

**Add `chapterTimelineMarkers(duration, chapters)` alongside `sponsorBlockTimelineMarkers`, mirroring its shape.** Returns `Array<{ left: number; title: string; startSeconds: number }>` (no `width`, since chapters are points): `left = (chapter.start / duration) * 100`, clamped/filtered exactly like the SponsorBlock helper (skip entries with non-finite or out-of-range times). Keeping the same file and a parallel naming/shape convention minimizes divergence from the pattern the codebase already established for SponsorBlock markers.

**Render chapter markers as a distinct thin element (e.g. a 1-2px wide vertical bar with higher z-index), not reusing the SponsorBlock marker's range-block styling.** A specific color/class distinct from `bg-sponsorblock`/`bg-sponsorblock-other` (e.g. a new `bg-chapter-marker` utility) keeps the two marker types visually unambiguous per the spec's explicit distinctness requirement.

**Wire click/tap on a chapter marker to `player.seek(startSeconds)`, reusing the existing `onSeek`-adjacent seek path** rather than introducing a new seek function — this automatically inherits the already-specified rejected-interval skip behavior (`persistent-audio-player`'s "Web playback skips configured rejected intervals" requirement), since `seek()` already clamps forward out of rejected ranges.

**Compact track markers are visual-only**, added to the same non-interactive `<div>` that already renders the compact SponsorBlock overlay (`aria-hidden="true"`, no click handler), consistent with the compact composition's existing no-seek-interaction rule.

## Risks / Trade-offs

- [Risk] Many closely-spaced chapters could make markers visually cluttered on the compact/narrow track → Mitigation: no de-duplication logic is in scope for this change; if it proves noisy in practice, a follow-up can add minimum-spacing collapsing. Out of scope here since no real-world chapter density data suggests it's needed yet.
- [Trade-off] Reusing `seek()` means a chapter landing inside a rejected interval will visibly jump twice (to the chapter, then forward past the rejected range) rather than landing directly past it — accepted because it matches the existing, already-specified seek behavior exactly, and avoids a special-cased seek path just for chapters.
