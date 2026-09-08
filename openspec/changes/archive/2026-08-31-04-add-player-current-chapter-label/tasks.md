## 1. Store helper

- [x] 1.1 Add (or confirm and reuse, if `03-add-player-chapter-list` already landed it) `currentChapterIndex(currentTime, chapters)` in `frontend/src/stores/player.ts`; verify with unit tests covering: no chapters, time before the first chapter, time within a chapter, and time after the last chapter

## 2. Wide composition and expanded view

- [x] 2.1 Add a computed chapter-title label in `PersistentPlayer.vue`'s wide composition, rendered near the episode title only when `currentChapterIndex >= 0`, truncated with ellipsis when it overflows; verify via component test that the label shows the correct title and disappears (with no reserved space) for an episode with no chapters
- [x] 2.2 Repeat 2.1 for `PersistentPlayerExpanded.vue`; verify via an analogous component test
- [x] 2.3 Verify via component test that the label updates when the mocked current time crosses from one chapter into the next

## 3. Compact composition and manual verification

- [x] 3.1 Add the computed chapter-title label to the compact composition between the episode title and channel/playback-time line, truncated with ellipsis; verify its placement and that no element or reserved space remains when there is no current chapter
- [x] 3.2 Manually verify with a fixture episode with chapters that the label updates smoothly during real playback without flicker at chapter boundaries
