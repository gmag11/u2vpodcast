## 1. Cache Infrastructure

- [ ] 1.1 Add an `images_dir()` helper (mirrors `audios_dir()`: `/app/images` in the container, `images/` locally) and ensure the directory exists at startup
- [ ] 1.2 Register a static route for `/images/{filename}` scoped to the cache directory (public, like the SPA static assets)

## 2. Cache Population & Refresh

- [ ] 2.1 Add a shared image-download helper (bounded, timeout-capped, max-size cap, atomic temp-file + rename write) that stores `{slug}.jpg` and returns the local URL on success while keeping the old file on failure
- [ ] 2.2 Populate the cache in `Channel::new` (creation) and `Channel::update_image` (manual refresh), setting `channel.image` to the local URL
- [ ] 2.3 Refresh the cached image during worker `update_channel` (each scheduled/forced sync), skipping inactive channels per the `active` flag semantics
- [ ] 2.4 Ensure the download path uses the same fetch mechanism as `YTInfo::new` so the single-connection throttle (`limit-youtube-concurrency`) covers it once implemented

## 3. Verification & Regression

- [ ] 3.1 Verify: page load makes zero requests to YouTube for images (network log); local `/images/{slug}.jpg` serves bytes; manual refresh replaces the file; sync refreshes it; failed download keeps the previous image and does not blank `channel.image`
- [ ] 3.2 Verify cache route confinement: request outside the cache directory is not served
- [ ] 3.3 Run the full test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm the cache introduces no new bugs (CORS/hash/active/blocking-io fixes still hold) and does not bypass the YouTube throttle when implemented