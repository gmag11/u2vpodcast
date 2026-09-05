## 1. Wide composition layout

- [x] 1.1 Move the interactive progress track (fill + chapter + SponsorBlock markers) to a full-width strip along the top edge of the `player-wide` block, using the existing `onSeek` handler and marker markup, with an extended invisible hit area (`py`/`-inset-y-*`) while keeping the visible track thin
- [x] 1.2 Convert the wide thumbnail to a static element (remove the expand interaction; the wide bar already has none, so confirm no click/button remains)
- [x] 1.3 Add the `elapsed / total` readout beside the thumbnail using `player.currentLabel` / `player.durationLabel` with tabular numerals
- [x] 1.4 Replace the wide metadata block with two lines: episode title wrapped in `ScrollingText` (active while `player.playing`) on the first line, and `Chapter · Channel` (current chapter title via `currentChapterTitle`, when present, followed by `channel_title`) on the second line
- [x] 1.5 Remove the now-unused `label / duration` metadata line from the wide block and retain all existing controls (previous, play/pause, stop, next, speed, shuffle, repeat, mute/volume, queue) in their current horizontal positions
- [x] 1.6 Split the wide metadata block into three lines: episode title (`ScrollingText`), current chapter title (optional), and channel name on its own line
- [x] 1.7 Add a "Chapters" toggle button in the wide bar that opens a popover with previous/next chapter controls and the full chapter list (jump-to-chapter, active-row highlight, rejected-interval skip), reusing the expanded view's chapter helpers and formatting; render no Chapters control when the episode has no stored chapters

## 2. Tests

- [x] 2.1 Update `frontend/src/components/PersistentPlayer.test.ts` wide-composition assertions to the new structure (full-width scrubber, thumbnail, elapsed/total time, two-line metadata)
- [x] 2.2 Add wide-specific scenarios: full-width scrubber seeks, extended hit area, title scrolls while playing / truncates while paused / reduced motion, static thumbnail, `Chapter · Channel` vs channel-only line
- [x] 2.3 Update wide metadata assertions to the three-line structure and add wide chapters-popover scenarios: opens, lists chapters, seeks on activation, highlights active chapter, previous/next navigation behavior, and no Chapters control without stored chapters

## 3. Verification

- [x] 3.1 Run frontend test suite (`pnpm --dir frontend test` or equivalent) and lint/typecheck, confirming all pass
- [x] 3.2 Manually verify the wide bar at >= 640px: precise seeking across the full width, metadata order, marquee on overflow, and that compact/expanded compositions are unchanged
