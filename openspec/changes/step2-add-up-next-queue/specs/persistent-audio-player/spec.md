## Purpose

Extends the persistent bottom player bar with next/previous navigation and an "Up next" queue panel, layered on top of the existing playback controls.

## MODIFIED Requirements

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