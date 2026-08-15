## Context

Per-channel RSS feeds are generated in `src/handlers/feed.rs` (`web_feed`, `get_feed`, `get_legacy_feed`, `build_feed`) and served at `/channels/{key}/feed.xml` and `/{key}/feed.xml`, protected by the `SessionOrBasicAuth` middleware so podcast clients can subscribe with basic auth. `web_feed` is configured at the root scope in `src/handlers/mod.rs`.

The model already exposes `Episode::read_all_with_channels(pool)` (added by the `all-episodes-api` capability), which returns every episode across channels joined with `channel_slug` and `channel_title` (via `COALESCE`), ordered by `published_at` DESC. The history screen (`HistoryView.vue`) already renders that cross-channel list, and `ChannelCard.vue` shows the per-channel feed link using `baseEndpoint` plus an RSS icon.

## Goals / Non-Goals

**Goals:**
- Serve one aggregated RSS document with every episode from every channel, newest first.
- Protect the endpoint with the same `SessionOrBasicAuth` middleware used by per-channel feeds.
- Expose a download link with the RSS icon on the history screen.

**Non-Goals:**
- Changing per-channel feed behavior or URLs.
- New auth model, DB changes, or migrations.
- Server-side pagination or search of the feed.
- Per-channel grouping inside the feed (single flat publication-order list).

## Decisions

### Decision 1: Serve the global feed at `GET /feed.xml`

Add a handler `get_global_feed` in `src/handlers/feed.rs` and register `web::resource("/feed.xml").route(web::get().to(get_global_feed)).wrap(SessionOrBasicAuth)` inside `web_feed`.

**Why**: `/feed.xml` cannot collide with the existing `/{key}/feed.xml` route, which requires a path segment before `/feed.xml`. Keeping it inside `web_feed` groups all feeds under the same middleware and outside the `/api` scope, so podcast clients can subscribe the same way they do per-channel feeds.

**Alternative considered**: `/api/1.0/feed.xml`. Rejected: existing feeds live outside `/api` for client friendliness; the global feed should behave identically.

### Decision 2: Reuse `Episode::read_all_with_channels`

The global feed calls the existing `Episode::read_all_with_channels(pool)`, which already returns every episode across channels with `channel_slug`/`channel_title`, ordered newest first.

**Why**: no new query or model method is needed; the ordering requirement (publication order, newest first) is exactly what this method returns.

### Decision 3: Enclosure from `channel_slug`, title prefixed with `channel_title`

For each item, build `<enclosure>` as `{url}/media/{slug}/{yt_id}.mp3` from the episode's own `channel_slug`, and prefix the item `<title>` with `channel_title` (e.g. `Confesiones de Gasolinera: Episodio 10`). Episodes whose `channel_slug` is empty are skipped so the feed never emits an enclosure pointing at a non-existent audio directory.

**Why**: the on-disk audio directory is named by slug, so the enclosure must be built from each episode's own `channel_slug` (per-channel feeds use the requested channel's slug; here there is no single requested channel). The title prefix disambiguates episodes in a merged feed.

### Decision 4: History screen link with the RSS icon

In `HistoryView.vue`, add an RSS icon link in the header row (next to the title) pointing at `${baseEndpoint}/feed.xml`, mirroring the `ChannelCard.vue` feed link (icon + tooltip).

**Why**: the history screen already presents the aggregated view, making it the most discoverable place for the aggregated feed; reusing the existing icon/link pattern keeps the UI consistent.

## Risks / Trade-offs

- **[Risk] Large feed.** The merged feed grows with the total episode count. → Mitigation: same query the history screen already runs; RSS clients handle large documents; pagination can be added later without a spec change.
- **[Risk] Episode without a channel.** `read_all_with_channels` coalesces missing channel fields to `""`, which would produce a broken enclosure. → Mitigation: skip items whose `channel_slug` is empty.
- **[Trade-off] Title prefix.** Item titles in the global feed carry the channel prefix, unlike per-channel feeds. → Per-channel feeds are unchanged; the prefix is only in the aggregated document.

## Migration Plan

1. Backend: add `get_global_feed` to `src/handlers/feed.rs` and register `/feed.xml` in `web_feed`. `cargo build` to confirm.
2. Frontend: add the RSS link to `frontend/src/views/HistoryView.vue` using `baseEndpoint`. `pnpm build` to confirm.
3. Manually verify: request `/feed.xml` with credentials and confirm every channel's episodes appear newest first, with correct `channel_slug`-based enclosures and prefixed titles; open the history screen and confirm the RSS link is visible and resolves the feed.

**Rollback**: revert the handler/route and the frontend link; no DB, config, or dependency changes exist.

## Open Questions

None.
