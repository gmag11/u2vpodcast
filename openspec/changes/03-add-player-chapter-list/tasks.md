## 1. Store helper

- [ ] 1.1 Add `currentChapterIndex(currentTime, chapters)` to `frontend/src/stores/player.ts` (or confirm and reuse an existing equivalent if `04-add-player-current-chapter-label` already landed one); verify with unit tests covering: time before the first chapter, time within a middle chapter, time after the last chapter's end, and an empty chapter list

## 2. Expanded view UI

- [ ] 2.1 Add a Chapters section to `PersistentPlayerExpanded.vue`, rendered only when `player.currentEpisode?.chapters` is non-empty, listing title + formatted start time per chapter; verify via component test that the section is absent for an episode with no chapters and present with the right rows for one that has them
- [ ] 2.2 Wire row activation to `player.seek(chapter.start)`; verify via component test that clicking/tapping a row calls `seek` with the expected time
- [ ] 2.3 Highlight the row matching `currentChapterIndex`, updating reactively as `currentTime` changes; verify via component test that the highlighted row changes as the mocked current time crosses a chapter boundary

## 3. Regression and manual verification

- [ ] 3.1 Verify existing expanded-view tests (queue panel, transport controls, scrubber) still pass unchanged
- [ ] 3.2 Manually verify with a fixture episode that has many chapters that the section scrolls internally without pushing other expanded-view controls off-screen
