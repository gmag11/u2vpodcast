## Purpose

Defines server-persisted, per-user, explicitly ordered playlists of episodes: CRUD, item management, reordering, and playback seeding.

## ADDED Requirements

### Requirement: Playlists are per-user, named, and manageable

The API SHALL let the authenticated user list, create (with a name), rename, and delete their own playlists. Playlist names SHALL be unique for the user; creating a duplicate name SHALL fail. Deleting a playlist SHALL remove its items. A user SHALL only access their own playlists.

#### Scenario: Creating and listing playlists
- **WHEN** the user creates playlists named "En coche" and "Trabajo"
- **THEN** both appear in the playlist list with their item counts

#### Scenario: Duplicate playlist name is rejected
- **WHEN** the user creates a playlist whose name already exists for them
- **THEN** the creation fails with a conflict response and no playlist is created

#### Scenario: Deleting a playlist removes its items
- **WHEN** the user deletes a playlist that contains episodes
- **THEN** the playlist and its items are removed, and other playlists are unaffected

#### Scenario: Playlists are private to the owner
- **WHEN** a playlist id belonging to another user is used
- **THEN** the request fails as if the resource did not exist

### Requirement: Playlist episodes keep an explicit order

Each playlist SHALL hold episodes in an explicit positional order. The API SHALL support appending an episode (persisting its position), removing an episode (reindexing the remaining order), and reordering the whole playlist by submitting the full ordered list of episode ids.

#### Scenario: Appending an episode
- **WHEN** the user adds an episode to a playlist with two episodes
- **THEN** the episode is appended at the end, after the existing two

#### Scenario: Duplicate episode in a playlist is rejected
- **WHEN** the user adds an episode already present in that playlist
- **THEN** the add fails and the playlist is unchanged

#### Scenario: Removing an episode reindexes order
- **WHEN** the user removes the middle episode of a three-episode playlist
- **THEN** the remaining two episodes keep their relative order with contiguous positions

#### Scenario: Reordering the playlist
- **WHEN** the user submits a new full order for a playlist
- **THEN** the episodes are stored in exactly that order

### Requirement: Playlist episodes are readable joined with channel info

The API SHALL return a playlist's episodes in their stored order, joined with channel slug and title so cards can render channel links and metadata without extra requests.

#### Scenario: Reading a playlist's episodes
- **WHEN** the user requests the episodes of a playlist
- **THEN** they are returned in position order with each episode's channel slug and title

### Requirement: Playing a playlist seeds the playback queue

Starting playback on an episode of a playlist SHALL seed the player's up-next queue with the remaining episodes of that playlist in its stored order, so auto-advance walks the playlist.

#### Scenario: Auto-advance through a playlist
- **WHEN** the user plays the first episode of a playlist
- **THEN** the queue contains the rest of the playlist in order and each finished episode advances to the next

#### Scenario: Playing a middle episode schedules the tail
- **WHEN** the user starts playback on the third episode of a five-episode playlist
- **THEN** the queue contains only the fourth and fifth episodes