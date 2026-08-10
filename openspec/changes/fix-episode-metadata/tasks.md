## 1. Core implementation

- [x] 1.1 In `src/models/ytdlp.rs`, add `#[serde(default)] pub timestamp: Option<i64>` to the `YtVideo` struct so the Unix epoch timestamp from yt-dlp is captured for precise episode ordering.

- [x] 1.2 In `src/utils/worker.rs`, replace `parse_date(&ytvideo.upload_date)` with a conversion from `ytvideo.timestamp`: use `Utc.timestamp_opt(ts, 0).unwrap()` when `timestamp` is `Some`, falling back to the current `upload_date` → midnight UTC logic when `None`. Remove the unused `parse_date` function and its `NaiveDate`/`NaiveDateTime` imports.

- [x] 1.3 In `src/handlers/feed.rs`, fix the `pubDate` format: replace `episode.published_at.to_string()` with `episode.published_at.to_rfc2822()` so the `<pubDate>` element follows RFC 2822.

- [x] 1.4 In `src/handlers/feed.rs`, prepend the YouTube video link to the episode description: build the description string as `format!("{}\n\n{}", episode.webpage_url, episode.description)` and use it for both `ItemBuilder::description` and `ITunesItemExtensionBuilder::summary`.

- [x] 1.5 In `src/handlers/feed.rs`, register the legacy route in `web_feed`: add a second `web::resource("/{slug}/feed.xml").route(web::get().to(get_feed)).wrap(BasicAuthGuard)` so the feed is also served at `/{slug}/feed.xml` alongside `/channels/{slug}/feed.xml`, using the same handler for both.

- [x] 1.6 In `src/handlers/feed.rs`, make `get_feed` return a plain `404 Not Found` when the channel slug does not exist. Previously it propagated `Error`, whose `error_response` panics when no session is attached (the feed route never sets one), turning the intended 404 into a 500. Refactor the handler so every branch returns `HttpResponse` directly (200 with the RSS body, 500 on DB error, 404 on unknown slug).

## 2. Verification

- [x] 2.1 `cargo build` in the container and redeploy; confirm the image runs.
- [x] 2.2 Wait for the worker to process at least one new video from a channel with multiple same-day videos. Then `curl -i -u admin:<admin_password> /channels/<slug>/feed.xml` and verify episodes are ordered by their precise YouTube timestamps (not grouped arbitrarily by date).
- [x] 2.3 In the same feed, verify each `<pubDate>` element follows RFC 2822 format (e.g., `"Fri, 15 Mar 2024 14:30:00 +0000"`).
- [x] 2.4 Verify each `<item>`'s `<description>` and `<itunes:summary>` both start with the YouTube video URL (`https://www.youtube.com/watch?v={yt_id}`) followed by a blank line and the video description.
- [x] 2.5 Subscribe to the feed in a podcast client and confirm: (a) episodes appear in the same order as on YouTube, (b) dates are correct, and (c) the YouTube link is tappable/clickable from the episode description.
- [x] 2.6 Confirm the `<guid>` (yt_id) is unchanged for all episodes, so existing podcast subscriptions do not re-download media.
- [x] 2.7 `curl -i -u admin:<admin_password> /<slug>/feed.xml` and confirm it returns `200 OK` with the same `<item>` entries as `/channels/<slug>/feed.xml` for the same channel.
- [x] 2.8 Confirm the frontend still links `/channels/<slug>/feed.xml` (canonical URL unchanged).
- [x] 2.9 `curl -i -u admin:<admin_password> /unknown-slug/feed.xml` and confirm it returns `404 Not Found`, and the same for `/channels/unknown-slug/feed.xml`.

## 3. Release notes

- [x] 3.1 Note in the release notes for the next tag that episode ordering in feeds now matches YouTube's exact upload order (using precise timestamps), pubDate follows RFC 2822, episode descriptions include a YouTube video link, the feed is again available at the legacy `/{slug}/feed.xml` URL, and unknown feed slugs return `404 Not Found`.
