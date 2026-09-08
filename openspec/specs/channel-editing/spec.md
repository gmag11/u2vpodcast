# channel-editing

## Purpose

Defines how users edit existing channels (title and URL) through the API and the SPA. Edits must persist across reloads, reject empty titles with a clear error, and never alter the channel's immutable slug or audio directory.

## Requirements

### Requirement: Channel edits persist title and URL

The channel update endpoint (and its supporting model/SQL) SHALL persist the edited `title` alongside `active`, `first`, `max`, and `url`. A successful `PUT /api/1.0/channels/{id-or-slug}/` SHALL return the channel with the new title and URL such that subsequent reads and reloads reflect the edit. A non-empty sanitized title SHALL be required; the URL SHALL be stored as provided after validation.

#### Scenario: Title edit is persisted
- **WHEN** a client submits an edit changing a channel's title
- **THEN** the response and all subsequent reads return the channel with the new title

#### Scenario: URL edit is persisted
- **WHEN** a client submits an edit changing a channel's URL
- **THEN** the response and subsequent reads return the new URL, and future syncs fetch from it

#### Scenario: Reload reflects edited values
- **WHEN** the SPA reloads the channel list after an edit
- **THEN** the edited title and URL are shown and are not silently reverted

### Requirement: Renaming a channel does not change its slug

Editing a channel's title SHALL NOT regenerate its slug and SHALL NOT rename its audio directory. The slug remains the immutable value assigned at creation, consistent with the `channel-slugs` capability.

#### Scenario: Title rename keeps slug and audio directory
- **WHEN** a channel with slug `confesiones_de_gasolinera` is renamed through the edit form
- **THEN** the slug stays `confesiones_de_gasolinera` and the audio directory path `{audios}/confesiones_de_gasolinera` is unchanged
