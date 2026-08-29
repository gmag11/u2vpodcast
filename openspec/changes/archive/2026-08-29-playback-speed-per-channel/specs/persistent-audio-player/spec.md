## MODIFIED Requirements

### Requirement: Persistent bottom player bar

The app SHALL render a persistent player bar fixed to the bottom of the viewport, spanning the full width, overlaying page content. The bar SHALL be hidden by default. When playback starts on any episode, the bar SHALL appear with an upward slide animation. The bar SHALL display the current episode's thumbnail, title, and controls for play/pause, stop, position (scrubber), volume (mute + level), and playback speed. The playback speed control SHALL offer the standard presets (0.5x, 1x, 1.25x, 1.5x, 2x) and a fine-grained stepper with + and − buttons that adjusts the rate in half-tenths (0.05 steps) within the supported range, so values such as 1.35x or 1.7x can be selected; both paths SHALL drive the shared player's speed state immediately. In addition, the bar SHALL expose a previous control and an "Up next" queue panel (a toggle button opening an overlay or popover) listing the upcoming episodes with remove control, as specified by the `up-next-queue` capability. The next control already present from the `auto-advance` capability (step 1) is retained and gains the dual short/long press behavior of that capability. The bar SHALL only be rendered while an authenticated session exists: on the login screen it SHALL not appear, and losing the session SHALL stop any playback.

#### Scenario: Bar is hidden before any playback
- **WHEN** the app loads and no episode has been played yet
- **THEN** the persistent bar is not visible

#### Scenario: Persistent bar appears on playback
- **WHEN** the user starts playback on an episode
- **THEN** the persistent bar slides up into view at the bottom of the screen showing that episode's thumbnail, title, and the standard controls

#### Scenario: Persistent bar reflects shared state
- **WHEN** playback state changes through any control (card or bar)
- **THEN** both the episode card and the persistent bar reflect the same playing status, position, volume, and speed

#### Scenario: Speed control adjusts in half-tenths
- **WHEN** the user presses the + button in the speed control while the current speed is 1.3x
- **THEN** the speed changes to 1.35x, playback rate updates immediately, and the displayed label shows 1.35x

#### Scenario: Speed control supports presets and fine steps
- **WHEN** the user opens the speed control
- **THEN** it shows the standard presets (0.5x, 1x, 1.25x, 1.5x, 2x) plus + and − stepper buttons that move the rate in 0.05 steps within the supported range

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