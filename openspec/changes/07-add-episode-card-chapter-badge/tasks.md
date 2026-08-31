## 1. Card indicator

- [x] 1.1 Add a has-chapters icon indicator to `EpisodeCard.vue`, rendered via `v-if="episode.chapters && episode.chapters.length > 0"`, positioned alongside the favorite and playlist icons; verify via component test that it appears for an episode with chapters and is absent (with no reserved space) for one without
- [x] 1.2 Add a localized hover/focus tooltip and verify the indicator renders correctly across the default, compact, and playlist presentations via component tests for each
- [x] 1.3 Keep fixed favorite, playlist, and chapter status slots in the mobile playlist presentation so icons remain aligned when a row has no chapters

## 2. Regression

- [x] 2.1 Verify existing `EpisodeCard` tests (favorite, listened mark, playlist toggle, progress strip) are unaffected
