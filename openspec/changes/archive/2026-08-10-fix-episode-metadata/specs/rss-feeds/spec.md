## Purpose

Augments the RSS feed generation capability (see `openspec/specs/rss-feeds/spec.md`) with four new requirements: precise episode ordering using YouTube timestamps, emitting `pubDate` in RFC 2822 format, including a YouTube video link at the top of each episode description, and keeping the feed reachable at the legacy URL for backwards compatibility.

## ADDED Requirements

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
