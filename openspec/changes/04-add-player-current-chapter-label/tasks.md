## 1. Store helper

- [ ] 1.1 Add (or confirm and reuse, if `03-add-player-chapter-list` already landed it) `currentChapterIndex(currentTime, chapters)` in `frontend/src/stores/player.ts`; verify with unit tests covering: no chapters, time before the first chapter, time within a chapter, and time after the last chapter

## 2. Wide composition and expanded view

- [ ] 2.1 Add a computed chapter-title label in `PersistentPlayer.vue`'s wide composition, rendered near the episode title only when `currentChapterIndex >= 0`, truncated with ellipsis when it overflows; verify via component test that the label shows the correct title and disappears (with no reserved space) for an episode with no chapters
- [ ] 2.2 Repeat 2.1 for `PersistentPlayerExpanded.vue`; verify via an analogous component test
- [ ] 2.3 Verify via component test that the label updates when the mocked current time crosses from one chapter into the next

## 3. Regression and manual verification

- [ ] 3.1 Verify the compact composition renders no chapter label under any condition (existing compact-composition tests continue to pass unchanged)
- [ ] 3.2 Manually verify with a fixture episode with chapters that the label updates smoothly during real playback without flicker at chapter boundaries
