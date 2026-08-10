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

For each `<item>`, the `<enclosure>` URL SHALL be `{url}/media/{slug}/{yt_id}.mp3` where `slug` is the requested channel's slug and `yt_id` is that item's own episode identifier. Because items are filtered by channel and the audio directory is named by slug, every enclosure URL corresponds to a real media file for that channel.

#### Scenario: Enclosure matches the item's channel
- **WHEN** a client requests `/channels/confesiones_de_gasolinera/feed.xml` and the feed contains an episode with `yt_id` `abc123`
- **THEN** that episode's `<enclosure>` URL is `{url}/media/confesiones_de_gasolinera/abc123.mp3`

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
