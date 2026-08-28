## Purpose

Generates each channel's RSS feed so it contains exclusively the episodes of that channel, newest first, with enclosure URLs that resolve to the channel's own media files.

## Requirements

### Requirement: A feed contains only its own channel's episodes

The system SHALL generate `/channels/{slug}/feed.xml` with `<item>` entries only for episodes belonging to the channel whose `slug` equals the requested `slug`. Episodes belonging to any other channel MUST NOT appear in the feed. The channel is resolved by slug (not by numeric id).

#### Scenario: Feed with mixed episodes is filtered
- **WHEN** the database contains episodes for two channels with slugs `confesiones_de_gasolinera` and `linux_y_tapas`, and a client requests `/channels/confesiones_de_gasolinera/feed.xml` with valid credentials
- **THEN** the RSS body contains `<item>` entries only for `confesiones_de_gasolinera`'s episodes, and no `<item>` for any `linux_y_tapas` episode

#### Scenario: Feed of a channel with no episodes
- **WHEN** a channel has no episodes and a client requests its feed by slug
- **THEN** the system responds `200 OK` with a valid RSS document whose `<item>` list is empty

### Requirement: Feed items are ordered newest first

The `<item>` entries in a feed SHALL be ordered by episode `published_at` descending, so the most recent episode appears first.

#### Scenario: Episodes ordered by publish date
- **WHEN** a channel has episodes published at different dates and a client requests its feed
- **THEN** the `<item>` entries appear in order of `published_at` descending (most recent first)

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

### Requirement: Episodes are ordered by precise YouTube timestamp

The `<item>` entries in a feed SHALL be ordered by the YouTube video's Unix epoch `timestamp` (second precision) descending, so the most recent episode appears first and episodes published on the same day are ordered in the exact sequence they appeared on YouTube.

#### Scenario: Same-day episodes respect YouTube order
- **WHEN** a YouTube channel publishes three videos on the same day at 10:00, 14:00, and 18:00 UTC, and the system processes them
- **THEN** the feed `<item>` entries appear in order 18:00 first, 14:00 second, 10:00 third

#### Scenario: Timestamp fallback to upload_date
- **WHEN** yt-dlp returns a video without a `timestamp` field (only `upload_date`)
- **THEN** the system falls back to parsing `upload_date` as midnight UTC for that episode

### Requirement: pubDate uses RFC 2822 format

The `<pubDate>` element of each `<item>` in a channel's RSS feed SHALL be formatted according to RFC 2822 (e.g., `"Fri, 15 Mar 2024 14:30:00 +0000"`).

#### Scenario: pubDate is RFC 2822 compliant
- **WHEN** a client requests `/channels/{slug}/feed.xml` for a channel that has episodes
- **THEN** every `<item>` contains a `<pubDate>` element whose value matches the RFC 2822 date-time format (`ddd, DD MMM YYYY HH:MM:SS +0000`)

#### Scenario: pubDate reflects the original YouTube timestamp
- **WHEN** a YouTube video was published at Unix timestamp `1710509400` (March 15, 2024 14:30:00 UTC) and the system creates the episode
- **THEN** the `<item>`'s `<pubDate>` is `"Fri, 15 Mar 2024 14:30:00 +0000"` in RFC 2822 format

### Requirement: Episode description starts with the YouTube video link

The `<description>` and `<itunes:summary>` of each `<item>` in a channel's RSS feed SHALL begin with the YouTube video URL (`webpage_url`) followed by a blank line, placing the original video description below the link.

#### Scenario: Description begins with YouTube link
- **WHEN** a client requests `/channels/{slug}/feed.xml` for a channel that has episodes
- **THEN** for every `<item>`, both `<description>` and `<itunes:summary>` start with `https://www.youtube.com/watch?v={yt_id}` followed by a blank line, then the video description

#### Scenario: Link is clickable from the podcast client
- **WHEN** a podcast app renders an episode description that begins with a plain YouTube URL
- **THEN** the URL is tappable/clickable, allowing the user to navigate directly to the original video on YouTube

### Requirement: Feed is served at the legacy URL

The system SHALL serve a channel's RSS feed both at `/channels/{slug}/feed.xml` and at the legacy `/{slug}/feed.xml`, returning the identical feed for the same channel from both URLs so podcast clients subscribed before the URL scheme changed keep receiving updates. Both routes SHALL apply the same access protection.

#### Scenario: Legacy URL returns the same feed
- **WHEN** a client requests `/{slug}/feed.xml` for a channel that has episodes
- **THEN** the system responds `200 OK` with an RSS document whose `<item>` entries are identical to those returned by `/channels/{slug}/feed.xml` for the same channel

#### Scenario: Canonical URL still works
- **WHEN** a client requests `/channels/{slug}/feed.xml` for a channel that has episodes
- **THEN** the system responds `200 OK` with the channel's RSS feed, unchanged by the addition of the legacy alias

#### Scenario: Legacy URL resolves the channel by slug
- **WHEN** a client requests `/{slug}/feed.xml` with a slug that does not match any channel
- **THEN** the system responds `404 Not Found`, the same as `/channels/{slug}/feed.xml` for an unknown slug

#### Scenario: Unknown channel slug returns 404
- **WHEN** a client requests `/channels/{slug}/feed.xml` or `/{slug}/feed.xml` with a slug that does not match any channel
- **THEN** the system responds `404 Not Found` instead of a generic server error

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

### Requirement: Feed duration matches the selected media representation
Each feed item's iTunes duration SHALL describe the actual selected enclosure. A processed enclosure SHALL use its measured processed duration; an original enclosure SHALL use the episode's original duration.

#### Scenario: Processed enclosure publishes processed duration
- **WHEN** a processed MP3 measured at 540 seconds is selected for an episode whose original duration is 600 seconds
- **THEN** the feed item publishes an iTunes duration representing 540 seconds

#### Scenario: Original enclosure publishes original duration
- **WHEN** an episode has no active processed MP3
- **THEN** its feed item retains the original episode duration
