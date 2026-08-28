## MODIFIED Requirements

### Requirement: Global feed items carry a channel-distinct enclosure and title

Each `<item>` in the global feed SHALL select the active processed SponsorBlock MP3 when one exists and otherwise SHALL select the original MP3. Its `<enclosure>` URL SHALL be `{url}/media/{slug}/{selected_filename}` where `slug` is the owning channel's slug, and its iTunes duration SHALL describe that selected file. The item title SHALL remain prefixed with the owning channel's title so episodes are distinguishable in the aggregated feed.

#### Scenario: Enclosure resolves to the owning channel's media
- **WHEN** episode `abc123` of channel `confesiones_de_gasolinera` selects processed file `abc123.sponsorblock.a81f302c.mp3`
- **THEN** its enclosure is `{url}/media/confesiones_de_gasolinera/abc123.sponsorblock.a81f302c.mp3` and its iTunes duration is the measured processed duration

#### Scenario: Original enclosure resolves to the owning channel's media
- **WHEN** episode `abc123` of channel `confesiones_de_gasolinera` has no active processed file
- **THEN** its enclosure is `{url}/media/confesiones_de_gasolinera/abc123.mp3` and its iTunes duration is the original episode duration

#### Scenario: Title is prefixed with the channel
- **WHEN** an episode titled `Episodio 10` belongs to a channel titled `Confesiones de Gasolinera`
- **THEN** the item's `<title>` is `Confesiones de Gasolinera: Episodio 10`

#### Scenario: Episode without a channel is excluded
- **WHEN** an episode's `channel_slug` resolves to empty
- **THEN** the episode does not produce a broken `<item>` and is skipped