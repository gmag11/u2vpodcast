## MODIFIED Requirements

### Requirement: Persistent bottom player bar

The app SHALL render a persistent player bar fixed to the bottom of the viewport, spanning the full width, overlaying page content. The bar SHALL be hidden by default. When playback starts on any episode, the bar SHALL appear with an upward slide animation. The bar SHALL only be rendered while an authenticated session exists: on the login screen it SHALL not appear, and losing the session SHALL stop any playback.

The bar SHALL provide two viewport-dependent compositions using the app's existing small breakpoint (640px) as the boundary.

**Wide composition (viewport width >= 640px)** — unchanged from previous behavior. The bar SHALL display the current episode's thumbnail, title, and controls for play/pause, stop, position (interactive scrubber), volume (mute + level), and playback speed. The playback speed control SHALL offer the standard presets (0.5x, 1x, 1.25x, 1.5x, 2x) and a fine-grained stepper with + and − buttons that adjusts the rate in half-tenths (0.05 steps) within the supported range, so values such as 1.35x or 1.7x can be selected; both paths SHALL drive the shared player's speed state immediately. In addition, the bar SHALL expose a previous control and an "Up next" queue panel (a toggle button opening an overlay or popover) listing the upcoming episodes with remove control, as specified by the `up-next-queue` capability. The next control from the `auto-advance` capability is retained with its dual short/long press behavior.

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
