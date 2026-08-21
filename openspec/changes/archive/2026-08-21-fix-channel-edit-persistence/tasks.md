## 1. Backend Persistence

- [x] 1.1 Add `title` to `UpdateChannel` and persist `title` and `url` in `Channel::update`'s SQL (keep slug immutable)
- [x] 1.2 Reject empty titles with a clear error; keep response shape unchanged
- [x] 1.3 Update `saveChannel` in `ChannelsView.vue` to apply the server response to the local row instead of the raw local object

## 2. Verification & Regression

- [x] 2.1 Edit a channel's title/url in the UI, reload the page, confirm the edit persists and the slug/feed URLs are unchanged
- [x] 2.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #6 resolved and no new bugs appeared