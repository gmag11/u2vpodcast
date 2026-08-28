## Purpose

Defines the single server-persisted playlist (pending episodes): a unique ordered list per instance with add, remove, reorder, completion-based removal, and playback seeding.

## Requirements

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

### Requirement: Deleting an episode removes it from the playlist

When an episode is deleted, whether individually or as part of deleting its whole channel, any playlist entry referencing that episode SHALL be deleted in the same operation, and the remaining playlist positions SHALL be reindexed contiguously. This SHALL happen regardless of whether the episode was ever added to the playlist.

#### Scenario: Retention-limit prune removes the playlist entry
- **WHEN** the retention-limit worker deletes an episode that is currently in the playlist
- **THEN** the episode's playlist entry is deleted and the remaining playlist positions are reindexed contiguously

#### Scenario: Deleting a channel removes its episodes' playlist entries
- **WHEN** a channel with two of its episodes on the playlist is deleted
- **THEN** both playlist entries are deleted along with the episodes, and the remaining playlist positions are reindexed contiguously

#### Scenario: Deleting an episode not on the playlist is a no-op for the playlist
- **WHEN** an episode that was never added to the playlist is deleted
- **THEN** the playlist is unaffected

### Requirement: Playlist order is edited with inline drag handles

When the playlist contains at least two episodes, each episode row in the main playlist view SHALL expose one compact six-dot drag handle immediately to the left of its existing episode card. The handle SHALL replace the stacked up/down buttons; reordering SHALL NOT require opening a separate drawer, modal, or editing mode. Dragging the handle SHALL move the complete episode row while preserving the card's playback and episode actions.

The user SHALL be able to move a row directly to any insertion position between the other rows with pointer or touch input. While a row is moving, the playlist SHALL visually identify the active row and the prospective insertion position between cards. While an episode is dragged near the top or bottom edge of the visible playlist, the page SHALL automatically scroll in that direction for as long as the pointer remains in the edge zone and further content is available. Auto-scroll SHALL stop when the pointer leaves the edge zone or the page reaches its boundary, allowing the episode to be dropped at a position that was not initially visible.

The handle SHALL provide an equivalent keyboard interaction that lets a focused row be picked up, moved, and dropped, and SHALL announce the operation state and resulting position to assistive technology.

On mobile viewports, the drag handle SHALL provide a touch target of at least 44 by 44 CSS pixels without materially narrowing or overlapping the episode card. Dragging SHALL start only from the handle so touching or swiping the card elsewhere preserves its existing controls and normal page scrolling.

Completing a drop that changes the order SHALL submit the complete resulting list of episode ids for persistence. The main playlist SHALL retain the new order when persistence succeeds. If persistence fails, the main playlist SHALL restore the last persisted order and the user SHALL receive an error notification. Dropping an episode without changing its position SHALL NOT submit a reorder request.

When playback was started from the playlist, completing a changed reorder SHALL immediately rebuild the active authored queue from the episodes after the current episode in the new playlist order. The next automatic advance and the Up Next interface SHALL use that updated queue without restarting or interrupting the current episode. If persistence fails, the active queue SHALL be reconciled with the restored playlist order. A queue started from a non-playlist episode list SHALL remain unchanged.

#### Scenario: Playlist presents compact drag handles
- **WHEN** the playlist contains at least two episodes
- **THEN** each episode card has one six-dot drag handle on its left and no up/down reorder buttons are shown

#### Scenario: Dropping an episode between two others
- **WHEN** the user drags an episode row by its handle and drops it between two other cards
- **THEN** the main playlist submits the complete resulting episode-id order and displays that order after persistence succeeds

#### Scenario: Dragging to an initially hidden position
- **WHEN** the user keeps a dragged episode near the top or bottom edge of the visible playlist and more episodes exist beyond that viewport edge
- **THEN** the page scrolls in that direction until the pointer leaves the edge zone or the page boundary is reached, exposing additional insertion positions without ending the drag

#### Scenario: Reordering on a mobile viewport
- **WHEN** the playlist is viewed on a narrow touch device
- **THEN** every drag handle has a touch target of at least 44 by 44 CSS pixels and neither overlaps nor causes horizontal overflow in its episode card

#### Scenario: Using the card without dragging
- **WHEN** the user clicks, taps, or swipes an episode card outside its drag handle
- **THEN** the card's existing actions and normal page scrolling remain available and no drag operation starts

#### Scenario: Reordering with a keyboard
- **WHEN** a keyboard user picks up a focused episode, moves it to a different insertion position, and drops it
- **THEN** the same complete order is persisted and the updated position is announced to assistive technology

#### Scenario: Reorder persistence fails
- **WHEN** the user completes a changed drop and the reorder request fails
- **THEN** the main playlist restores the last persisted order and an error notification is shown

#### Scenario: Episode is dropped in its original position
- **WHEN** the user drops an episode without changing its position
- **THEN** the visible order remains unchanged and no reorder request is sent

#### Scenario: Active playlist playback follows a new order
- **WHEN** an episode is playing from the playlist and the user completes a changed reorder
- **THEN** Up Next immediately lists the episodes after the current episode in the new playlist order and automatic advance plays the first of them

#### Scenario: Active queue follows reorder rollback
- **WHEN** an active playlist-sourced queue is updated optimistically and reorder persistence fails
- **THEN** Up Next is restored to the remaining order derived from the restored playlist

#### Scenario: Non-playlist queue is not replaced
- **WHEN** playback was seeded from another episode list and the user reorders the playlist
- **THEN** the existing Up Next queue remains unchanged

#### Scenario: Playlist cannot be meaningfully reordered
- **WHEN** the playlist contains fewer than two episodes
- **THEN** the playlist view does not expose drag handles or stepwise reorder controls
