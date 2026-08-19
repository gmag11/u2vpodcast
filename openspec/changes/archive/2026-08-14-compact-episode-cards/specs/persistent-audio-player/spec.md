## MODIFIED Requirements

### Requirement: Episode card controls are bound to the shared player

Each `EpisodeCard` SHALL bind its play/pause and stop controls to the global audio store instead of owning a private `<audio>` element. The card SHALL NOT expose inline seek, volume, or speed controls; those controls SHALL live exclusively in the persistent bar. Starting playback in a card SHALL start the shared player; toggling pause in the card SHALL pause the shared element; stopping in the card SHALL reset and stop the shared element.

#### Scenario: Card play starts the persistent bar
- **WHEN** the user presses play in an episode card
- **THEN** the shared element plays that episode and the persistent bar appears, synchronized with the card

#### Scenario: Card and bar controls are interchangeable
- **WHEN** the user toggles pause via the persistent bar while the card is playing, or uses the scrubber/volume/speed controls in the bar
- **THEN** the shared audio state updates and both the card and the bar reflect the change, with the card reflecting playing/paused state and the bar reflecting position, volume, and speed

#### Scenario: Card shows the active episode state
- **WHEN** the shared player is playing a given episode
- **THEN** that episode's card shows the paused/playing state consistent with the shared player
