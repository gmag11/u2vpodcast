## 1. Card indicator

- [ ] 1.1 Add a has-chapters icon indicator to `EpisodeCard.vue`, rendered via `v-if="episode.chapters && episode.chapters.length > 0"`, positioned near existing compact metadata; verify via component test that it appears for an episode with chapters and is absent (with no reserved space) for one without
- [ ] 1.2 Verify the indicator renders correctly across the default, compact, and playlist presentations via component tests for each

## 2. Regression

- [ ] 2.1 Verify existing `EpisodeCard` tests (favorite, listened mark, playlist toggle, progress strip) are unaffected
