## Purpose

Defines automatic advance-to-next behavior: when the shared audio player reaches the end of an episode, it plays the next episode from the context list that started playback (the visible channel or feed list), and stops when no next episode exists.

## ADDED Requirements

### Requirement: Player advances to the next episode when the current one ends

When the shared audio element fires `ended` and the playback queue contains a next episode, the player SHALL load and play that episode using the same shared element, keeping the persistent bar visible and updating its current-episode display. When the queue is empty, the player SHALL stop exactly as before this capability (position reset, bar auto-hide).

#### Scenario: Advancing through a channel list
- **WHEN** the user starts playback on episode N from a channel's episodes page and episode N finishes while other episodes follow it in the list
- **THEN** the player loads and plays the episode that followed N in the displayed channel list without user action

#### Scenario: Advancing through the global feed
- **WHEN** the user starts playback on an episode from the History screen (global feed) and it finishes while other episodes follow in that visible feed
- **THEN** the player plays the next visible episode of the feed, honoring the current search/filter ordering

#### Scenario: Advancing stops at the end of the list
- **WHEN** the last episode of the list is playing and reaches the end
- **THEN** the player stops, resets its position to zero, and the persistent bar begins its auto-hide delay

### Requirement: Queue is seeded from the context list at play time

When playback starts from a list, the player SHALL build its queue as a snapshot of that list taken at play time, excluding the played episode and everything before it, and preserving the displayed order (including any active search filter). The queue SHALL NOT be rebuilt if the underlying view list changes later; it is a snapshot.

#### Scenario: Seeding from a filtered channel list
- **WHEN** the user filters a channel's episodes and presses play on one of the visible episodes
- **THEN** the queue contains the remaining visible episodes in the filtered order

#### Scenario: Seeding from the filtered global feed
- **WHEN** the user filters the History screen and presses play on one of the visible episodes
- **THEN** the queue contains the remaining visible feed episodes in the filtered order

#### Scenario: Play without a context list keeps single-episode behavior
- **WHEN** the user starts playback on an episode outside any list (no context list provided)
- **THEN** the queue is empty and the player stops when the episode ends

#### Scenario: Queue snapshot is stable when the view changes
- **WHEN** playback started from a list and the user later changes the search filter or the list refreshes
- **THEN** the queued next episodes remain those captured at play time, unaffected by the view change

### Requirement: Manual next control in the player bar

The persistent player bar SHALL expose a next/skip control placed immediately to the right of the stop control. When the queue contains an episode, activating it SHALL advance playback to the next queued episode immediately using the same mechanism as end-of-playback advance. When the queue is empty the control SHALL be disabled.

#### Scenario: Skipping to the next queued episode
- **WHEN** the next control is activated while the queue holds an episode
- **THEN** the player immediately loads and plays that episode, and the queue drains by one

#### Scenario: Disabled without a queue
- **WHEN** the queue is empty
- **THEN** the next control is disabled and activating it does nothing

#### Scenario: Placement next to stop
- **WHEN** the player bar is visible
- **THEN** the next control renders immediately to the right of the stop control