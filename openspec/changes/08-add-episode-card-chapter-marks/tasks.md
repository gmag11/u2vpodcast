## 1. Card progress strip

- [ ] 1.1 Add a `chapterMarkers` computed in `EpisodeCard.vue` using `chapterTimelineMarkers()` (from `02-add-player-chapter-markers`) against the card's episode duration; verify via component test that markers render at expected `left%` positions for a fixture episode with chapters
- [ ] 1.2 Render chapter marker elements inside the existing bottom progress strip, using the same visual treatment as the player's chapter markers; verify via component test that the strip shows both chapter and SponsorBlock markers together, visually distinguishable via `data-testid`/class assertions
- [ ] 1.3 Verify via component test that clicking/dragging the progress strip has no effect (existing read-only behavior), with chapter markers present

## 2. Regression and manual validation

- [ ] 2.1 Verify existing progress-strip and SponsorBlock-marker tests on `EpisodeCard` are unaffected
- [ ] 2.2 Manually review a fixture episode with many closely-spaced chapters rendered at real card width; if marks are unreadable at this scale, flag this as a decision point before considering the change complete (per proposal.md's Note)
