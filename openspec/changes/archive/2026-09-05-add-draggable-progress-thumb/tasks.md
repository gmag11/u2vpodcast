## 1. Shared Scrubber Component

- [x] 1.1 Create `frontend/src/components/ProgressScrubber.vue` with props for progress (0-100), duration, sponsor/chapter markers, and data-testid; render track, fill, SponsorBlock segments, and chapter markers (reusing existing marker markup/classes from `PersistentPlayer.vue`/`PersistentPlayerExpanded.vue`)
- [x] 1.2 Render a draggable thumb (accent-colored dot, larger hit area) at the current playback position on top of the markers, using the existing `bg-accent-400` fill color
- [x] 1.3 Implement pointer-event drag handling (pointerdown/move/up with `setPointerCapture`) storing a local `dragRatio` preview; guard on unknown duration (`duration <= 0` or non-finite)
- [x] 1.4 Add drag-preview tooltip above the thumb showing the target time in `elapsed / total` format (reusing the existing time formatter), clamped inside the viewport at track ends
- [x] 1.5 On release, emit `seek(seconds)`; treat a press with negligible movement as click-to-seek; hide tooltip after release

## 2. Wire into Compositions

- [x] 2.1 Replace the wide progress track in `PersistentPlayer.vue` with `ProgressScrubber`, keeping `data-testid="player-progress-compact"`/wide test ids, `role="slider"`, aria labels, and existing `onSeek` guard behavior; remove the now-duplicated inline track markup
- [x] 2.2 Replace the expanded progress track in `PersistentPlayerExpanded.vue` with `ProgressScrubber` (keep `data-testid="player-progress-expanded"` and chapter-marker seek handling)
- [x] 2.3 Leave the compact read-only track in `PersistentPlayer.vue` untouched (no thumb, no drag, no seek), keeping its `aria-hidden` marker-only markup
- [x] 2.4 Ensure chapter-marker tooltips and `@click.stop="player.seek(...)"` still work inside the shared component for wide/expanded; compact markers stay non-interactive read-only as before

## 3. Verification

- [x] 3.1 Run the frontend test suite (check `frontend/package.json` for the test script) and add or update component tests for drag preview, click-to-seek, and unknown-duration guard if test infrastructure exists
- [x] 3.2 Run frontend lint and typecheck (per `frontend/package.json` scripts); fix any issues
- [x] 3.3 Manually verify drag + tooltip on wide, expanded, and compact (mobile) tracks in the running SPA, confirming release seeks and SponsorBlock skip still applies