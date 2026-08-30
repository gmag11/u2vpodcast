## 1. Store helper

- [ ] 1.1 Add `chapterTimelineMarkers(duration: number, chapters: Chapter[] | null | undefined)` to `frontend/src/stores/player.ts`, mirroring `sponsorBlockTimelineMarkers`'s clamping/filtering rules but returning `{ left, title, startSeconds }` per chapter; verify with unit tests covering an empty/undefined chapter list, a chapter at time 0, a chapter beyond duration (excluded), and normal mid-track chapters
- [ ] 1.2 Add a `chapters` field to the frontend `Episode`/`Chapter` types in `frontend/src/types.ts` if not already present from `01-add-chapter-capture-and-embed`; verify the project type-checks

## 2. Wide composition and expanded view

- [ ] 2.1 Add a `chapterMarkers` computed in `PersistentPlayer.vue` using `player.currentEpisode?.chapters` and the current duration; verify via component test that markers render at the expected `left%` positions for a fixture episode with chapters
- [ ] 2.2 Render chapter marker elements inside the wide composition's scrubber track, styled distinctly from the existing SponsorBlock overlay (new marker style/class), with a click handler that calls `player.seek(marker.startSeconds)`; verify via component test that clicking a marker calls `seek` with the expected time
- [ ] 2.3 Repeat 2.1-2.2 for `PersistentPlayerExpanded.vue`'s scrubber; verify via component test analogous to 2.2

## 3. Compact composition

- [ ] 3.1 Render the same chapter markers on the compact read-only progress track in `PersistentPlayer.vue`, non-interactive (`aria-hidden`, no click handler), alongside the existing SponsorBlock overlay; verify via component test that markers render but a click/tap on the compact track does not trigger a seek

## 4. Regression and manual verification

- [ ] 4.1 Verify existing SponsorBlock marker and seek tests still pass unchanged (no regression from the added overlay)
- [ ] 4.2 Manually verify against a fixture episode with both chapters and an active rejected SponsorBlock interval that activating a chapter marker whose start falls inside the rejected interval lands the playhead at the end of that interval (existing skip behavior applies)
- [ ] 4.3 Manually verify an episode with no stored chapters renders no chapter markers on any composition
