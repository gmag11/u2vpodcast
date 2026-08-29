# episode-cards

## MODIFIED Requirements

### Requirement: Play/pause and stop bound to the shared player

The card's play/pause and stop controls SHALL bind to the global audio player store. Pressing play on an episode SHALL start the shared player for that episode; toggling pause SHALL pause the shared element. Pressing stop on a card SHALL halt a reproducing current episode without touching its saved position, and SHALL reset the saved position to 0 (keeping the listened mark) when the card's episode is not reproducing — including on a non-current card, which is the episode-card "rewind this episode" affordance. The persistent player bar's stop SHALL only halt the shared element and SHALL never reset a saved position. The controls SHALL reflect the shared playing state.

#### Scenario: Play starts the shared player
- **WHEN** the user presses play in an episode card
- **THEN** the shared player loads and plays that episode and the persistent bar appears

#### Scenario: Pause toggles the shared player
- **WHEN** the card's episode is playing and the user presses the card's pause button
- **THEN** the shared element pauses and both the card and the persistent bar show the paused state

#### Scenario: Card stop on a reproducing current episode halts and keeps the position
- **WHEN** the user presses the card's stop button while that episode is the current one and is playing
- **THEN** the shared player halts, the episode's saved position is unchanged, and the persistent bar begins its auto-hide

#### Scenario: Card stop on a non-reproducing episode resets its saved position
- **WHEN** the user presses the card's stop button on an episode that is not reproducing (a non-current card, or the current episode stopped or paused)
- **THEN** the episode's saved position is reset to 0, its listened mark is kept, and no other episode's playback is affected

#### Scenario: Persistent bar stop never resets a saved position
- **WHEN** the user presses the persistent bar's stop button
- **THEN** the shared element halts (or converges to the stopped state) and no saved position is changed