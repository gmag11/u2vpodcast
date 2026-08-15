## ADDED Requirements

### Requirement: Global feed aggregates every episode newest first

The system SHALL serve a protected RSS document at `/feed.xml` containing every episode from every channel in a single feed, ordered by `published_at` descending (newest first). The endpoint SHALL be protected by the same `SessionOrBasicAuth` middleware used by the per-channel feeds.

#### Scenario: Authenticated request returns all episodes
- **WHEN** a client with valid credentials requests `/feed.xml`
- **THEN** the response is `200 OK` with an RSS document whose `<item>` entries include every episode from every channel, ordered newest first

#### Scenario: Unauthenticated request is rejected
- **WHEN** a request without valid credentials hits `/feed.xml`
- **THEN** the request is rejected by the `SessionOrBasicAuth` middleware and the feed is not returned

#### Scenario: Empty database returns an empty feed
- **WHEN** no episodes exist and a client requests `/feed.xml`
- **THEN** the system responds `200 OK` with a valid RSS document whose `<item>` list is empty

### Requirement: Global feed items carry a channel-distinct enclosure and title

Each `<item>` in the global feed SHALL have an `<enclosure>` URL of `{url}/media/{slug}/{yt_id}.mp3` where `slug` is the owning channel's slug and `yt_id` is that item's own episode identifier, and SHALL prefix the item title with the owning channel's title so episodes are distinguishable in the aggregated feed.

#### Scenario: Enclosure resolves to the owning channel's media
- **WHEN** an episode of channel with slug `confesiones_de_gasolinera` and `yt_id` `abc123` appears in `/feed.xml`
- **THEN** the episode's `<enclosure>` URL is `{url}/media/confesiones_de_gasolinera/abc123.mp3`

#### Scenario: Title is prefixed with the channel
- **WHEN** an episode titled `Episodio 10` belongs to a channel titled `Confesiones de Gasolinera`
- **THEN** the item's `<title>` is `Confesiones de Gasolinera: Episodio 10`

#### Scenario: Episode without a channel is excluded
- **WHEN** an episode's `channel_slug` resolves to empty
- **THEN** the episode does not produce a broken `<item>` and is skipped

### Requirement: History screen exposes the global feed download

The history screen SHALL render a download link with an RSS icon pointing at the global feed URL, so users can download or subscribe to the aggregated feed directly from the cross-channel view.

#### Scenario: Link is present on the history screen
- **WHEN** a user opens the history screen
- **THEN** a link with an RSS icon pointing at `{baseEndpoint}/feed.xml` is visible in the header area of the screen
