## Purpose

Defines the persistent bar extension for queue control and the visible, editable, persisted "up next" queue that feeds auto-advance behavior.

## ADDED Requirements

### Requirement: Shown the up-next queue and controls in the persistent bar

The player bar SHALL expose next/previous navigation controls and a queue panel ("Up next") listing the upcoming episodes. The next control SHALL be disabled when the queue is empty; the previous control SHALL be disabled when there is no playback history. The queue panel SHALL allow removing an individual upcoming episode and clearing the whole queue.

#### Scenario: Skipping to the next episode
- **WHEN** the user presses the next control while episodes remain in the queue
- **THEN** the player immediately loads and plays the first queued episode

#### Scenario: Previous returns to the last played episode
- **WHEN** the user presses the previous control after at least one advance
- **THEN** the player reloads the most recently played episode from the playback history and resumes it

#### Scenario: Queue panel shows upcoming episodes
- **WHEN** the user opens the queue panel while episodes are queued
- **THEN** the panel lists the upcoming episodes in order with a per-item remove action and a total count

#### Scenario: Removing and clearing the queue
- **WHEN** the user removes an individual episode from the panel, or presses clear all
- **THEN** the queue is updated accordingly and playback of the current episode is unaffected

### Requirement: Queue persists across page reloads

The up-next queue and playback history SHALL be serialized to `localStorage` and rehydrated when the app loads, so a page reload does not lose the queue. Malformed or unreadable stored payloads SHALL be discarded silently.

#### Scenario: Reload keeps the queue
- **WHEN** the user reloads the page while episodes remain queued
- **THEN** the queue panel shows the same upcoming episodes and playback can continue from the queue

#### Scenario: Corrupt stored queue is discarded
- **WHEN** the stored queue payload cannot be parsed
- **THEN** the app loads with an empty queue and no error is surfaced

### Requirement: Playing from a list seeds the persisted queue

Starting playback with a context list SHALL replace the current queue with the remaining items of that list (snapshot, as defined by the `auto-advance` capability). Starting playback without a list (for example, replaying an already-queued episode) SHALL keep the existing queue unchanged.

#### Scenario: New list play replaces the queue
- **WHEN** the user starts playback on an episode from a channel list or the global feed
- **THEN** the queue is replaced by the remaining items of that visible list

#### Scenario: Single-episode play keeps the queue
- **WHEN** the user starts playback on an episode with no context list while a queue already exists
- **THEN** the existing queue is untouched and auto-advance proceeds through it when the episode ends

### Requirement: Queue consumption at end of playback

When an episode ends, the player SHALL remove it from the queue, persist the updated queue, and advance to the next queued episode, pushing the finished episode onto the playback history for the previous control. When no episode remains, the player SHALL stop and clear the queue.

#### Scenario: Advanced episode is recorded in history
- **WHEN** an episode finishes and the next one starts
- **THEN** the finished episode becomes the most recent entry of the playback history used by the previous control

#### Scenario: Empty queue on stop
- **WHEN** the last queued episode finishes and there is no repeat mode active
- **THEN** the player stops and the queue is emptied and persisted as empty