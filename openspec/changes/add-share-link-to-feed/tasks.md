# Tasks

## 1. Feed item public link

- [x] 1.1 Thread the effective share secret from `build_feed` and `get_global_feed` through `channel_items`/`global_items` into `episode_item`, and set each item's RSS 2.0 `<link>` to `{url}/app/share/{token}` using the deterministic share-token derivation; verify with a unit test that a built item's `<link>` token resolves to that item's `yt_id` and is identical across two builds
- [x] 1.2 Update the existing `feed.rs` item tests for the new builder arguments and add assertions that adding `<link>` left the `<enclosure>`, `<guid>`, and `<pubDate>` unchanged; verify with `cargo test handlers::feed`
- [x] 1.3 Confirm the per-channel feed, the legacy slug alias, the legacy numeric-id alias, and the global feed all emit the same `<link>` prefix and that two builds of the same feed produce identical links; verify with a test covering both legacy routes and the global feed

## 2. Documentation

- [x] 2.1 Document in `README.md` that feed items link to a deterministic public share page; verify the README matches the emitted `<link>` prefix

## 3. Integration verification

- [x] 3.1 Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test`
- [x] 3.2 End-to-end check: fetch a channel feed with credentials and confirm every item carries a `{url}/app/share/…` `<link>` and that two fetches yield identical links; open that link's metadata and audio without credentials and confirm they resolve; confirm the feed itself still returns `401` without credentials
