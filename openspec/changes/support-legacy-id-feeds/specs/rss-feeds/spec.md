## ADDED Requirements

### Requirement: Feed is served at the legacy id URL

The system SHALL resolve the feed path segment in `/channels/{key}/feed.xml` as a channel id when the segment consists only of digits, and as a channel slug otherwise. The short `/{slug}/feed.xml` alias SHALL keep resolving by slug only. When the segment is a numeric id, the system SHALL return the identical RSS document as the same channel's slug-based URL, with `<enclosure>` URLs built from the channel's canonical slug so episodes resolve to the on-disk audio directory.

#### Scenario: Legacy id URL returns the same feed as the slug URL

- **WHEN** a client requests `/channels/3/feed.xml` and the channel with `id` `3` has slug `confesiones_de_gasolinera`
- **THEN** the system responds `200 OK` with an RSS document whose `<item>` entries are identical to those returned by `/channels/confesiones_de_gasolinera/feed.xml`

#### Scenario: Legacy id URL emits canonical slug enclosures

- **WHEN** a client requests `/channels/3/feed.xml` for a channel whose slug is `confesiones_de_gasolinera` and the feed contains an episode with `yt_id` `abc123`
- **THEN** that episode's `<enclosure>` URL is `{url}/media/confesiones_de_gasolinera/abc123.mp3`, using the canonical slug and not the numeric id

#### Scenario: Unknown numeric id returns 404

- **WHEN** a client requests `/channels/999/feed.xml` and no channel with `id` `999` exists
- **THEN** the system responds `404 Not Found`, the same as for an unknown slug

#### Scenario: Non-numeric segment still resolves by slug

- **WHEN** a client requests `/channels/confesiones_de_gasolinera/feed.xml`
- **THEN** the system responds `200 OK` with that channel's feed, unchanged by the legacy id alias
