## MODIFIED Requirements

### Requirement: A feed contains only its own channel's episodes

The system SHALL generate `/channels/{slug}/feed.xml` with `<item>` entries only for episodes belonging to the channel whose `slug` equals the requested `slug`. Episodes belonging to any other channel MUST NOT appear in the feed. The channel is resolved by slug (not by numeric id).

#### Scenario: Feed with mixed episodes is filtered
- **WHEN** the database contains episodes for two channels with slugs `confesiones_de_gasolinera` and `linux_y_tapas`, and a client requests `/channels/confesiones_de_gasolinera/feed.xml` with valid credentials
- **THEN** the RSS body contains `<item>` entries only for `confesiones_de_gasolinera`'s episodes, and no `<item>` for any `linux_y_tapas` episode

#### Scenario: Feed of a channel with no episodes
- **WHEN** a channel has no episodes and a client requests its feed by slug
- **THEN** the system responds `200 OK` with a valid RSS document whose `<item>` list is empty

### Requirement: Feed enclosure URLs point at the channel's own media

For each `<item>`, the `<enclosure>` URL SHALL be `{url}/media/{slug}/{yt_id}.mp3` where `slug` is the requested channel's slug and `yt_id` is that item's own episode identifier. Because items are filtered by channel and the audio directory is named by slug, every enclosure URL corresponds to a real media file for that channel.

#### Scenario: Enclosure matches the item's channel
- **WHEN** a client requests `/channels/confesiones_de_gasolinera/feed.xml` and the feed contains an episode with `yt_id` `abc123`
- **THEN** that episode's `<enclosure>` URL is `{url}/media/confesiones_de_gasolinera/abc123.mp3`