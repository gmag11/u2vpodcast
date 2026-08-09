## Purpose

Assigns each channel a stable, unique, human-readable slug derived from its YouTube title, stored immutably, so every public URL (feed, media, JSON API, SPA) addresses the channel by a self-describing name instead of an opaque numeric database id.

## ADDED Requirements

### Requirement: Each channel has an immutable slug

The system SHALL store a `slug` field on every channel (NOT NULL, UNIQUE) that is derived from the channel's title at creation time and SHALL NOT change thereafter, even if the YouTube title changes. The numeric `id` remains the primary key and the episodes `channel_id` foreign key stays numeric.

#### Scenario: YouTube renames the channel but the slug stays
- **WHEN** a channel was created with title "Confesiones de Gasolinera" and slug `confesiones_de_gasolinera`, and YouTube later renames the channel to "CdG"
- **THEN** the channel's slug remains `confesiones_de_gasolinera` and all its feed/media/API URLs stay unchanged

### Requirement: Slug format

The slug SHALL be derived from the channel title by: lowercasing, normalizing unicode accents/diacritics to ASCII, replacing every run of non-alphanumeric characters with a single underscore, and trimming leading/trailing underscores. The result MUST contain only lowercase `a-z`, `0-9`, and underscores.

#### Scenario: Title with spaces and accents
- **WHEN** a channel is created with the title "Confesiones de Gasolinera"
- **THEN** its slug is `confesiones_de_gasolinera`

#### Scenario: Title with accents and punctuation
- **WHEN** a channel is created with the title "¡Híbridos, eléctricos y más!"
- **THEN** its slug is `hibridos_electricos_y_mas`

### Requirement: Slug uniqueness on collision

When slugifying a new channel's title would produce a slug already in use, the system SHALL append a hyphen and the lowest integer (`-2`, `-3`, …) that makes the slug unique.

#### Scenario: Two channels with the same title
- **WHEN** a channel "Confesiones de Gasolinera" already has slug `confesiones_de_gasolinera`, and a second channel with the same title is added
- **THEN** the second channel's slug is `confesiones_de_gasolinera-2`

### Requirement: Existing channels and audio directories are migrated at startup

On startup, the system SHALL backfill the `slug` column for any existing channel that lacks one (deriving it from its current title with the same slugify and uniqueness rules), and SHALL rename the audio directory `/app/audios/{id}/` to `/app/audios/{slug}/` for each channel when the `{id}` directory exists and the `{slug}` directory does not. This migration runs once and is idempotent.

#### Scenario: First startup after the upgrade
- **WHEN** the app starts with existing channels whose `slug` is NULL and audio directories named by numeric id
- **THEN** each channel gets a backfilled `slug` and its audio directory is renamed from `/app/audios/{id}/` to `/app/audios/{slug}/`

#### Scenario: Restart after migration already ran
- **WHEN** the app restarts after a successful migration and all channels already have a `slug`
- **THEN** no rename happens and no error is raised

### Requirement: The JSON API addresses channels by slug

The JSON API channel routes SHALL address a channel by its slug: `/api/1.0/channels/{slug}/` (single channel read), `/api/1.0/channels/{slug}/episodes/` (episodes for a channel), and channel create/update/delete identify the channel by slug in the path or body. The Channel JSON response SHALL include the `slug` field.

#### Scenario: Read a channel by slug
- **WHEN** a client sends `GET /api/1.0/channels/confesiones_de_gasolinera/` with a valid session
- **THEN** the system responds `200` with the channel whose slug is `confesiones_de_gasolinera`

#### Scenario: Episodes for a channel by slug
- **WHEN** a client sends `GET /api/1.0/channels/confesiones_de_gasolinera/episodes/` with a valid session
- **THEN** the system responds `200` with the episodes of the channel whose slug is `confesiones_de_gasolinera`

### Requirement: The SPA routes address channels by slug

The SPA's channel detail route SHALL be `/app/{slug}` (was `/app/{id}`), and the feed/media links rendered by the SPA SHALL use the channel's `slug`.

#### Scenario: Channel card links to the detail page by slug
- **WHEN** the SPA renders a channel card for a channel with slug `confesiones_de_gasolinera`
- **THEN** the "open channel" link points to `/app/confesiones_de_gasolinera`

#### Scenario: Channel card links to the feed by slug
- **WHEN** the SPA renders a channel card for a channel with slug `confesiones_de_gasolinera`
- **THEN** the feed link points to `/channels/confesiones_de_gasolinera/feed.xml`