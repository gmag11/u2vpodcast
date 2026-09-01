## 1. Store helpers

- [x] 1.1 Add `nextChapterStart(currentTime, chapters)` returning the next chapter's start time or `null` when in the last chapter; verify with unit tests covering a middle chapter, the last chapter, and an empty chapter list
- [x] 1.2 Add `previousChapterSeekTarget(currentTime, chapters)` implementing the 3-second restart-vs-previous threshold, returning `null` when disabled; verify with unit tests covering: >3s into a middle chapter (restart), <=3s into a middle chapter (go to previous), >3s into the first chapter (restart), <=3s into the first chapter (disabled/null), and an empty chapter list

## 2. Expanded view UI

- [x] 2.1 Add previous-chapter and next-chapter controls to `PersistentPlayerExpanded.vue`, rendered only when `player.currentEpisode?.chapters` is non-empty, grouped near the Chapters section; verify via component test that both controls are absent for an episode with no chapters
- [x] 2.2 Wire next-chapter to `nextChapterStart` + `player.seek(...)`, disabling the control when the helper returns `null`; verify via component test for both the enabled and last-chapter-disabled cases
- [x] 2.3 Wire previous-chapter to `previousChapterSeekTarget` + `player.seek(...)`, disabling the control when the helper returns `null`; verify via component test covering the restart, go-to-previous, and disabled cases

## 3. Regression and manual verification

- [x] 3.1 Verify existing episode-level previous/next control tests are unaffected
- [x] 3.2 Manually verify with a fixture episode with chapters and an active rejected SponsorBlock interval that chapter navigation into a rejected range triggers the existing skip-forward behavior
