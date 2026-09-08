## MODIFIED Requirements

### Requirement: Feed enclosure URLs point at the channel's own media
For each `<item>`, the system SHALL select the active processed SponsorBlock MP3 when one exists and otherwise SHALL select the original MP3. The `<enclosure>` URL SHALL be `{url}/media/{slug}/{selected_filename}` where `slug` is the episode's canonical channel slug. An original filename SHALL be `{yt_id}.mp3`; a processed filename SHALL be `{yt_id}.sponsorblock.{hash-prefix}.mp3`. Every enclosure URL SHALL correspond to an existing file owned by that episode.

#### Scenario: Enclosure matches the item's channel
- **WHEN** a feed contains episode `abc123` with active processed file `abc123.sponsorblock.a81f302c.mp3`
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.sponsorblock.a81f302c.mp3`

#### Scenario: Enclosure falls back to the original episode
- **WHEN** a feed contains episode `abc123` without an active processed file
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.mp3`

#### Scenario: Legacy id feed emits the canonical selected enclosure
- **WHEN** a client requests `/channels/3/feed.xml` for a channel whose canonical slug is `confesiones_de_gasolinera`
- **THEN** every enclosure uses `/media/confesiones_de_gasolinera/{selected_filename}` rather than the numeric id

### Requirement: Feed is served at the legacy id URL

The system SHALL resolve the feed path segment in `/channels/{key}/feed.xml` as a channel id when the segment consists only of digits, and as a channel slug otherwise. The short `/{slug}/feed.xml` alias SHALL keep resolving by slug only. When the segment is a numeric id, the system SHALL return the identical RSS document as the same channel's slug-based URL, with `<enclosure>` URLs built from the channel's canonical slug and selected original or processed media filename.

#### Scenario: Legacy id URL returns the same feed as the slug URL

- **WHEN** a client requests `/channels/3/feed.xml` and the channel with `id` `3` has slug `confesiones_de_gasolinera`
- **THEN** the system responds `200 OK` with an RSS document whose `<item>` entries are identical to those returned by `/channels/confesiones_de_gasolinera/feed.xml`

#### Scenario: Legacy id URL emits canonical slug enclosures

- **WHEN** a client requests `/channels/3/feed.xml` for a channel whose slug is `confesiones_de_gasolinera` and an episode selects `{selected_filename}`
- **THEN** that episode's `<enclosure>` URL is `{url}/media/confesiones_de_gasolinera/{selected_filename}`, using the canonical slug and not the numeric id

#### Scenario: Unknown numeric id returns 404

- **WHEN** a client requests `/channels/999/feed.xml` and no channel with `id` `999` exists
- **THEN** the system responds `404 Not Found`, the same as for an unknown slug

#### Scenario: Non-numeric segment still resolves by slug

- **WHEN** a client requests `/channels/confesiones_de_gasolinera/feed.xml`
- **THEN** the system responds `200 OK` with that channel's feed, unchanged by the legacy id alias

## ADDED Requirements

### Requirement: Feed duration matches the selected media representation
Each feed item's iTunes duration SHALL describe the actual selected enclosure. A processed enclosure SHALL use its measured processed duration; an original enclosure SHALL use the episode's original duration.

#### Scenario: Processed enclosure publishes processed duration
- **WHEN** a processed MP3 measured at 540 seconds is selected for an episode whose original duration is 600 seconds
- **THEN** the feed item publishes an iTunes duration representing 540 seconds

#### Scenario: Original enclosure publishes original duration
- **WHEN** an episode has no active processed MP3
- **THEN** its feed item retains the original episode duration