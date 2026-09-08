## Purpose

Extends the episode card with the single-playlist add/remove toggle and the "mark as not listened" control defined by the `playlist` capability.

## ADDED Requirements

### Requirement: Add/remove toggle for the single playlist

Each episode card SHALL expose a toggle reflecting whether the episode is in the single playlist: adding when absent, removing when present, with a notification on each action.

#### Scenario: Adding an episode
- **WHEN** the episode is not in the playlist and the user activates the card's playlist toggle
- **THEN** the episode is appended to the end of the playlist and a success notification is shown

#### Scenario: Removing an episode
- **WHEN** the episode is in the playlist and the user activates the card's playlist toggle
- **THEN** the episode is removed from the playlist and reindexed, and a notification is shown

#### Scenario: Adding an already-present episode is prevented
- **WHEN** the episode is already in the playlist
- **THEN** the action fails with a message and the playlist is unchanged

### Requirement: Mark-as-not-listened control re-adds to the playlist

For an episode marked listened, the card SHALL expose a control that clears the listened state (resetting the stored position to zero) and appends the episode to the end of the playlist.

#### Scenario: Unmarking a listened episode
- **WHEN** the user activates the "mark as not listened" control on a listened episode
- **THEN** the episode's listened state clears, its position resets to zero, the card swaps back from the played mark, and the episode appears again at the end of the playlist

#### Scenario: Unmarking an episode already pending
- **WHEN** the intended episode is already in the playlist
- **THEN** the listened state still clears and the episode remains in the playlist exactly once