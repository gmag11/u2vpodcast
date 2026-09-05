## Why

The persistent player's progress track only supports jump-by-click (and drag for the wide/expanded compositions via a hidden hit area), but there is no visible draggable thumb, so users cannot see or fine-tune their current playback position while scrubbing. On the compact composition the track is read-only and cannot be scrubbed at all. This makes precise seeking awkward, especially on long episodes.

## What Changes

- Add a visible draggable thumb (dot) on the progress track of the **wide** and **expanded** player compositions, rendered at the current playback position, using the same accent color as the progress fill.
- While dragging the thumb, show a tooltip above it displaying the time (current label format, `elapsed / total`) of the position that will be sought on release.
- While dragging, update the thumb position and tooltip to follow the pointer, without committing a seek; on release, seek playback to the chosen position (subject to existing SponsorBlock skip behavior).
- Make the **compact** (mobile) progress track interactive: show the same draggable thumb and tooltip so users can scrub from the collapsed mobile player.
- Keep the existing click-to-seek behavior working alongside the thumb.

## Capabilities

### New Capabilities

- `draggable-progress-thumb`: A reusable draggable thumb on the player progress track, with drag-preview tooltip showing target time, applied consistently across the compact, wide, and expanded player compositions.

### Modified Capabilities

- `persistent-audio-player`: The interactive scrubber requirements change so that all player compositions (compact included) render a draggable thumb with a drag-preview time tooltip and support scrubbing by drag, not just click.

## Impact

- `frontend/src/components/PersistentPlayer.vue` — compact (mobile) and wide progress tracks.
- `frontend/src/components/PersistentPlayerExpanded.vue` — expanded progress track.
- Possibly a small shared Vue component for the thumb + tooltip (e.g. `ProgressScrubber.vue`).
- No backend, store, or API changes. Player `seek()` in `frontend/src/stores/player.ts` is reused unchanged.
- i18n: reuse existing time formatting / `player.seek` labels; add any new aria/label strings if needed.
