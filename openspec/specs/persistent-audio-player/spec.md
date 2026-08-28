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

The app SHALL render a persistent player bar fixed to the bottom of the viewport, spanning the full width, overlaying page content. The bar SHALL be hidden by default. When playback starts on any episode, the bar SHALL appear with an upward slide animation. The bar SHALL display the current episode's thumbnail, title, and controls for play/pause, stop, position (scrubber), volume (mute + level), and playback speed. In addition, the bar SHALL expose a previous control and an "Up next" queue panel (a toggle button opening an overlay or popover) listing the upcoming episodes with remove control, as specified by the `up-next-queue` capability. The next control already present from the `auto-advance` capability (step 1) is retained and gains the dual short/long press behavior of that capability. The bar SHALL only be rendered while an authenticated session exists: on the login screen it SHALL not appear, and losing the session SHALL stop any playback.

#### Scenario: Bar is hidden before any playback
- **WHEN** the app loads and no episode has been played yet
- **THEN** the persistent bar is not visible

#### Scenario: Persistent bar appears on playback
- **WHEN** the user starts playback on an episode
- **THEN** the persistent bar slides up into view at the bottom of the screen showing that episode's thumbnail, title, and the standard controls

#### Scenario: Persistent bar reflects shared state
- **WHEN** playback state changes through any control (card or bar)
- **THEN** both the episode card and the persistent bar reflect the same playing status, position, volume, and speed

#### Scenario: Bar exposes next/previous and queue toggle
- **WHEN** the bar is visible
- **THEN** it also shows next/previous controls (disabled or enabled per queue emptiness and playback history) and a button that opens the "Up next" panel

#### Scenario: Queue panel opens from the bar
- **WHEN** the user presses the queue button in the visible bar
- **THEN** an "Up next" panel opens listing the upcoming episodes with a per-item remove action, and closes on a second press or outside interaction

#### Scenario: Bar is absent on the login screen
- **WHEN** the user is not authenticated (the login screen is shown)
- **THEN** the player bar is not rendered and no playback is active

#### Scenario: Losing the session stops playback
- **WHEN** the session disappears (logout) while playback was active
- **THEN** playback stops and the bar no longer renders

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

### Requirement: Episode switching from a card retargets the bar

When the user starts playback on an episode that is not the current one, the shared player SHALL replace its current source with the newly selected episode, and the persistent bar SHALL display the new episode's thumbnail and title.

#### Scenario: Switching episodes updates the bar
- **WHEN** the persistent bar is playing episode A and the user presses play on episode B in the list
- **THEN** the shared element stops A, loads B, and the bar now shows B's thumbnail and title while playback continues

### Requirement: Animated auto-hide on stop

When playback stops (user presses stop, or the audio reaches its end) and the up-next queue is empty, the persistent bar SHALL remain visible for 10 seconds and then disappear with a downward slide animation. When the queue is not empty the bar SHALL NOT auto-hide: it SHALL stay visible so the queue stays accessible for inspection and management. Any new play action SHALL bring the bar back with an upward slide animation. While audio is playing or paused mid-track (not stopped), the bar SHALL remain visible.

#### Scenario: Bar hides after stop with delay and animation
- **WHEN** the user presses stop with an empty queue and no new playback starts
- **THEN** the bar stays visible for 10 seconds, then animates downward and is removed from view

#### Scenario: Play resumes before the hide delay
- **WHEN** the user presses play again within the 10-second delay after stop
- **THEN** the bar stays visible and playback resumes without disappearing

#### Scenario: Paused mid-track keeps the bar visible
- **WHEN** the user pauses an episode without stopping it
- **THEN** the bar remains visible and shows the paused state; it does not auto-hide

#### Scenario: Bar stays visible with queued episodes
- **WHEN** the user stops playback while the up-next queue still holds episodes
- **THEN** the bar remains visible indefinitely (no auto-hide) so the queue panel stays reachable

#### Scenario: Hide resumes once the queue empties
- **WHEN** the queue becomes empty (removed or cleared) while the bar is stopped
- **THEN** the bar starts its 10-second auto-hide delay as usual

### Requirement: Stop action clears playback

The stop control SHALL halt playback, reset the position to zero, and mark the player as stopped (distinct from paused). After stop, the position scrubber SHALL reset and the bar SHALL begin its auto-hide delay.

#### Scenario: Stop resets position
- **WHEN** the user presses stop while the episode is at 3:00
- **THEN** playback halts, the current time resets to 0, and the bar begins its 10-second auto-hide

### Requirement: Shuffle and repeat toggles in the persistent bar

The persistent player bar SHALL expose shuffle and repeat controls reflecting the player's mode state, as specified by the `playback-modes` capability.

#### Scenario: Shuffle toggle visible and reactive
- **WHEN** the bar is visible
- **THEN** a shuffle control is shown and highlights when shuffle mode is active

#### Scenario: Repeat toggle cycles states
- **WHEN** the bar is visible
- **THEN** a repeat control is shown that cycles through none, all, and one, visually indicating the active state

### Requirement: Web playback skips stored sponsor intervals on the original timeline
The shared player SHALL continue loading the original `/media/{slug}/{yt_id}.mp3` source and SHALL use the normalized SponsorBlock segments included in the episode payload. Whenever the playhead enters a sponsor interval, the player SHALL seek to that interval's end. Playback position, duration, seeking, completion, and persisted progress SHALL remain expressed on the original MP3 timeline. Episode-card and persistent-player progress tracks SHALL display sponsor intervals in a distinct color whenever segment data is available, including before playback and while paused.

#### Scenario: Playback enters a sponsor interval
- **WHEN** normal playback reaches the start of a stored sponsor interval `[120, 150]`
- **THEN** the shared player seeks to original-media time 150 and continues playback

#### Scenario: User seeks into a sponsor interval
- **WHEN** the user moves the scrubber or uses a relative seek to a time inside `[120, 150]`
- **THEN** the player advances to original-media time 150

#### Scenario: Playback resumes inside a sponsor interval
- **WHEN** persisted progress points inside a stored sponsor interval
- **THEN** resume advances to the end of that interval instead of playing the sponsor segment

#### Scenario: Progress is persisted after a skip
- **WHEN** the player skips a sponsor interval ending at original-media time 150
- **THEN** subsequent progress writes and labels continue using the original timeline at or after 150

#### Scenario: Episode has no stored segments
- **WHEN** an episode payload has an empty or unavailable SponsorBlock snapshot
- **THEN** the shared player behaves exactly as ordinary original-MP3 playback

#### Scenario: Progress tracks show sponsor intervals while idle
- **WHEN** an episode has stored SponsorBlock intervals and is not currently playing
- **THEN** its episode-card progress track and the paused persistent-player track display those intervals in a color distinct from playback progress

### Requirement: Refreshed segment snapshots take effect without replacing the source
When an authenticated refresh returns a different SponsorBlock hash for the current episode, the player SHALL replace its active segment set with the returned normalized segments without reloading or replacing the original MP3 source. An identical hash SHALL leave player state unchanged.

#### Scenario: Manual refresh changes current segments
- **WHEN** refresh returns a new hash and normalized intervals for the currently loaded episode
- **THEN** later playback and seeks use the new intervals while the current original media source and playhead are retained

#### Scenario: Manual refresh is unchanged
- **WHEN** refresh returns the same hash as the episode already holds
- **THEN** the player performs no source reload or playhead change
