## Purpose

Defines the layout and inline playback controls of the episode card in the Vue 3 SPA. Cards keep a compact vertical footprint by exposing only play/pause and stop bound to the shared player, showing a total-duration-only label, and placing the controls per breakpoint; advanced controls (seek, volume, speed) live exclusively in the persistent bottom player bar.

## Requirements

### Requirement: Compact card footprint without inline advanced controls

Each `EpisodeCard` SHALL render without an inline seek bar, without a live position counter, without a playback speed control, and without a volume control. These controls SHALL remain available only in the persistent bottom player bar. The card SHALL retain play/pause and stop controls.

#### Scenario: Card has no seek bar
- **WHEN** an episode card is rendered
- **THEN** the card shows no seek/scrubber bar and no live position counter

#### Scenario: Card has no speed or volume controls
- **WHEN** an episode card is rendered
- **THEN** the card shows no playback speed control and no volume/mute control

#### Scenario: Advanced controls remain in the persistent bar
- **WHEN** playback is active on an episode
- **THEN** the persistent bar still provides the position scrubber, volume (mute + level), and playback speed controls

### Requirement: Total duration label only

The episode card SHALL display the episode's total duration. When the card's episode is the currently loaded one, the label SHALL come from the shared player's total duration; otherwise it SHALL come from the episode's stored duration. The card SHALL NOT display the current playback position.

#### Scenario: Card shows total duration for the current episode
- **WHEN** the card's episode is currently loaded in the shared player
- **THEN** the card displays the total duration of the episode and no position counter

#### Scenario: Card shows stored duration for other episodes
- **WHEN** the card's episode is not currently loaded in the shared player
- **THEN** the card displays the episode's stored total duration

### Requirement: Play/pause and stop bound to the shared player

The card's play/pause and stop controls SHALL bind to the global audio player store. Pressing play on an episode SHALL start the shared player for that episode; toggling pause SHALL pause the shared element; pressing stop SHALL reset and stop the shared element. The controls SHALL reflect the shared playing state and disable stop when the episode is not the current one.

#### Scenario: Play starts the shared player
- **WHEN** the user presses play in an episode card
- **THEN** the shared player loads and plays that episode and the persistent bar appears

#### Scenario: Pause toggles the shared player
- **WHEN** the card's episode is playing and the user presses the card's pause button
- **THEN** the shared element pauses and both the card and the persistent bar show the paused state

#### Scenario: Stop clears playback
- **WHEN** the user presses the card's stop button
- **THEN** the shared player stops, resets its position to zero, and begins the persistent bar's auto-hide

### Requirement: Play/stop placement by breakpoint

On desktop (`sm` and up) the play/pause and stop buttons SHALL be positioned below the episode thumbnail. On mobile the buttons SHALL be positioned to the right of the thumbnail. Exactly one placement SHALL be visible at any viewport width.

#### Scenario: Buttons below thumbnail on desktop
- **WHEN** the card is viewed at `sm` breakpoint or wider
- **THEN** the play/pause and stop buttons render below the thumbnail

#### Scenario: Buttons right of thumbnail on mobile
- **WHEN** the card is viewed below the `sm` breakpoint
- **THEN** the play/pause and stop buttons render to the right of the thumbnail

#### Scenario: Only one placement renders
- **WHEN** the card is rendered at any viewport width
- **THEN** only the placement for that width is visible

### Requirement: Played mark, resume hint, and progress strip on episode cards

The episode card SHALL render its playback state compactly, per the `playback-progress` capability: the top-right corner tinted green when the episode is listened (no label or icon), a resume hint for partially played episodes, and a read-only progress strip spanning the card's bottom edge that reflects the saved position (the live playhead for the currently playing episode) and ignores pointer interaction.

#### Scenario: Played mark on completed episodes
- **WHEN** an episode has `listen` true
- **THEN** the card's top-right corner is tinted green instead of showing a label or check

#### Scenario: Resume hint on partial episodes
- **WHEN** an episode has a stored position above 30 seconds and `listen` is false
- **THEN** the card shows a hint with the stored position (for example "Continue at MM:SS") and an affordance to start over

#### Scenario: Progress strip reflects the saved point
- **WHEN** an episode has a saved position
- **THEN** the card shows a bottom progress strip sized proportionally to `position_seconds` over the episode duration

#### Scenario: Progress strip is read-only
- **WHEN** the user clicks or drags on the card's progress strip
- **THEN** playback is unaffected (the strip has no interaction handlers)

#### Scenario: No indicator for untouched episodes
- **WHEN** an episode has never been played or its position is at most 30 seconds
- **THEN** the card shows neither the played mark nor a resume hint, and no progress strip

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
