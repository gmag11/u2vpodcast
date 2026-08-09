## Purpose

Generates each channel's RSS feed so it contains exclusively the episodes of that channel, newest first, with enclosure URLs that resolve to the channel's own media files.

## Requirements

### Requirement: A feed contains only its own channel's episodes

The system SHALL generate `/channels/{channel_id}/feed.xml` with `<item>` entries only for episodes whose `channel_id` equals the requested `channel_id`. Episodes belonging to any other channel MUST NOT appear in the feed.

#### Scenario: Feed with mixed episodes is filtered
- **WHEN** the database contains episodes for channel 1 and channel 2, and a client requests `/channels/1/feed.xml` with valid credentials
- **THEN** the RSS body contains `<item>` entries only for channel 1's episodes, and no `<item>` for any channel 2 episode

#### Scenario: Feed of a channel with no episodes
- **WHEN** a channel has no episodes and a client requests its feed
- **THEN** the system responds `200 OK` with a valid RSS document whose `<item>` list is empty

### Requirement: Feed items are ordered newest first

The `<item>` entries in a feed SHALL be ordered by episode `published_at` descending, so the most recent episode appears first.

#### Scenario: Episodes ordered by publish date
- **WHEN** a channel has episodes published at different dates and a client requests its feed
- **THEN** the `<item>` entries appear in order of `published_at` descending (most recent first)

### Requirement: Feed enclosure URLs point at the channel's own media

For each `<item>`, the `<enclosure>` URL SHALL be `{url}/media/{channel_id}/{yt_id}.mp3` where `channel_id` is the requested channel and `yt_id` is that item's own episode identifier. Because items are filtered by channel, every enclosure URL corresponds to a real media file for that channel.

#### Scenario: Enclosure matches the item's channel
- **WHEN** a client requests `/channels/1/feed.xml` and the feed contains an episode with `yt_id` `abc123`
- **THEN** that episode's `<enclosure>` URL is `{url}/media/1/abc123.mp3`
