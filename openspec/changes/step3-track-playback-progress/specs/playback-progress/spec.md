## Purpose

Defines server-persisted per-episode playback position and listened state: the episode API exposes both, the player saves position during playback and resumes automatically, and completion marks the episode as played.

## ADDED Requirements

### Requirement: Episode exposes playback progress fields

Every episode returned by the episode API SHALL include its persisted playback position (`position_seconds`) and its listened state: the `listen` boolean mark plus the `listened_at` timestamp when it was completed.

#### Scenario: Episode payload includes progress
- **WHEN** the frontend requests episodes (channel list or global feed)
- **THEN** each episode includes `position_seconds`, `listen`, and `listened_at` fields reflecting the latest saved progress

#### Scenario: Fresh episode has zero progress
- **WHEN** an episode has never been played
- **THEN** `position_seconds` is 0, `listen` is false, and `listened_at` is null

### Requirement: Playback progress is savable through the API

The API SHALL expose an authenticated endpoint to update an episode's progress in one call: a position in seconds and an optional listened flag. A successful update SHALL persist both fields and return the updated episode.

#### Scenario: Saving a position
- **WHEN** the player saves progress for an episode at 1300 seconds without completing it
- **THEN** the episode's stored `position_seconds` becomes 1300 and `listen` stays false

#### Scenario: Marking an episode listened on completion
- **WHEN** the player saves progress for an episode with the listened flag set
- **THEN** `listen` becomes true, `listened_at` is set to the current server time, and `position_seconds` stores the final position

#### Scenario: Marking listened via long-press skip
- **WHEN** the user long-presses next (step-2 dual next control) on the current episode
- **THEN** the episode is marked listened exactly as if it had completed: `listen` true, `listened_at` set, `position_seconds` stored as its duration

### Requirement: Player resumes from the stored position

When playback starts on an episode whose stored position is above 30 seconds and below 95% of its duration, the player SHALL seek to that position automatically and continue from there. A "start over" affordance SHALL let the user play the episode from zero, clearing the stored position.

#### Scenario: Automatic resume mid-episode
- **WHEN** the user plays an episode previously left at 45 minutes of a 60-minute duration
- **THEN** playback starts at 45 minutes instead of zero

#### Scenario: Resume also applies when navigating back
- **WHEN** the user navigates back (step-2 dual previous) to an episode whose stored position is above 30 seconds and below 95% of the duration
- **THEN** playback resumes from the stored position, identical to a fresh play of that episode

#### Scenario: No resume for near-start or near-end positions
- **WHEN** the user plays an episode whose stored position is at most 30 seconds, or at least 95% of the duration
- **THEN** playback starts from the beginning

#### Scenario: Start over clears the position
- **WHEN** the user presses "start over" for an episode with a stored position
- **THEN** playback starts at zero and the stored position is reset to 0

### Requirement: Keyboard shortcut seeks by 15 seconds

While an episode is loaded, the player SHALL seek 15 seconds forward on the `ArrowRight` key and 15 seconds backward on the `ArrowLeft` key, clamping to the episode bounds. The shortcuts SHALL only apply while the frontend document has focus and SHALL NOT capture the keys when focus is in an editable control (input, textarea, select, contenteditable) or a slider.

#### Scenario: Arrow right seeks forward
- **WHEN** the frontend has focus, an episode is loaded, and the user presses `ArrowRight`
- **THEN** playback seeks 15 seconds forward, clamped to the episode duration

#### Scenario: Arrow left seeks backward
- **WHEN** the frontend has focus, an episode is loaded, and the user presses `ArrowLeft`
- **THEN** playback seeks 15 seconds backward, clamped to zero

#### Scenario: No episode loaded
- **WHEN** the frontend has focus but no episode is loaded and the user presses an arrow key
- **THEN** nothing happens

#### Scenario: Editable controls keep the keys
- **WHEN** the focus is inside a text input, textarea, select, contenteditable element, or a slider
- **THEN** arrow keys behave normally (input cursor / slider value) and do not seek

#### Scenario: Page without focus
- **WHEN** the frontend tab/window does not have focus
- **THEN** the arrow shortcuts do not fire

### Requirement: Player saves position during playback

While an episode is playing, the player SHALL persist its current position at least once every 10 seconds, and SHALL persist again on pause, on stop, and immediately before the page is hidden or unloaded. The listened flag SHALL be sent true only when the episode completes or the user explicitly marks it (long-press skip).

#### Scenario: Throttled position saves
- **WHEN** an episode plays for several minutes in a single session
- **THEN** the position is persisted periodically (no more often than every 10 seconds) so a later resume starts at the last saved point

#### Scenario: Position saved on pause and unload
- **WHEN** the user pauses an episode or closes/reloads the tab mid-episode
- **THEN** the latest position is persisted before the episode stops being observed

### Requirement: Episode card shows played mark and resume hint

The episode card SHALL display a visible played mark (check indicator with "listened" label) when the episode's `listen` is true. For a partially played episode (`listen` false with a position above the 30-second threshold) the card SHALL show a progress hint with the stored position. Both indicators SHALL reflect the shared player's in-memory episode so they update without a reload.

#### Scenario: Played mark on completed episodes
- **WHEN** an episode has been completed (listen true)
- **THEN** its card shows a check mark and a "listened" label

#### Scenario: Resume hint on partial episodes
- **WHEN** an episode has a position above 30 seconds and is not marked listened
- **THEN** its card shows a hint indicating it can be continued (for example "Continue at MM:SS")

#### Scenario: Completed episode hint disappears
- **WHEN** the user finishes an episode that previously showed a resume hint
- **THEN** the card switches to the played mark and the resume hint is no longer shown