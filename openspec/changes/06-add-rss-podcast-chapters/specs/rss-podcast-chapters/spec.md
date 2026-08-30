## Purpose

Exposes each episode's chapters to external podcast clients through the Podcasting 2.0 `<podcast:chapters>` mechanism: a per-episode JSON chapters endpoint referenced from the RSS feed, kept consistent with whichever media representation (original or SponsorBlock-processed) the feed item actually selects.

## ADDED Requirements

### Requirement: Episode chapters are exposed as Podcasting 2.0 JSON
The system SHALL expose a JSON chapters endpoint per episode following the Podcasting 2.0 chapters JSON schema: a top-level object with a `chapters` array of entries, each with `startTime` (seconds) and `title`. When the episode's selected feed enclosure is the original MP3, the endpoint SHALL return the episode's stored, untranslated chapters. When the episode's selected feed enclosure is a SponsorBlock-processed MP3, the endpoint SHALL return the same translated chapters embedded into that processed file. An episode with no stored chapters SHALL return a successful response with an empty `chapters` array.

#### Scenario: Original enclosure selected
- **WHEN** an episode's feed item selects the original MP3 and the episode has stored chapters
- **THEN** the chapters endpoint returns the original, untranslated start times and titles

#### Scenario: Processed enclosure selected
- **WHEN** an episode's feed item selects a SponsorBlock-processed MP3
- **THEN** the chapters endpoint returns the same translated chapters embedded into that processed file, not the original untranslated chapters

#### Scenario: Episode has no stored chapters
- **WHEN** an episode has no stored chapters, regardless of which enclosure is selected
- **THEN** the chapters endpoint responds successfully with an empty `chapters` array

### Requirement: Feed items reference the chapters endpoint via the Podcasting 2.0 namespace
Each feed `<item>` for an episode with stored chapters SHALL include a `<podcast:chapters url="..." type="application/json+chapters"/>` element pointing at that episode's chapters endpoint, using the `podcast` namespace declared on the channel element. An episode with no stored chapters SHALL have no `<podcast:chapters>` element in its `<item>`.

#### Scenario: Item includes podcast:chapters
- **WHEN** a feed item is generated for an episode with stored chapters
- **THEN** its `<item>` includes a `<podcast:chapters>` element whose `url` resolves to that episode's chapters endpoint

#### Scenario: Item omits podcast:chapters when there are no chapters
- **WHEN** a feed item is generated for an episode with no stored chapters
- **THEN** its `<item>` has no `<podcast:chapters>` element

#### Scenario: Namespace is declared once per channel
- **WHEN** a feed document is generated for a channel with at least one episode that has chapters
- **THEN** the `podcast` XML namespace is declared on the feed's `<channel>` (or `<rss>`) element exactly once
