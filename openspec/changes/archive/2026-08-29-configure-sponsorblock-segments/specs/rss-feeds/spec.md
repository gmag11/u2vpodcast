## MODIFIED Requirements

### Requirement: Feed enclosure URLs point at the channel's own media

For each `<item>`, the system SHALL select the active processed SponsorBlock MP3 when SponsorBlock is enabled and one exists; otherwise it SHALL select the original MP3. The `<enclosure>` URL SHALL be `{url}/media/{slug}/{selected_filename}` where `slug` is the episode's canonical channel slug. An original filename SHALL be `{yt_id}.mp3`; a processed filename SHALL be `{yt_id}.sponsorblock.{hash-prefix}.mp3`. Every enclosure URL SHALL correspond to an existing file owned by that episode.

#### Scenario: Enclosure matches the item's channel
- **WHEN** SponsorBlock is enabled and a feed contains episode `abc123` with active processed file `abc123.sponsorblock.a81f302c.mp3`
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.sponsorblock.a81f302c.mp3`

#### Scenario: Enclosure falls back to the original episode
- **WHEN** a feed contains episode `abc123` without an eligible active processed file
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.mp3`

#### Scenario: Legacy id feed emits the canonical selected enclosure
- **WHEN** a client requests `/channels/3/feed.xml` for a channel whose canonical slug is `confesiones_de_gasolinera`
- **THEN** every enclosure uses `/media/confesiones_de_gasolinera/{selected_filename}` rather than the numeric id

#### Scenario: Disabled SponsorBlock ignores processed media
- **WHEN** SponsorBlock is disabled and episode `abc123` has an existing processed file
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.mp3`