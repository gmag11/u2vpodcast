## Purpose

Defines a single app-wide audio player shared across the Vue 3 SPA. One `<audio>` element is owned by a global Pinia store; a persistent bottom bar surfaces playback state and controls for every episode, replacing per-card players.

## Requirements

### Requirement: Timeline displays chapter markers on the original media timeline
When an episode has stored chapters, the persistent player's progress track SHALL render a marker at each chapter's original start time, using the episode's stored, untranslated chapter times against the original media duration — consistent with the shared player always operating on the original timeline. This applies to the wide composition's interactive scrubber, the expanded view's scrubber, and the compact composition's read-only track. Chapter markers SHALL be visually distinct from SponsorBlock segment markers and from the playback-progress fill. On an interactive scrubber (wide or expanded), hovering or keyboard-focusing a chapter marker SHALL show an immediate styled tooltip containing the chapter title, and activating it SHALL seek playback to that chapter's start time, subject to the existing rejected-interval skip behavior when that time falls inside a segment marked as rejected. The compact composition's read-only track SHALL render chapter markers without accepting seek interaction, consistent with its existing no-seek behavior. An episode with no stored chapters SHALL render no chapter markers.

#### Scenario: Chapter markers appear on the wide scrubber
- **WHEN** the wide composition is displayed for an episode that has stored chapters
- **THEN** a marker appears at each chapter's original start time along the scrubber, distinct from any SponsorBlock markers present

#### Scenario: Chapter markers appear on the expanded scrubber
- **WHEN** the expanded "now playing" view is open for an episode that has stored chapters
- **THEN** the same chapter markers appear on its scrubber at the same relative positions as the wide composition

#### Scenario: Chapter markers appear on the compact track
- **WHEN** the compact composition is displayed for an episode that has stored chapters
- **THEN** chapter markers appear on the read-only progress track and do not accept seek interaction

#### Scenario: Activating a chapter marker seeks to its start
- **WHEN** the user clicks or taps a chapter marker on the wide or expanded scrubber
- **THEN** the shared player seeks to that chapter's original start time

#### Scenario: Interactive chapter marker identifies its chapter
- **WHEN** the user hovers or keyboard-focuses a chapter marker on the wide or expanded scrubber
- **THEN** a styled tooltip immediately shows that chapter's title

#### Scenario: Chapter marker falls inside a rejected SponsorBlock interval
- **WHEN** the user activates a chapter marker whose start time falls inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking, landing at the end of the complete overlapping rejected interval

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** no chapter markers are rendered on any composition's track

#### Scenario: Chapter and SponsorBlock markers are visually distinguishable
- **WHEN** a progress track shows both chapter markers and SponsorBlock segment markers
- **THEN** the two marker types use distinct visual treatments so neither is mistaken for the other

### Requirement: Single shared audio source owned by a global store

The app SHALL own exactly one `<audio>` element managed by a global audio player Pinia store. The store SHALL hold the currently loaded episode (its media URL, title, thumbnail, channel slug, and yt_id) and the live playback state (playing, current time, duration, volume, muted, playback rate, loading). All player UI in the app SHALL drive and read this single store; there SHALL NOT be multiple concurrent `<audio>` elements playing different sources.

#### Scenario: Starting playback loads the shared element
- **WHEN** the user presses play on any episode
- **THEN** the shared store sets that episode as the current source, loads its `/media/{slug}/{yt_id}.mp3` URL into the single `<audio>` element, and playback starts

#### Scenario: Playing a second episode swaps the source
- **WHEN** the user presses play on a different episode while another is playing
- **THEN** the shared element stops the previous source and loads the new episode's media URL

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

### Requirement: Web playback skips configured rejected intervals on the original timeline
The shared player SHALL continue loading the original `/media/{slug}/{yt_id}.mp3` source. When SponsorBlock is enabled, it SHALL use the normalized categorized SponsorBlock segments included in the episode payload. Whenever the playhead enters a segment marked as rejected, the player SHALL seek to the end of the complete overlapping rejected interval. Segments not marked as rejected SHALL remain playable. Playback position, duration, seeking, completion, and persisted progress SHALL remain expressed on the original MP3 timeline. Episode-card and persistent-player progress tracks SHALL display all SponsorBlock segments whenever enabled data is available, including before playback and while paused; this applies to both the interactive wide-composition scrubber and the read-only compact-composition track, which SHALL use the same segment colors and positions. `sponsor` segments SHALL use the existing sponsor color and every other category SHALL use a second color distinct from both sponsor markers and playback progress. When SponsorBlock is disabled, the player SHALL perform no SponsorBlock skips and SHALL render no SponsorBlock markers.

#### Scenario: Playback enters a rejected interval
- **WHEN** normal playback reaches a segment marked as rejected from original-media time 120 to 150
- **THEN** the shared player seeks to the end of the complete overlapping rejected interval and continues playback

#### Scenario: User seeks into a rejected interval
- **WHEN** the user moves the scrubber or uses a relative seek to a time inside a segment marked as rejected
- **THEN** the player advances to the end of the complete overlapping rejected interval

#### Scenario: Playback resumes inside a rejected interval
- **WHEN** persisted progress points inside a segment marked as rejected
- **THEN** resume advances past the complete overlapping rejected interval instead of playing it

#### Scenario: Playback enters a non-rejected segment
- **WHEN** normal playback reaches a segment whose category is not configured for rejection
- **THEN** playback continues through that segment without an automatic seek

#### Scenario: Progress is persisted after a skip
- **WHEN** the player skips a rejected interval ending at original-media time 150
- **THEN** subsequent progress writes and labels continue using the original timeline at or after 150

#### Scenario: Episode has no stored segments
- **WHEN** an episode payload has an empty or unavailable SponsorBlock snapshot
- **THEN** the shared player behaves exactly as ordinary original-MP3 playback

### Requirement: Expanded view lists chapters with jump-to-chapter interaction

When the current episode has stored chapters, the expanded "now playing" view
SHALL display a "Chapters" section listing every chapter's title and start time
in order. Activating a chapter row SHALL seek playback to that chapter's start
time, subject to the existing rejected-interval skip behavior when that time
falls inside a segment marked as rejected. The row corresponding to the chapter
containing the current playback position SHALL be visually highlighted and SHALL
update as playback progresses. When the current episode has no stored chapters,
the expanded view SHALL display no Chapters section.

#### Scenario: Chapters section appears for an episode with chapters
- **WHEN** the expanded view is open for an episode that has stored chapters
- **THEN** a Chapters section lists every chapter's title and start time in order

#### Scenario: Activating a chapter row seeks to its start
- **WHEN** the user taps or clicks a chapter row
- **THEN** the shared player seeks to that chapter's start time

#### Scenario: Chapter row seek respects rejected intervals
- **WHEN** the user activates a chapter row whose start time falls inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking

#### Scenario: Current chapter is highlighted
- **WHEN** playback position falls within a chapter's time range
- **THEN** that chapter's row is visually highlighted while no other row is

#### Scenario: Highlight follows playback
- **WHEN** playback advances past a chapter boundary
- **THEN** the highlighted row updates to the new current chapter without user interaction

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** the expanded view shows no Chapters section

#### Scenario: SponsorBlock is disabled during playback
- **WHEN** SponsorBlock is disabled regardless of stored snapshot or rejected-category configuration
- **THEN** playback performs no SponsorBlock seek and episode-card and persistent-player tracks show no SponsorBlock markers

#### Scenario: Progress tracks show all segments while idle
- **WHEN** an idle or paused episode has rejected and non-rejected SponsorBlock segments
- **THEN** its episode-card progress track and persistent-player track display every segment

#### Scenario: Marker colors distinguish sponsor category
- **WHEN** a progress track contains `sponsor` and non-sponsor category segments
- **THEN** sponsor segments use the existing sponsor marker color and all non-sponsor segments use the distinct secondary marker color regardless of rejection status

#### Scenario: Compact track shows segments without seeking
- **WHEN** the viewport is narrower than 640px and the current episode has SponsorBlock enabled with stored segments
- **THEN** the compact read-only track renders every segment with the same colors and relative positions as the wide scrubber while accepting no seek interaction

### Requirement: Refreshed segment snapshots take effect without replacing the source
When SponsorBlock is enabled and an authenticated refresh returns changed SponsorBlock segment data or rejection metadata for the current episode, the player SHALL replace its active segment set without reloading or replacing the original MP3 source. An identical snapshot hash and identical segment data SHALL leave player state unchanged. When SponsorBlock is disabled, the frontend SHALL expose no SponsorBlock refresh action and SHALL discard any active SponsorBlock segment set without changing the source or playhead.

#### Scenario: Manual refresh changes current segments
- **WHEN** refresh returns changed segments or rejection metadata for the currently loaded episode
- **THEN** later playback and seeks use the new rejected intervals while the current original media source and playhead are retained

#### Scenario: Manual refresh changes only playable segments
- **WHEN** refresh changes only non-rejected categorized segments for the currently loaded episode
- **THEN** timeline markers update while playback behavior, current source, and playhead are retained

#### Scenario: Manual refresh is unchanged
- **WHEN** refresh returns the same snapshot hash and segment data the episode already holds
- **THEN** the player performs no source reload or playhead change

#### Scenario: SponsorBlock is disabled for the current episode
- **WHEN** the frontend receives episode data with SponsorBlock disabled
- **THEN** no refresh action or active SponsorBlock segments remain while the source and playhead are retained

### Requirement: Player compositions show the current chapter title

When the current episode has stored chapters, the persistent player's compact
and wide compositions and the expanded "now playing" view SHALL display the
title of the chapter containing the current playback position as a secondary
label near the episode title. The label SHALL update without requiring user
interaction as playback crosses chapter boundaries. When the current episode has
no stored chapters, or playback position is before the first chapter's start, no
chapter label SHALL be shown, and no layout space SHALL be reserved for it. In
the compact composition, the label SHALL appear between the episode title and
the channel/playback-time line.

#### Scenario: Chapter label shown for an episode with chapters
- **WHEN** any player composition is displayed for an episode with stored chapters and playback is within a chapter's range
- **THEN** that chapter's title is shown as a secondary label near the episode title

#### Scenario: Label updates across chapter boundaries
- **WHEN** playback advances from one chapter into the next
- **THEN** the displayed label updates to the new current chapter's title without user interaction

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** no chapter label is shown and no layout space is reserved for it

#### Scenario: Compact composition places the chapter label between existing metadata
- **WHEN** the compact composition (viewport narrower than 640px) is displayed
- **THEN** the current chapter title is rendered between the episode title and the channel/playback-time line

### Requirement: Expanded view offers chapter-level previous/next navigation

When the current episode has stored chapters, the expanded "now playing" view
SHALL display previous-chapter and next-chapter controls, distinct from the
existing episode-level previous/next controls. Activating next-chapter SHALL
seek to the start of the chapter immediately after the one containing the
current playback position; if the current position is within the last chapter,
the control SHALL be disabled. Activating previous-chapter SHALL restart the
current chapter (seek to its start) when more than 3 seconds have elapsed since
that chapter's start, and SHALL otherwise seek to the start of the preceding
chapter; if the current position is within the first chapter and at or before 3
seconds into it, the control SHALL be disabled. Both controls SHALL apply
existing rejected-interval skip behavior after seeking when SponsorBlock is
enabled. When the current episode has no stored chapters, neither control SHALL
be rendered.

#### Scenario: Next-chapter advances to the following chapter
- **WHEN** the user activates next-chapter while playback is within a chapter that has a following chapter
- **THEN** the player seeks to the start of the following chapter

#### Scenario: Next-chapter is disabled in the last chapter
- **WHEN** playback is within the episode's last chapter
- **THEN** the next-chapter control is disabled

#### Scenario: Previous-chapter restarts the current chapter
- **WHEN** the user activates previous-chapter more than 3 seconds after the current chapter's start
- **THEN** the player seeks to the start of the current chapter

#### Scenario: Previous-chapter moves to the preceding chapter
- **WHEN** the user activates previous-chapter within 3 seconds of the current chapter's start and a preceding chapter exists
- **THEN** the player seeks to the start of the preceding chapter

#### Scenario: Previous-chapter is disabled at the episode's first chapter start
- **WHEN** playback is within 3 seconds of the first chapter's start
- **THEN** the previous-chapter control is disabled

#### Scenario: Chapter navigation respects rejected intervals
- **WHEN** a chapter-navigation seek lands on a time inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** neither chapter-navigation control is rendered

### Requirement: Global spacebar toggles active playback

While the application document has focus, the shared player SHALL treat a spacebar key press as a global play/pause toggle when an episode is playing or paused. The shortcut SHALL pause playing audio and resume paused audio. It SHALL have no effect when the player is stopped or no episode is loaded. The shortcut SHALL NOT override spacebar behavior while focus is in an editable field or an interactive control with native spacebar behavior.

#### Scenario: Spacebar pauses playing audio
- **WHEN** an episode is playing, the application document has focus, and the user presses the spacebar outside an editable field or interactive control
- **THEN** the shared player pauses the episode and prevents the page's default spacebar action

#### Scenario: Spacebar resumes paused audio
- **WHEN** an episode is paused, the application document has focus, and the user presses the spacebar outside an editable field or interactive control
- **THEN** the shared player resumes the episode and prevents the page's default spacebar action

#### Scenario: Spacebar has no effect after stop
- **WHEN** the player is stopped and the user presses the spacebar
- **THEN** playback remains stopped and the key press does not start or resume an episode

#### Scenario: Spacebar has no effect without a loaded episode
- **WHEN** no episode is loaded and the user presses the spacebar
- **THEN** no playback starts and player state remains unchanged

#### Scenario: Focused controls retain spacebar behavior
- **WHEN** focus is in an editable field or an interactive control with native spacebar behavior and the user presses the spacebar
- **THEN** the shared player state remains unchanged and the focused element retains its normal spacebar behavior

#### Scenario: Unfocused document ignores spacebar
- **WHEN** the application document does not have focus and a spacebar key event occurs
- **THEN** the shared player state remains unchanged
