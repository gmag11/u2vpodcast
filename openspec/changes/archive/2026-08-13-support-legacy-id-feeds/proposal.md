## Why

The switch from numeric channel ids to slugs changed feed URLs from `/channels/{id}/feed.xml` to `/channels/{slug}/feed.xml`. Podcast clients subscribed with the old URLs now fail to resolve their feeds, forcing every existing subscriber to manually update all feeds after an upgrade.

## What Changes

- The `/channels/{key}/feed.xml` route accepts a numeric channel id in place of the slug, resolving the channel by id when the path segment is numeric and by slug otherwise. The short `/{key}/feed.xml` alias is untouched: it only ever served slugs and keeps doing so.
- The canonical feed served by the application remains the slug-based URL; the id-based URL acts purely as a backward-compatibility alias.
- Enclosure URLs in feeds served through the id-based alias point at the channel's canonical slug media directory, so downloaded episodes keep matching the on-disk audio directories after migration.
- A feed requested through the id-based alias returns the identical RSS document as the same channel's slug-based URL.

## Capabilities

### New Capabilities

### Modified Capabilities

- `rss-feeds`: feed URLs must accept a legacy numeric channel id in addition to the slug, resolving to the same channel and returning an identical feed.

## Impact

- `src/handlers/feed.rs`: resolve the feed path segment with `Channel::read_by_id_or_slug` and build enclosure URLs from the resolved channel's canonical slug instead of the raw path segment.
- `src/models/channel.rs`: reuse the existing `read_by_id_or_slug` resolver (no new capability).
- Podcast clients subscribed to `/channels/{id}/feed.xml` keep receiving updates without re-subscribing.
- Legacy media URLs (`/media/{id}/{yt_id}.mp3`) are intentionally out of scope; only the feed routes change.