# Proposal

## Why

The public episode-share feature mints a permanent, unguessable link that plays an episode in a browser, but that link is only reachable from the episode card of an authenticated user. A feed item currently has no `<link>`, so a subscriber browsing the feed in a podcast client has no way to open the episode's public page. Setting `<link>` on every item to the episode's deterministic share page makes each episode reachable from the feed without weakening the feed's own protection and without making the feed body unstable.

## What Changes

- Every feed `<item>` (per-channel, legacy slug alias, legacy numeric-id alias, and the global feed) SHALL set `<link>` to `{url}/app/share/{token}`, where the token is the episode's permanent, deterministic share token (`{yt_id}-{hash(share_secret, "v1|{webpage_url}")}`).
- Because the token is deterministic, the same feed build yields the same `<link>` every time. The feed body stays byte-stable across requests and the existing "identical feed from the legacy URL alias" guarantee is preserved.
- The `<enclosure>`, `<guid>`, `<pubDate>`, ordering, description/summary, and access protection of the feeds SHALL remain unchanged. The `<enclosure>` stays the protected stable media URL; the feed continues to require credentials.
- No new configuration, no database change, and no new dependency. This change builds on the `add-public-episode-share` token and public page.

Non-goals: making the feed itself public; changing the enclosure or the `<guid>`; a per-episode RSS feed; any change to share-token lifetime or revocation.

## Capabilities

### New Capabilities
<!-- None: this change reuses the existing share-token and public-page behavior. -->

### Modified Capabilities
- `rss-feeds`: each item gains a deterministic public share `<link>`.
- `global-feed`: each aggregated item also gains the deterministic public share `<link>`.

## Impact

- Code: `src/handlers/feed.rs` (thread the share secret into the item builders and set `<link>`), `README.md` (document that feed items link to a public page).
- Tests: the feed module's item tests gain link assertions; existing direct calls to the item builders are updated for the new arguments.
- Specs: `openspec/specs/rss-feeds`, `openspec/specs/global-feed`.
- No database migration, no API payload change, no new dependency, and no change to the feed's credential protection.
- Depends on `add-public-episode-share` for the share-token derivation and the `/app/share/{token}` page.
