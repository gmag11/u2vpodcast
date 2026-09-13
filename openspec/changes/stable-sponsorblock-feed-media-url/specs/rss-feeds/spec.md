## MODIFIED Requirements

### Requirement: Feed enclosure URLs point at the channel's own media

For each `<item>`, the system SHALL set the `<enclosure>` URL to the stable `{url}/media/{slug}/{yt_id}.mp3`, where `slug` is the episode's canonical channel slug, regardless of whether SponsorBlock is enabled or a processed derivative exists. The system SHALL set the `<enclosure>` `length` attribute to the byte size of the representation served at that URL: the active processed MP3 when SponsorBlock is enabled and a valid derivative exists, otherwise the original MP3. The enclosure URL SHALL resolve to a representation owned by that episode.

#### Scenario: Enclosure matches the item's channel
- **WHEN** SponsorBlock is enabled and a feed contains episode `abc123` with active processed file `abc123.sponsorblock.a81f302c.mp3`
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.mp3` using the item's canonical channel slug

#### Scenario: Enclosure falls back to the original episode
- **WHEN** a feed contains episode `abc123` without an eligible active processed file
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.mp3` and its reported length is the original file's byte size

#### Scenario: Processed representation reports its byte size
- **WHEN** SponsorBlock is enabled, episode `abc123` has an existing active processed file of 24,605,805 bytes, and that file is the representation served at the stable URL
- **THEN** its `<enclosure>` `length` is `24605805`

#### Scenario: Legacy id feed emits the canonical selected enclosure
- **WHEN** a client requests `/channels/3/feed.xml` for a channel whose canonical slug is `confesiones_de_gasolinera`
- **THEN** every enclosure uses `/media/confesiones_de_gasolinera/{yt_id}.mp3` rather than the numeric id or a processed filename

#### Scenario: Disabled SponsorBlock ignores processed media
- **WHEN** SponsorBlock is disabled and episode `abc123` has an existing processed file
- **THEN** its enclosure URL is `{url}/media/{slug}/abc123.mp3` and its reported length is the original file's size

### Requirement: Feed duration matches the selected media representation

Each feed item's iTunes duration SHALL describe the representation served at the episode's stable enclosure URL. A processed representation SHALL use its measured processed duration; an original representation SHALL use the episode's original duration.

#### Scenario: Processed enclosure publishes processed duration
- **WHEN** a processed MP3 measured at 540 seconds is served at the stable enclosure URL for an episode whose original duration is 600 seconds
- **THEN** the feed item publishes an iTunes duration representing 540 seconds

#### Scenario: Original enclosure publishes original duration
- **WHEN** an episode has no active processed MP3
- **THEN** its feed item retains the original episode duration
