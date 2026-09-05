## MODIFIED Requirements

### Requirement: Persistent bottom player bar

The app SHALL render a persistent player bar fixed to the bottom of the viewport, spanning the full width, overlaying page content. The bar SHALL be hidden by default. When playback starts on any episode, the bar SHALL appear with an upward slide animation. The bar SHALL only be rendered while an authenticated session exists: on the login screen it SHALL not appear, and losing the session SHALL stop any playback.

The bar SHALL provide two viewport-dependent compositions using the app's existing small breakpoint (640px) as the boundary.

**Wide composition (viewport width >= 640px)** — the bar SHALL be a fixed-height (non-expandable) single row. Along its top edge the bar SHALL render an interactive progress track spanning the full width of the bar, showing elapsed playback proportion and, when present, chapter and SponsorBlock segment markers. The visual track SHALL be thin, but its interactive hit area SHALL extend beyond the visible track so it can be targeted and dragged for precise seeking across the full bar width; activating it SHALL seek playback to the chosen position. The thumbnail SHALL be rendered statically (not clickable). Beside the thumbnail the bar SHALL show the elapsed/total time readout (`elapsed / total`), using tabular numerals, and a three-line metadata block: the current episode's title on the first line, the current chapter title (when within a chapter) on the second line, and the channel name on the third line. The title SHALL use the shared scrolling text behavior: when the title is wider than the available space it SHALL scroll horizontally from right to left while playback is active, and SHALL be truncated rather than scrolled while playback is not active or when the user's system requests reduced motion. The bar SHALL display controls for play/pause, stop, position (the interactive scrubber), volume (mute + level), and playback speed. The playback speed control SHALL offer the standard presets (0.5x, 1x, 1.25x, 1.5x, 2x) and a fine-grained stepper with + and − buttons that adjusts the rate in half-tenths (0.05 steps) within the supported range, so values such as 1.35x or 1.7x can be selected; both paths SHALL drive the shared player's speed state immediately. In addition, the bar SHALL expose a previous control and an "Up next" queue panel (a toggle button opening an overlay or popover) listing the upcoming episodes with remove control, as specified by the `up-next-queue` capability. The next control from the `auto-advance` capability is retained with its dual short/long press behavior.

When the current episode has stored chapters, the wide composition SHALL also expose a "Chapters" control that opens a popover (mirroring the queue panel pattern) with previous-chapter and next-chapter controls and the full chapter list. Activating a chapter row SHALL seek playback to that chapter's start, subject to the existing rejected-interval skip behavior when that time falls inside a segment marked as rejected. The row for the chapter containing the current playback position SHALL be visually highlighted and SHALL update as playback progresses. Previous-chapter and next-chapter SHALL behave as in the expanded view (restart the current chapter when more than 3 seconds in, otherwise jump to the preceding chapter; advance to the following chapter; disable at boundaries). When the current episode has no stored chapters, the wide composition SHALL render no Chapters control.

**Compact composition (viewport width < 640px)** — the bar SHALL display exactly the following and nothing else:
- a read-only progress track spanning the full width of the bar along its top edge, showing elapsed playback proportion and, when SponsorBlock data is available and enabled, its segment markers;
- the current episode's thumbnail rendered as a square;
- the episode title on a single line; when the title is wider than the available space it SHALL scroll horizontally from right to left while playback is active, and SHALL be truncated rather than scrolled while playback is not active or when the user's system requests reduced motion;
- the current episode's channel name;
- the elapsed playback position as a single clock value using the minimal-unit format `M:SS`, `MM:SS`, or `H:MM:SS` (for example `0:00`, `11:00`, `1:00:00`); the total duration SHALL NOT be shown;
- a play/pause control aligned to the trailing edge of the bar.

In the compact composition the stop, previous, next, playback speed, shuffle, repeat, volume/mute and "Up next" queue controls SHALL NOT be rendered, and the progress track SHALL NOT accept seek interaction of any kind (pointer, keyboard, or assistive-technology value change). Playback state itself, including queue contents, auto-advance, shuffle, repeat, speed and volume, SHALL be unaffected by which composition is displayed.

Switching between compositions SHALL be driven purely by viewport width and SHALL NOT interrupt, restart or reposition playback.

#### Scenario: Bar is hidden before any playback
- **WHEN** the app loads and no episode has been played yet
- **THEN** the persistent bar is not visible

#### Scenario: Persistent bar appears on playback
- **WHEN** the user starts playback on an episode
- **THEN** the persistent bar slides up into view at the bottom of the screen showing that episode's thumbnail, title, and the controls of the composition matching the current viewport width

#### Scenario: Persistent bar reflects shared state
- **WHEN** playback state changes through any control (card or bar)
- **THEN** both the episode card and the persistent bar reflect the same playing status, position, volume, and speed

#### Scenario: Speed control adjusts in half-tenths
- **WHEN** the user presses the + button in the speed control while the current speed is 1.3x on a viewport of at least 640px
- **THEN** the speed changes to 1.35x, playback rate updates immediately, and the displayed label shows 1.35x

#### Scenario: Speed control supports presets and fine steps
- **WHEN** the user opens the speed control on a viewport of at least 640px
- **THEN** it shows the standard presets (0.5x, 1x, 1.25x, 1.5x, 2x) plus + and − stepper buttons that move the rate in 0.05 steps within the supported range

#### Scenario: Bar exposes next/previous and queue toggle
- **WHEN** the bar is visible on a viewport of at least 640px
- **THEN** it also shows next/previous controls (disabled or enabled per queue emptiness and playback history) and a button that opens the "Up next" panel

#### Scenario: Queue panel opens from the bar
- **WHEN** the user presses the queue button in the visible bar on a viewport of at least 640px
- **THEN** an "Up next" panel opens listing the upcoming episodes with a per-item remove action, and closes on a second press or outside interaction

#### Scenario: Wide scrubber spans the full bar width
- **WHEN** the wide composition is displayed and an episode is loaded
- **THEN** the interactive progress track spans the full width of the bar along its top edge, and clicking or dragging anywhere on it seeks playback to that position

#### Scenario: Wide scrubber has an extended hit area
- **WHEN** the user targets the wide scrubber
- **THEN** the clickable/draggable region extends above and below the thin visible track so the track is easy to target while remaining visually thin

#### Scenario: Wide metadata shows title, chapter and channel lines
- **WHEN** the wide composition is displayed for an episode with stored chapters and playback is within a chapter's range
- **THEN** the metadata block shows the episode title on the first line, the current chapter title on the second line, and the channel name on the third line

#### Scenario: Wide metadata omits chapter when absent
- **WHEN** the wide composition is displayed and the current episode has no stored chapters or playback is before the first chapter's start
- **THEN** the second metadata line is omitted and the channel name remains on its own line

#### Scenario: Chapters popover opens from the wide bar
- **WHEN** the wide composition is displayed for an episode that has stored chapters and the user activates the "Chapters" control
- **THEN** a popover opens showing previous/next chapter controls and the full chapter list with title and start time in order

#### Scenario: Chapter row seek from the wide popover
- **WHEN** the user activates a chapter row in the wide popover
- **THEN** the shared player seeks to that chapter's start time

#### Scenario: Chapter row seek respects rejected intervals
- **WHEN** the user activates a chapter row in the wide popover whose start time falls inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking

#### Scenario: Wide popover highlights the current chapter
- **WHEN** playback position falls within a chapter's range and the wide popover is open
- **THEN** that chapter's row is visually highlighted and the highlight follows playback across boundaries

#### Scenario: Wide chapter navigation matches expanded behavior
- **WHEN** the user activates previous/next chapter in the wide popover
- **THEN** navigation behaves exactly as the expanded view: previous restarts the current chapter when more than 3 seconds in otherwise moves to the preceding chapter; next advances to the following chapter; controls disable at boundaries

#### Scenario: No Chapters control without stored chapters
- **WHEN** the wide composition is displayed and the current episode has no stored chapters
- **THEN** no Chapters control is rendered

#### Scenario: Wide time readout sits beside the thumbnail
- **WHEN** the wide composition is displayed
- **THEN** the elapsed/total time readout (for example `11:09 / 45:00`) is shown beside the thumbnail using tabular numerals

#### Scenario: Wide title scrolls when it overflows
- **WHEN** the wide composition is displayed, playback is active, and the episode title is wider than the space available for it
- **THEN** the title scrolls horizontally from right to left so its full text becomes readable over time

#### Scenario: Wide title does not scroll while paused
- **WHEN** the wide composition is displayed, an overflowing title is scrolling, and the user pauses playback
- **THEN** the title stops scrolling and is truncated

#### Scenario: Wide title respects reduced motion
- **WHEN** the wide composition is displayed, the user's system requests reduced motion, and an overflowing title is shown during playback
- **THEN** the title does not animate and is truncated instead

#### Scenario: Wide thumbnail is not interactive
- **WHEN** the user clicks the thumbnail in the wide composition
- **THEN** no expanded view opens and playback is unaffected

#### Scenario: Wide bar retains all controls
- **WHEN** the wide composition is displayed
- **THEN** it shows previous, play/pause, stop, next, the full-width scrubber, speed, shuffle, repeat, mute/volume, and the "Up next" queue toggle

#### Scenario: Compact bar shows only the reduced information set
- **WHEN** an episode is loaded and the viewport is narrower than 640px
- **THEN** the bar shows a full-width read-only progress track, a square thumbnail, the episode title, the channel name, the elapsed-time clock and a play/pause control, and shows no stop, previous, next, speed, shuffle, repeat, volume or queue control

#### Scenario: Compact clock shows elapsed time only
- **WHEN** the viewport is narrower than 640px and playback is at 11 minutes 9 seconds of a 45-minute episode
- **THEN** the bar shows `11:09` and does not show the total duration

#### Scenario: Compact clock uses hours only when needed
- **WHEN** the viewport is narrower than 640px and playback passes one hour
- **THEN** the clock switches from `MM:SS` to `H:MM:SS`

#### Scenario: Compact title scrolls when it overflows
- **WHEN** the viewport is narrower than 640px, playback is active, and the episode title is wider than the space available for it
- **THEN** the title scrolls horizontally from right to left so its full text becomes readable over time

#### Scenario: Compact title does not scroll when it fits
- **WHEN** the viewport is narrower than 640px and the episode title fits within the space available for it
- **THEN** the title is shown statically without scrolling

#### Scenario: Compact title stops scrolling while paused
- **WHEN** the viewport is narrower than 640px, an overflowing title is scrolling, and the user pauses playback
- **THEN** the title stops scrolling and is truncated

#### Scenario: Compact title respects reduced motion
- **WHEN** the viewport is narrower than 640px, the user's system requests reduced motion, and an overflowing title is displayed during playback
- **THEN** the title does not animate and is truncated instead

#### Scenario: Compact play/pause remains functional
- **WHEN** the user presses the play/pause control in the compact bar
- **THEN** the shared player toggles between playing and paused exactly as it does from the wide composition

#### Scenario: Compact progress track rejects seeking
- **WHEN** the viewport is narrower than 640px and the user taps, drags or otherwise interacts with the progress track
- **THEN** the playback position does not change

#### Scenario: Composition follows viewport width changes
- **WHEN** the viewport is resized across the 640px boundary while an episode is playing
- **THEN** the bar swaps to the other composition and playback continues uninterrupted at the same position

#### Scenario: Bar is absent on the login screen
- **WHEN** the user is not authenticated (the login screen is shown)
- **THEN** the player bar is not rendered and no playback is active

#### Scenario: Losing the session stops playback
- **WHEN** the session disappears (logout) while playback was active
- **THEN** playback stops and the bar no longer renders
