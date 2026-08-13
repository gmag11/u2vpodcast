## 1. Feed resolution

- [x] 1.1 In `src/handlers/feed.rs`, resolve the channel with `Channel::read_by_id_or_slug(&data.pool, &key)` instead of `read_by_slug`
- [x] 1.2 Build the enclosure URL from the resolved `channel.slug` (canonical slug) instead of the raw path segment

## 2. Verification

- [x] 2.1 Confirm `/channels/{id}/feed.xml` returns the identical RSS document as the slug URL for the same channel
- [x] 2.2 Confirm enclosure URLs in an id-requested feed point at `/media/{slug}/{yt_id}.mp3`
- [x] 2.3 Confirm unknown numeric ids return `404 Not Found` and the short `/{slug}/feed.xml` route still resolves by slug only
- [x] 2.4 Run `cargo build` and `cargo test` (backend) to verify no regressions

