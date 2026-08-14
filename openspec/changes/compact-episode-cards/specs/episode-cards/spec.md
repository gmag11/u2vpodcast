## ADDED Requirements

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
