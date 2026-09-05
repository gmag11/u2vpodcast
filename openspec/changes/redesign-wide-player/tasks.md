## 1. Wide composition layout

- [ ] 1.1 Move the interactive progress track (fill + chapter + SponsorBlock markers) to a full-width strip along the top edge of the `player-wide` block, using the existing `onSeek` handler and marker markup, with an extended invisible hit area (`py`/`-inset-y-*`) while keeping the visible track thin
- [ ] 1.2 Convert the wide thumbnail to a static element (remove the expand interaction; the wide bar already has none, so confirm no click/button remains)
- [ ] 1.3 Add the `elapsed / total` readout beside the thumbnail using `player.currentLabel` / `player.durationLabel` with tabular numerals
- [ ] 1.4 Replace the wide metadata block with two lines: episode title wrapped in `ScrollingText` (active while `player.playing`) on the first line, and `Chapter · Channel` (current chapter title via `currentChapterTitle`, when present, followed by `channel_title`) on the second line
- [ ] 1.5 Remove the now-unused `label / duration` metadata line from the wide block and retain all existing controls (previous, play/pause, stop, next, speed, shuffle, repeat, mute/volume, queue) in their current horizontal positions

## 2. Tests

- [ ] 2.1 Update `frontend/src/components/PersistentPlayer.test.ts` wide-composition assertions to the new structure (full-width scrubber, thumbnail, elapsed/total time, two-line metadata)
- [ ] 2.2 Add wide-specific scenarios: full-width scrubber seeks, extended hit area, title scrolls while playing / truncates while paused / reduced motion, static thumbnail, `Chapter · Channel` vs channel-only line

## 3. Verification

- [ ] 3.1 Run frontend test suite (`pnpm --dir frontend test` or equivalent) and lint/typecheck, confirming all pass
- [ ] 3.2 Manually verify the wide bar at >= 640px: precise seeking across the full width, metadata order, marquee on overflow, and that compact/expanded compositions are unchanged
