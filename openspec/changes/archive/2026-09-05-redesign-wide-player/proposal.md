## Why

The desktop (wide, viewport >= 640px) persistent player bar has not followed the recent compact (mobile) redesign. Its single dense row squeezes the interactive scrubber into a short centered segment, making precise seeking harder, and it hard-truncates long episode titles. The goal is to bring the wide bar in line with the mobile visual language while keeping the fixed, non-expandable single-height bar.

## What Changes

- Move the interactive progress scrubber (with chapter and SponsorBlock markers) to a full-width strip along the top edge of the wide bar, maximizing horizontal seek precision while taking almost no vertical space. The visual strip stays thin; the interactive hit area extends beyond it for easier targeting.
- Reorganize the left metadata block from one title + chapter + `label / duration` line to three lines: the episode title (using the existing scrolling/`ScrollingText` marquee when it overflows, matching the compact bar), the current chapter title (optional), and the channel name.
- Move the elapsed/total time readout (`currentLabel / durationLabel`) to sit beside the thumbnail, using tabular numerals.
- Keep the thumbnail static (not clickable) and keep all existing controls in their current horizontal position: previous, play/pause, stop, next, speed, shuffle, repeat, mute/volume, and the "Up next" queue panel.
- Add a "Chapters" toggle that opens a popover (mirroring the queue panel pattern) exposing previous/next chapter navigation and the full chapter list with jump-to-chapter and active-chapter highlighting, matching the mobile expanded view's chapter behavior without a full-screen expanded view.
- No new capabilities; no change to playback state or transport behavior, and the compact/expanded compositions are unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `persistent-audio-player`: Update the "Persistent bottom player bar" requirement's wide-composition layout — full-width top-edge scrubber, two-line scrolling metadata (title + `Chapter · Channel`), and elapsed/total time beside the thumbnail — while retaining all existing controls and behaviors.

## Impact

- `frontend/src/components/PersistentPlayer.vue` — the `player-wide` block (markup and styles only; no store logic changes).
- Tests: `frontend/src/components/PersistentPlayer.test.ts` — update/extend coverage for the new wide layout.
- No API, backend, store, or data-model changes.
