## Purpose

Defines server-persisted per-episode playback position and listened state: the episode API exposes both, the player saves position during playback and resumes automatically, and completion marks the episode as played.

## Requirements

### Requirement: Episode exposes playback progress fields

Every episode returned by the episode API SHALL include its persisted playback position (`position_seconds`) and its listened state: the `listen` boolean mark plus the `listened_at` timestamp when it was completed.

#### Scenario: Episode payload includes progress
- **WHEN** the frontend requests episodes (channel list or global feed)
- **THEN** each episode includes `position_seconds`, `listen`, and `listened_at` fields reflecting the latest saved progress

#### Scenario: Fresh episode has zero progress
- **WHEN** an episode has never been played
- **THEN** `position_seconds` is 0, `listen` is false, and `listened_at` is null

### Requirement: Playback progress is savable through the API

The API SHALL expose an authenticated endpoint to update an episode's progress in one call: a position in seconds and an optional listened flag. A successful update SHALL persist both fields and complete with a success status (`204 No Content`); the write is fire-and-forget, so no response body is required. An unknown episode SHALL be answered with a `404`. The episode is addressed by its public id (`yt_id`).

#### Scenario: Saving a position
- **WHEN** the player saves progress for an episode at 1300 seconds without completing it
- **THEN** the episode's stored `position_seconds` becomes 1300 and `listen` stays false

#### Scenario: Marking an episode listened on completion
- **WHEN** the player saves progress for an episode with the listened flag set
- **THEN** `listen` becomes true on the false→true transition, `listened_at` is set to the current server time, and `position_seconds` stores the final position

#### Scenario: Repeated saves of a listened episode keep its timestamp
- **WHEN** the player keeps saving position on an episode that is already marked listened (e.g. while replaying it)
- **THEN** `listened_at` stays the original completion time and only `position_seconds` is updated

#### Scenario: Marking listened via long-press skip
- **WHEN** the user long-presses next (step-2 dual next control) on the current episode
- **THEN** the episode is marked listened exactly as if it had completed: `listen` true, `listened_at` set, `position_seconds` stored as its duration

#### Scenario: Unmarking an episode
- **WHEN** the player (or the card's unmark control) saves progress with the listened flag set to false
- **THEN** `listen` becomes false, `listened_at` is cleared, and `position_seconds` is updated to the sent value (0 for the unmark flow)

### Requirement: Player resumes from the stored position

When playback starts on an episode whose stored position is above 30 seconds and below 95% of its duration, the player SHALL seek to that position automatically and continue from there. Before deciding, the player SHALL query the server for the episode's stored progress (`GET /api/1.0/episodes/{yt_id}/progress/`), so a stale local copy never bypasses the resume. A "start over" affordance SHALL let the user play the episode from zero, clearing the stored position.

#### Scenario: Automatic resume mid-episode
- **WHEN** the user plays an episode previously left at 45 minutes of a 60-minute duration
- **THEN** playback starts at 45 minutes instead of zero

#### Scenario: Resume reads the server value, not the stale local copy
- **WHEN** the user plays an episode whose local copy reports no position but the server stores a position above 30 seconds
- **THEN** the player queries the stored progress and starts from the server position

#### Scenario: Resume also applies when navigating back, next, and auto-advance
- **WHEN** the user navigates back (step-2 dual previous), skips to the next episode, or the player auto-advances through the queue, arriving at an episode whose stored position is above 30 seconds and below 95% of the duration
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

#### Scenario: Re-listening updates the saved point
- **WHEN** a listened episode is replayed and the user stops mid-way
- **THEN** the live position is persisted (the mark stays listened) and the next play resumes there instead of restarting from the beginning

### Requirement: Stop halts playback; only the card's stop resets a non-reproducing episode

The player's stop control SHALL halt playback when the current episode is reproducing, flushing its current position so a later resume starts there, and SHALL NEVER reset a saved position when there is no target (persistent-bar stop): on a stopped or paused current episode it SHALL only converge to the stopped state, leaving the saved position untouched. The card's stop control (which passes its episode as target) SHALL: halt a reproducing current episode keeping its position; and reset to 0, keeping the listened mark unchanged, the saved position of the episode it belongs to when that episode is not reproducing — either a non-current card or the current episode when stopped or paused. Internal stops that are not user gestures (end of queue after completion, session teardown) SHALL keep the position as before. From the player itself, the only way to clear a saved position SHALL be the explicit "start over" flow.

#### Scenario: Player-bar stop halts a reproducing episode and keeps its position
- **WHEN** the user presses the persistent bar's stop while an episode is playing at 45 minutes
- **THEN** playback halts and the saved position stays 45 minutes, so the next play resumes there

#### Scenario: Player-bar stop on a stopped or paused episode keeps the position
- **WHEN** the user presses the persistent bar's stop on the current episode that is not reproducing (already stopped, or paused) and has a saved position above 0
- **THEN** the player converges to the stopped state and the saved position is left unchanged

#### Scenario: Card stop on a non-current episode resets that episode
- **WHEN** the user presses a card's stop while another episode is current and the card's episode is not reproducing
- **THEN** that episode's saved position is reset to 0 (listened mark kept) and the current episode's playback is untouched

#### Scenario: Card stop on the current episode when not reproducing resets it
- **WHEN** the user presses the current card's stop while the current episode is stopped or paused
- **THEN** the current episode's saved position is reset to 0, keeping the listened mark

#### Scenario: Card stop on a reproducing current episode halts and keeps the position
- **WHEN** the user presses the current card's stop while the current episode is playing
- **THEN** playback halts and the saved position is kept for a later resume

#### Scenario: Completion keeps the position
- **WHEN** an episode completes and the queue ends, or the session is torn down
- **THEN** the episode halts and its position is kept (no reset)
### Requirement: Episode card shows played mark, resume hint, and progress strip

The episode card SHALL display a played mark in its top-right corner — the corner itself tinted green, no icon or label — when the episode's `listen` is true. For a partially played episode (`listen` false with a position above the 30-second threshold) the card SHALL show a progress hint with the stored position. The card SHALL also render a read-only progress strip sized to the saved position, which for the current episode tracks the live playhead and never responds to pointer interaction. Both indicators SHALL reflect the shared player's in-memory episode so they update without a reload.

#### Scenario: Played mark on completed episodes
- **WHEN** an episode has been completed (listen true)
- **THEN** its card's top-right corner is tinted green, with no icon or label text

#### Scenario: Resume hint on partial episodes
- **WHEN** an episode has a position above 30 seconds and is not marked listened
- **THEN** its card shows a hint indicating it can be continued (for example "Continue at MM:SS")

#### Scenario: Progress strip reflects the saved point
- **WHEN** an episode has a saved position
- **THEN** its card shows a bottom progress strip whose width is proportional to the saved position over the episode duration

#### Scenario: Progress strip is read-only
- **WHEN** the user clicks or drags on the card's progress strip
- **THEN** playback is unaffected (the strip has no interaction handlers)

#### Scenario: Completed episode hint disappears
- **WHEN** the user finishes an episode that previously showed a resume hint
- **THEN** the card switches to the played mark and the resume hint is no longer shown
