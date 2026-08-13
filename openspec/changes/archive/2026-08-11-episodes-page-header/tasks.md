## 1. Channel title resolution

- [x] 1.1 In `EpisodesView.vue`, add a `channelTitle` ref and resolve it during `load()` by fetching the channel list (`api.getChannels()`) and matching `id === Number(route.params.channelId)`; consolidate with the existing slug fallback so both share one channel-list lookup
- [x] 1.2 Set a fallback title (`'Episodes'`) when no matching channel is found

## 2. Header UI

- [x] 2.1 Add a page header as the first element of the episodes page content (below the shared app header), with a left arrow button and the channel title
- [x] 2.2 Wire the left arrow to `router.push({ name: 'channels' })`; use a Phosphor left-arrow icon in a bordered icon button styled with existing tokens
- [x] 2.3 Keep spacing consistent (header above the search bar), reusing `font-display` and text tokens

## 3. Verification

- [x] 3.1 `pnpm lint`, `pnpm run build`, and `pnpm test` pass
- [x] 3.2 Smoke-test with the running app: open a channel's episodes page → header shows the channel title and a back arrow; click the arrow → returns to the channel list
