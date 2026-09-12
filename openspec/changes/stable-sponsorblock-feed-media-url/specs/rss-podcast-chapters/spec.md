## MODIFIED Requirements

### Requirement: Episode chapters are exposed as Podcasting 2.0 JSON
The system SHALL expose a JSON chapters endpoint per episode following the Podcasting 2.0 JSON Chapters 1.2 schema: a top-level object with `version` set to `"1.2.0"` and a `chapters` array of entries, each with `startTime` (seconds) and `title`. The endpoint SHALL use the `application/json+chapters` content type. When the representation served at the episode's stable media URL is the original MP3, the endpoint SHALL return the episode's stored, untranslated chapters. When the representation served at the stable media URL is a SponsorBlock-processed MP3, the endpoint SHALL return the same translated chapters embedded into that processed file. An episode with no stored chapters SHALL return a successful response with an empty `chapters` array.

#### Scenario: Original enclosure selected
- **WHEN** the stable media URL serves the original MP3 and the episode has stored chapters
- **THEN** the chapters endpoint returns the original, untranslated start times and titles

#### Scenario: SponsorBlock has no rejected segments
- **WHEN** SponsorBlock is enabled but an episode has no effective rejected segments and therefore serves the original MP3
- **THEN** the chapters endpoint returns the original, untranslated start times and titles

#### Scenario: Processed enclosure selected
- **WHEN** the stable media URL serves a SponsorBlock-processed MP3
- **THEN** the chapters endpoint returns the same translated chapters embedded into that processed file, not the original untranslated chapters

#### Scenario: Episode has no stored chapters
- **WHEN** an episode has no stored chapters, regardless of which representation is served
- **THEN** the chapters endpoint responds successfully with an empty `chapters` array
