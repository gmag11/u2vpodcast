## Purpose

Defines the single server-persisted playlist (pending episodes): a unique ordered list per instance with add, remove, reorder, completion-based removal, and playback seeding.

## ADDED Requirements

### Requirement: A single playlist with explicit order

The app SHALL expose exactly one playlist per instance holding episodes in an explicit positional order. Adding an episode SHALL append it at the end; adding an episode already present SHALL fail and leave the list unchanged. Removing an episode SHALL reindex the remaining order contiguously. Reordering SHALL accept a full ordered list of episode ids and store exactly that order.

#### Scenario: Appending an episode
- **WHEN** the user adds an episode to a playlist that already has two episodes
- **THEN** the episode is appended after the existing two

#### Scenario: Duplicate episode rejected
- **WHEN** the user adds an episode already in the playlist
- **THEN** the add fails and the playlist is unchanged

#### Scenario: Removal reindexes positions
- **WHEN** the user removes the middle episode of a three-episode playlist
- **THEN** the remaining two keep their relative order with contiguous positions

#### Scenario: Reorder rewrites the full order
- **WHEN** the user submits a new complete ordering of the playlist's episodes
- **THEN** the playlist stores exactly that order

### Requirement: Playlist episodes readable joined with channel info

The API SHALL return the playlist's episodes in stored order, joined with channel slug and title so cards render channel links without extra requests. Episodes that no longer exist SHALL be omitted.

#### Scenario: Reading the playlist
- **WHEN** the user requests the playlist
- **THEN** the episodes are returned in position order with their channel slug and title

#### Scenario: Missing episodes are skipped
- **WHEN** an episode referenced by the playlist no longer exists
- **THEN** it is omitted from the playlist response

### Requirement: Finishing an episode removes it from the playlist and marks it listened

When an episode that came from the playlist finishes (`ended`), or is marked listened by the step-2 long-press skip, the player SHALL mark it listened (per `playback-progress`) and remove it from the playlist. A short-press skip SHALL NOT remove it, because it does not mark the episode listened.

#### Scenario: Completed episode leaves the playlist
- **WHEN** an episode put on the playlist finishes playing
- **THEN** the episode is marked listened and removed from the playlist

#### Scenario: Long-press skip also removes it
- **WHEN** the user long-presses next on a playlist episode
- **THEN** the episode is marked listened and removed from the playlist

#### Scenario: Short-press skip keeps it
- **WHEN** the user short-presses next on a playlist episode that has not finished
- **THEN** the episode stays in the playlist unmarked

### Requirement: Playing the playlist seeds the playback queue

Starting playback on a playlist episode SHALL seed the up-next queue with the remaining playlist episodes in stored order, so auto-advance walks the playlist.

#### Scenario: Auto-advance through the playlist
- **WHEN** the user plays the first episode of the playlist
- **THEN** the queue contains the rest of the playlist in order and each finished episode advances to the next while being removed from the playlist

#### Scenario: Playing a middle episode schedules the tail
- **WHEN** the user starts playback on the third episode of a five-episode playlist
- **THEN** the queue contains only the fourth and fifth episodes

### Requirement: Marking an episode as not listened re-adds it as pending

An episode marked listened is no longer in the playlist. The app SHALL provide a control that marks an episode as not listened: clearing the listened state and resetting its position to zero, and appending the episode to the end of the playlist.

#### Scenario: Re-adding a listened episode
- **WHEN** the user marks a listened episode as not listened
- **THEN** the episode's listened state clears (position reset to 0) and it is appended at the end of the playlist

#### Scenario: Already pending episodes are not duplicated
- **WHEN** the user marks as not listened an episode that is already in the playlist
- **THEN** the episode stays in the playlist exactly once