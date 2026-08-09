## 1. Core implementation

- [x] 1.1 In `src/handlers/feed.rs`, replace `Episode::read_all(&data.pool)` with `Episode::read_episodes_for_channel(&data.pool, channel_id)` in `get_feed`, so each feed contains only the requested channel's episodes. Confirm the inner `match` still maps `Ok(episodes)` to the RSS items builder unchanged.

## 2. Verification

- [x] 2.1 `cargo build` in the container and redeploy; confirm the image runs. Confirmed by operator.
- [x] 2.2 Seed or confirm a database that has episodes for at least two channels, then `curl -i -u admin:<admin_password> /channels/<id>/feed.xml` for each channel and confirm each feed's `<item>` list contains only that channel's episodes (no `<item>` from another channel). Confirmed by operator.
- [x] 2.3 For each feed, confirm the `<item>` entries are ordered by `published_at` descending (most recent first). Confirmed by operator.
- [x] 2.4 For one episode in each feed, confirm its `<enclosure>` URL is `{url}/media/<channel_id>/<yt_id>.mp3` and that a `curl -i -u admin:<admin_password> <enclosure-url>` returns `200` with the MP3 body (validates the enclosure now resolves for the feed's channel). Confirmed by operator.
- [x] 2.5 Confirm a feed of a channel with no episodes returns `200` with a valid RSS document and an empty `<item>` list. Confirmed by operator.

## 3. Release notes

- [x] 3.1 Note in the release notes for the next tag that feed contents were corrected to per-channel episodes; existing podcast clients will pick up the corrected feeds automatically on their next refresh. Recorded for the next release tag (no CHANGELOG file exists; the project documents releases via git tags).
