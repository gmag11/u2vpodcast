## ADDED Requirements

### Requirement: Expanded now-playing view for the compact composition

While the compact composition (viewport width < 640px) is displayed, tapping the persistent bar's thumbnail SHALL open a full-screen expanded "now playing" view that slides up from the bottom over the current page content. The expanded view SHALL NOT change or interrupt playback; it only changes what is visible.

The expanded view SHALL display:
- a close control (a chevron-down icon) in its top-left corner;
- the current episode's thumbnail, rendered larger than in the compact bar;
- the episode title and the current episode's channel name; when the title is wider than the available space it SHALL scroll continuously from right to left while playback is active, and SHALL be truncated rather than scrolled while playback is not active or when the user's system requests reduced motion;
- an interactive progress bar spanning the width of the view, together with the elapsed time and the remaining time (or total duration) as separate labels flanking it;
- a playback speed control equivalent to the wide composition's (standard presets plus the fine-grained +/- stepper in 0.05 steps);
- a combined shuffle/repeat control, as specified by the `playback-modes` capability;
- an "Up next" queue toggle equivalent to the wide composition's, opening the same queue panel;
- transport controls: previous, seek-back-10-seconds, play/pause, seek-forward-10-seconds, and next.

The expanded view SHALL NOT display a volume or mute control.

The expanded view need not occupy the entire vertical viewport; it SHALL be dismissible and SHALL NOT block interaction with the rest of the app while closed.

The expanded view SHALL only be reachable while the compact composition is active. If the viewport is resized to >= 640px while the expanded view is open, the expanded view SHALL close and the wide composition SHALL be shown.

#### Scenario: Tapping the compact bar's thumbnail opens the expanded view
- **WHEN** the viewport is narrower than 640px, the persistent bar is visible, and the user taps the thumbnail
- **THEN** the expanded "now playing" view slides up from the bottom over the page content, showing the current episode's details and controls, and playback continues unaffected

#### Scenario: Expanded view shows an interactive scrubber
- **WHEN** the expanded view is open
- **THEN** the progress bar accepts pointer, keyboard, and assistive-technology interaction to seek to any position in the track, and the elapsed and remaining time labels update accordingly

#### Scenario: Seeking from the expanded view updates playback
- **WHEN** the user drags or taps the expanded view's progress bar to a new position
- **THEN** the shared player seeks to that position and the compact bar (once the expanded view closes) reflects the same position

#### Scenario: Expanded view exposes speed, transport, and queue controls
- **WHEN** the expanded view is open
- **THEN** it shows the playback speed control (presets and stepper), previous, seek-back-10s, play/pause, seek-forward-10s, next, the combined shuffle/repeat control, and the "Up next" queue toggle, all bound to the shared player state

#### Scenario: Expanded view has no volume control
- **WHEN** the expanded view is open
- **THEN** no volume or mute control is rendered anywhere in the view

#### Scenario: Closing the expanded view returns to the compact bar
- **WHEN** the user presses the chevron-down close control
- **THEN** the expanded view closes, the compact bar is shown again, and playback is unaffected

#### Scenario: Opening the queue panel from the expanded view
- **WHEN** the user presses the "Up next" toggle in the expanded view
- **THEN** the same queue panel used by the wide composition opens, listing upcoming episodes with a per-item remove action

#### Scenario: Widening the viewport while expanded closes the expanded view
- **WHEN** the expanded view is open and the viewport is resized to 640px or wider
- **THEN** the expanded view closes and the wide composition of the persistent bar is shown, with playback continuing uninterrupted at the same position

#### Scenario: Expanded view reflects state changes made elsewhere
- **WHEN** the expanded view is open and playback state changes through another control (for example, a card action)
- **THEN** the expanded view's progress, play/pause state, speed, and mode indicators update to match

#### Scenario: Expanded title scrolls continuously while playing
- **WHEN** the expanded view is open, playback is active, and the episode title is wider than the available space
- **THEN** the title scrolls continuously from right to left so its full text remains readable over time

#### Scenario: Expanded title stops scrolling while paused or reduced motion is requested
- **WHEN** the expanded view shows an overflowing title and playback is not active or the user's system requests reduced motion
- **THEN** the title does not animate and is truncated instead
