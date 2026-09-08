## Purpose

Defines the persistent bar extension for queue control and the visible, editable, persisted "up next" queue that feeds auto-advance behavior.

## Requirements

### Requirement: Shown the up-next queue and controls in the persistent bar

The player bar SHALL expose previous/next navigation controls (the next control is already provided by the `auto-advance` capability / step 1, next to the stop button) and a queue panel ("Up next") listing the upcoming episodes. The next control SHALL be disabled when the queue is empty; the previous control SHALL be disabled when there is no playback history. The queue panel SHALL allow removing an individual upcoming episode and clearing the whole queue.

#### Scenario: Skipping to the next episode
- **WHEN** the user presses the next control while episodes remain in the queue
- **THEN** the player immediately loads and plays the first queued episode

#### Scenario: Previous restarts the current episode past the threshold
- **WHEN** the user presses the previous control while the current episode has played for more than 3 seconds
- **THEN** the current episode restarts from zero instead of navigating

#### Scenario: Previous navigates back within the threshold
- **WHEN** the user presses the previous control while the current episode has played for at most 3 seconds
- **THEN** the player loads the most recently played episode from the playback history

#### Scenario: Queue panel shows upcoming episodes
- **WHEN** the user opens the queue panel while episodes are queued
- **THEN** the panel lists the upcoming episodes in order with a per-item remove action and a total count

#### Scenario: Removing and clearing the queue
- **WHEN** the user removes an individual episode from the panel, or presses clear all
- **THEN** the queue is updated accordingly and playback of the current episode is unaffected

### Requirement: Queue persists across page reloads

The up-next queue, the playback history, and the currently loaded episode SHALL be serialized to `localStorage` and rehydrated when the app loads, so a page reload does not lose the queue or the session's current episode. Malformed or unreadable stored payloads SHALL be discarded silently.

#### Scenario: Reload keeps the queue
- **WHEN** the user reloads the page while episodes remain queued
- **THEN** the queue panel shows the same upcoming episodes and playback can continue from the queue

#### Scenario: Reload restores the current episode
- **WHEN** the user reloads the page while an episode is loaded
- **THEN** that episode is restored as the player's current episode, so the bar and its controls reflect it

#### Scenario: Reload with a queue but no episode keeps the bar reachable
- **WHEN** the loader restores a non-empty queue yet no current episode (e.g. a legacy payload)
- **THEN** the player bar is still shown in a queue-only state, so the queue remains accessible until the user plays an episode

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

### Requirement: Long-press next skips and marks the current episode listened

The next control SHALL distinguish a short press from a long press (threshold 500ms). A short press SHALL skip to the first queued episode without changing listened state. A long press SHALL also skip AND mark the current episode as listened — storing the final position as its duration — without waiting for the episode to finish. The listened mark SHALL follow the same persistence path as `ended` (per the `playback-progress` capability) and SHALL be reflected immediately on the episode card.

#### Scenario: Short press skips without marking
- **WHEN** the user presses and releases the next control within 500ms
- **THEN** the first queued episode plays and the current episode is not marked listened

#### Scenario: Long press skips and marks listened
- **WHEN** the user holds the next control for more than 500ms
- **THEN** the first queued episode plays and the current episode is marked listened with its final position stored as its duration

#### Scenario: Holding during playback of a finished episode
- **WHEN** the user long-presses next on an episode whose card already shows the played mark
- **THEN** the skip happens and the listened state stays as-is (no duplicate marking)

### Requirement: Queue consumption at end of playback

When an episode ends, the player SHALL remove it from the queue, persist the updated queue, and advance to the next queued episode, pushing the finished episode onto the playback history for the previous control. When no episode remains, the player SHALL stop and clear the queue.

#### Scenario: Advanced episode is recorded in history
- **WHEN** an episode finishes and the next one starts
- **THEN** the finished episode becomes the most recent entry of the playback history used by the previous control

#### Scenario: Empty queue on stop
- **WHEN** the last queued episode finishes and there is no repeat mode active
- **THEN** the player stops and the queue is emptied and persisted as empty

### Requirement: Navigating back applies saved playback position

When playback navigates back to a previously played episode, the player SHALL apply the same resume policy as starting an episode (per the `playback-progress` capability): resume from the saved position when it is above 30 seconds and below 95% of the duration, otherwise start from zero.

#### Scenario: Resume after navigating back
- **WHEN** the user navigates back to an episode whose saved position is at 45 minutes of a 60-minute duration
- **THEN** the episode resumes at 45 minutes instead of starting from zero

#### Scenario: Restart without a saved position
- **WHEN** the user navigates back to an episode with no meaningful saved position
- **THEN** the episode plays from the beginning
