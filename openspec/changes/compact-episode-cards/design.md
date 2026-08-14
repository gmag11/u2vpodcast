## Context

The Vue 3 SPA renders `EpisodeCard` components in `EpisodesView` (default variant) and `HistoryView` (compact variant). Each card currently carries a full inline player row (play/pause, stop, seek bar, position counter) plus a second row with speed and volume controls. A global Pinia player store (`frontend/src/stores/player.ts`) owns a single `<audio>` element, and a persistent bottom bar (`PersistentPlayer.vue`) already surfaces position (scrubber), volume, and speed controls for every episode.

Because the persistent bar already provides the advanced controls, the card's inline copies are redundant. The goal is to reduce the card's vertical footprint by deleting those controls while keeping the card playable (play/pause/stop) and showing only the total duration.

Constraint: the persistent player (`PersistentPlayer.vue`) and the player store are NOT modified. The card keeps reading from the same shared store.

## Goals / Non-Goals

**Goals:**
- Reduce the vertical space an episode card occupies.
- Remove the inline seek bar, speed control, volume control, and live position counter from the card.
- Keep the total duration label on the card.
- Reposition play/pause and stop buttons: below the thumbnail on desktop (`sm+`), right of the thumbnail on mobile.
- Keep play/pause/stop fully bound to the shared player store.

**Non-Goals:**
- No change to the persistent bottom player bar or its controls.
- No change to the player store API (`seek`, `setVolume`, `toggleMute`, `setSpeed`, `currentLabel` stay in place for the bar).
- No change to card content, thumbnails, metadata, links, or the compact-variant channel-title label.
- No backend or API changes.

## Decisions

### 1. Delete inline seek, speed, volume UI

Remove the seek slider row and the speed/volume row from `EpisodeCard.vue`. Delete now-unused script state and handlers: `showSpeed`, `speeds`, `onSeek`, `onVolumeInput`, and the `PhGauge`, `PhSpeakerHigh`, `PhSpeakerSlash` imports. The shared store still owns the `<audio>` element, so playback keeps working.

Rationale: the persistent bar is the canonical home for these controls. Duplicating them only costs vertical space. The store API is untouched so the bar is unaffected.

### 2. Duration label shows total only

When the card's episode is the current one, render `player.durationLabel` (total duration). Otherwise render `props.episode.duration`. Drop the `player.currentLabel` position counter.

Rationale: the seek/position context is gone from the card, so a position counter would be meaningless there; the persistent bar still shows live position.

### 3. Play/stop placement by breakpoint

Wrap the thumbnail plus buttons so their relative position flips at the `sm` breakpoint, matching the existing Tailwind `sm:` pattern used across the card:

- **Mobile (`< sm`, default):** the card body is a column. The thumbnail row becomes a horizontal strip: thumbnail on the left, the play/stop buttons stacked to its right. Thumbnail gets a fixed square-ish size (e.g. `h-20 w-28`/`h-24 w-24`) instead of `w-full` so buttons fit beside it.
- **Desktop (`sm+`):** the thumbnail column contains the thumbnail on top with the play/stop buttons below it. Buttons are hidden on mobile (`sm:hidden` / `hidden sm:flex` variants) so only one placement renders at a time.

Both variants keep identical button markup, aria-labels, disabled states, and store bindings (`player.play`, `player.togglePlay`, `player.stop`, `isCurrent`, `isPlaying`, `player.loading`).

Rationale: reuses the existing `sm:` breakpoint the card already uses for the thumbnail/info split, so no new breakpoint logic is introduced. Using two elements (one per breakpoint) avoids moving DOM nodes with JS.

### 4. Keep store API surface

No store changes. `seek`, `setVolume`, `toggleMute`, `setSpeed`, and `currentLabel` remain exported for the persistent bar. The card simply stops calling some of them.

Rationale: minimal blast radius; the persistent player spec (unchanged runtime) still depends on them.

## Risks / Trade-offs

- **Removing inline seek on mobile leaves no seek in the card.** → Acceptable: the persistent bar appears on playback and provides the scrubber; this matches the "cards stay compact" goal.
- **Play/stop position differs from current layout, could surprise users.** → Minor; persistent bar is the continuity cue. No migration needed for a visual-only change.
- **Two button groups (mobile + desktop) risk style drift.** → Keep both blocks rendering the same button components/styling; layout differs only via flex containers.
- **Tailwind classes now reference thumbnail sizes that must match between breakpoint variants.** → Keep size classes constant across the two layouts (single `h-* w-*` values).

## Migration Plan

Pure frontend component change; deploy with the normal SPA build. Rollback = revert `EpisodeCard.vue`. No schema, data, or API migration.
