## Purpose

Defines a single app-wide audio player shared across the Vue 3 SPA. One `<audio>` element is owned by a global Pinia store; a persistent bottom bar surfaces playback state and controls for every episode, replacing per-card players.

## Requirements

### Requirement: Single shared audio source owned by a global store

The app SHALL own exactly one `<audio>` element managed by a global audio player Pinia store. The store SHALL hold the currently loaded episode (its media URL, title, thumbnail, channel slug, and yt_id) and the live playback state (playing, current time, duration, volume, muted, playback rate, loading). All player UI in the app SHALL drive and read this single store; there SHALL NOT be multiple concurrent `<audio>` elements playing different sources.

#### Scenario: Starting playback loads the shared element
- **WHEN** the user presses play on any episode
- **THEN** the shared store sets that episode as the current source, loads its `/media/{slug}/{yt_id}.mp3` URL into the single `<audio>` element, and playback starts

#### Scenario: Playing a second episode swaps the source
- **WHEN** the user presses play on a different episode while another is playing
- **THEN** the shared element stops the previous source and loads the new episode's media URL

### Requirement: Persistent bottom player bar

The app SHALL render a persistent player bar fixed to the bottom of the viewport, spanning the full width, overlaying page content. The bar SHALL be hidden by default. When playback starts on any episode, the bar SHALL appear with an upward slide animation. The bar SHALL display the current episode's thumbnail, title, and controls for play/pause, stop, position (scrubber), volume (mute + level), and playback speed.

#### Scenario: Bar is hidden before any playback
- **WHEN** the app loads and no episode has been played yet
- **THEN** the persistent bar is not visible

#### Scenario: Persistent bar appears on playback
- **WHEN** the user starts playback on an episode
- **THEN** the persistent bar slides up into view at the bottom of the screen showing that episode's thumbnail, title, and the standard controls

#### Scenario: Persistent bar reflects shared state
- **WHEN** playback state changes through any control (card or bar)
- **THEN** both the episode card and the persistent bar reflect the same playing status, position, volume, and speed

### Requirement: Episode card controls are bound to the shared player

Each `EpisodeCard` SHALL bind its play/pause, seek, volume, and speed controls to the global audio store instead of owning a private `<audio>` element. Starting playback in a card SHALL start the shared player; toggling pause in the card SHALL pause the shared element; seeking, volume, and speed changes in the card SHALL apply to the shared element and stay visible in the persistent bar.

#### Scenario: Card play starts the persistent bar
- **WHEN** the user presses play in an episode card
- **THEN** the shared element plays that episode and the persistent bar appears, synchronized with the card

#### Scenario: Card and bar controls are interchangeable
- **WHEN** the user toggles pause via the persistent bar while the card is playing, or seeks via the card while the bar shows position
- **THEN** the shared audio state updates and both the card and the bar reflect the change

#### Scenario: Card shows the active episode state
- **WHEN** the shared player is playing a given episode
- **THEN** that episode's card shows the paused/playing state consistent with the shared player

### Requirement: Episode switching from a card retargets the bar

When the user starts playback on an episode that is not the current one, the shared player SHALL replace its current source with the newly selected episode, and the persistent bar SHALL display the new episode's thumbnail and title.

#### Scenario: Switching episodes updates the bar
- **WHEN** the persistent bar is playing episode A and the user presses play on episode B in the list
- **THEN** the shared element stops A, loads B, and the bar now shows B's thumbnail and title while playback continues

### Requirement: Animated auto-hide on stop

When playback stops (user presses stop, or the audio reaches its end), the persistent bar SHALL remain visible for 10 seconds and then disappear with a downward slide animation. Any new play action SHALL bring the bar back with an upward slide animation. While audio is playing or paused mid-track (not stopped), the bar SHALL remain visible.

#### Scenario: Bar hides after stop with delay and animation
- **WHEN** the user presses stop and no new playback starts
- **THEN** the bar stays visible for 10 seconds, then animates downward and is removed from view

#### Scenario: Play resumes before the hide delay
- **WHEN** the user presses play again within the 10-second delay after stop
- **THEN** the bar stays visible and playback resumes without disappearing

#### Scenario: Paused mid-track keeps the bar visible
- **WHEN** the user pauses an episode without stopping it
- **THEN** the bar remains visible and shows the paused state; it does not auto-hide

### Requirement: Stop action clears playback

The stop control SHALL halt playback, reset the position to zero, and mark the player as stopped (distinct from paused). After stop, the position scrubber SHALL reset and the bar SHALL begin its auto-hide delay.

#### Scenario: Stop resets position
- **WHEN** the user presses stop while the episode is at 3:00
- **THEN** playback halts, the current time resets to 0, and the bar begins its 10-second auto-hide
