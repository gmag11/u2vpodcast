# Design

## Context

See `proposal.md` - Why.

- `src/handlers/feed.rs` builds every item through `episode_item(url, audio_root, slug, episode, title, sponsorblock_enabled)`, mapped by `channel_items` (per-channel feed and its legacy aliases) and `global_items` (aggregated feed). The function already builds `<enclosure>` from the selected representation and `<guid>` from `yt_id`; it sets no `<link>`. `build_feed` and `get_global_feed` have `Data<AppState>`, so they hold `config.url` and the share secret.
- `add-public-episode-share` provides the permanent, deterministic share token: `hash = HMAC-SHA256(share_secret, "u2vpodcast/share/v1|" + webpage_url)` and `token = "{yt_id}-{hex(hash)}"`, served at `{url}/app/share/{token}`. The `Episode` model exposes `webpage_url` and `yt_id`.
- The feed's `<enclosure>` stays the protected stable media URL, and the feed is wrapped in `SessionOrBasicAuth`; this change does not alter either.

## Goals / Non-Goals

**Goals:**

- Give every feed item a discoverable public episode page without changing the enclosure, guid, ordering, or feed protection.
- Keep the feed byte-stable so the existing legacy-alias identity guarantee holds.

**Non-Goals:**

- Making the audio enclosure public or changing `<guid>`.
- A per-episode feed.
- Any new configuration or persistence.
- Changing share-token lifetime or revocation.

## Decisions

### Decision: Set `<link>` from the deterministic share token

Each `episode_item` sets `<link>` to `format!("{url}/app/share/{}", share_token(share_secret, &episode.webpage_url, &episode.yt_id))`, reusing the share module's deterministic derivation. The share secret is threaded from `build_feed`/`get_global_feed` through `channel_items`/`global_items` into `episode_item` (one extra borrowed argument, or a small borrowed config struct).

Because the token is a deterministic function of `(share_secret, webpage_url, yt_id)`, `episode_item` produces the same `<link>` on every build. No expiry is embedded, so the feed body is byte-stable and the `rss-feeds` legacy-alias identity guarantee is preserved without modification.

Alternatives considered:

- **Mint a per-request token with an expiry** (the earlier draft). Rejected: it made the feed non-deterministic and forced relaxing the legacy-alias spec. The deterministic, permanent token removes both problems.
- **`podcast:contentLink` instead of `<link>`.** Rejected: `<link>` is the RSS 2.0 episode page understood by every client and needs no extra namespace semantics.

### Decision: Keep enclosure, guid, pubDate, description, and protection untouched

The item builders only gain `<link>`. Ordering and `<guid>` are unchanged, so already-subscribed clients keep matching the same episodes.

## Risks / Trade-offs

- **Permanent capability visible to subscribers** → the feed already requires credentials, so exposure is limited to authenticated subscribers; tokens are not logged.
- **A changed `webpage_url` invalidates the link** → `webpage_url` is the episode's stable canonical URI.
- **Existing item-builder tests pass fewer arguments** → updated in the same change so the feed module's tests keep compiling and assert the new `<link>`.

## Migration Plan

- No database migration and no configuration change; the change is additive.
- Deploy the new build; previously fetched feeds keep working (only a new, stable `<link>` appears).
- Rollback: remove the `<link>` assignment; no state to clean up.

## Open Questions

None that affect the specs, approach, or tasks.
