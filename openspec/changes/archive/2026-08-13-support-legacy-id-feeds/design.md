## Context

Feed URLs moved from numeric channel ids to slugs (`/channels/{slug}/feed.xml` plus the legacy `/{slug}/feed.xml` alias). Existing podcast clients still subscribed to `/channels/{id}/feed.xml` receive `404 Not Found` after the upgrade because the handler only resolves channels by slug. The application serves only the slug-based feed as canonical; the id-based URL is a compatibility concern.

## Goals / Non-Goals

**Goals:**
- Accept a numeric channel id in the feed path segment of `/channels/{key}/feed.xml`.
- Return an identical RSS document regardless of whether the channel was addressed by id or by slug.
- Keep the slug-based feed as the only canonical URL served by the application.

**Non-Goals:**
- Serving legacy media URLs (`/media/{id}/{yt_id}.mp3`) — out of scope; media directories are renamed to slugs.
- Redirecting id-based requests to the canonical slug URL (the RSS body is served directly, no HTTP redirect).
- Advertising or linking the id-based feed anywhere in the UI.

## Decisions

### D1: Resolve the path key with `read_by_id_or_slug`

The handler parses the `/channels/{key}/feed.xml` path segment as a `String`, and resolves the channel with the existing `Channel::read_by_id_or_slug` helper: numeric keys resolve by `id`, anything else by `slug`. The short `/{slug}/feed.xml` route keeps resolving by slug only.

**Rationale**: a single handler keeps route registration unchanged and reuses an already-tested resolver. A typed `{id: i64}` route was considered but rejected: it would need separate route registrations and would reject non-numeric segments with `400`, duplicating logic already present in `read_by_id_or_slug`.

**Trade-off**: a channel whose slug consists only of digits (e.g. a title that slugifies to `2024`) would be shadowed by the id lookup. Accepted: slugs are derived from transliterated titles with a non-numeric `channel-{id}` fallback, so pure-digit slugs are effectively unreachable in practice.

### D2: Build enclosure URLs from the resolved channel's canonical slug

Enclosure URLs use `channel.slug` (the resolved row) instead of the raw path segment, so a feed requested via `/channels/3/feed.xml` emits `/media/confesiones_de_gasolinera/{yt_id}.mp3`.

**Rationale**: after migration the audio directories are named by slug; using the raw numeric path segment would produce enclosure URLs pointing at non-existent directories and break downloads. Using the canonical slug makes legacy-subscribed clients self-heal: the first feed refresh rewrites every stored enclosure to the working slug URL.

**Alternative considered**: keep the raw path key in enclosures. Rejected — produces broken media URLs for every id-based subscriber.

## Risks / Trade-offs

- [Pure-digit slug shadowed by id lookup] → Mitigation: slug fallback format `channel-{id}` is non-numeric; documented as an accepted edge case.
- [id no longer exists after migration / stale client] → Mitigation: resolver returns `404 Not Found`, consistent with the unknown-slug behavior.
- [Legacy media URLs still broken for clients that cached them] → Mitigation: out of scope by design; feed enclosures regenerate with canonical slugs on first fetch, so clients self-correct.
