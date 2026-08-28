## Purpose

Defines the layout and inline playback controls of the episode card in the Vue 3 SPA. Cards keep a compact vertical footprint by exposing only play/pause and stop bound to the shared player, showing a total-duration-only label, and placing the controls per breakpoint; advanced controls (seek, volume, speed) live exclusively in the persistent bottom player bar.

## Requirements

### Requirement: Compact card footprint without inline advanced controls

Each `EpisodeCard` SHALL render without an inline seek bar, without a live position counter, without a playback speed control, and without a volume control. These controls SHALL remain available only in the persistent bottom player bar. The card SHALL retain play/pause and stop controls.

#### Scenario: Card has no seek bar
- **WHEN** an episode card is rendered
- **THEN** the card shows no seek/scrubber bar and no live position counter

#### Scenario: Card has no speed or volume controls
- **WHEN** an episode card is rendered
- **THEN** the card shows no playback speed control and no volume/mute control

#### Scenario: Advanced controls remain in the persistent bar
- **WHEN** playback is active on an episode
- **THEN** the persistent bar still provides the position scrubber, volume (mute + level), and playback speed controls

### Requirement: Total duration label only

The episode card SHALL display the episode's total duration. When the card's episode is the currently loaded one, the label SHALL come from the shared player's total duration; otherwise it SHALL come from the episode's stored duration. The card SHALL NOT display the current playback position.

#### Scenario: Card shows total duration for the current episode
- **WHEN** the card's episode is currently loaded in the shared player
- **THEN** the card displays the total duration of the episode and no position counter

#### Scenario: Card shows stored duration for other episodes
- **WHEN** the card's episode is not currently loaded in the shared player
- **THEN** the card displays the episode's stored total duration

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
### Requirement: Play/stop placement by breakpoint

On desktop (`sm` and up), play/pause and stop controls SHALL remain positioned below the episode thumbnail. On mobile, default and compact cards outside the playlist SHALL keep the controls to the right of the thumbnail. The playlist-specific mobile presentation SHALL use its episode image as the play/pause control and SHALL expose no stop control. Exactly one presentation for the active context and viewport SHALL be available.

#### Scenario: Buttons below thumbnail on desktop
- **WHEN** any episode card is viewed at the `sm` breakpoint or wider
- **THEN** the play/pause and stop buttons render below the thumbnail using the existing desktop presentation

#### Scenario: Buttons right of thumbnail on mobile
- **WHEN** a default or compact episode card outside the playlist is viewed below the `sm` breakpoint
- **THEN** the play/pause and stop buttons render to the right of the thumbnail using the existing mobile presentation

#### Scenario: Playlist uses its dense mobile controls
- **WHEN** a playlist episode card is viewed below the `sm` breakpoint
- **THEN** the episode image operates play/pause, no stop control is available, and the specified secondary actions are accessed from its overflow menu

#### Scenario: Only one placement renders
- **WHEN** an episode card is rendered in any context and viewport width
- **THEN** only the presentation selected for that context and width is available

### Requirement: Playlist mobile presentation preserves episode-card behavior

An episode card selected for the playlist-specific mobile presentation SHALL preserve the shared player binding, total-duration value, playback progress and listened-state indicators, favorite action, playlist removal, progress reset, original external link, channel navigation, notifications, and accessibility names defined for those behaviors. Changing presentation SHALL NOT create a second playback or episode-state implementation. Favorite and playlist-state icons rendered directly in the row SHALL be read-only, and favorite state SHALL continue to use the existing star icon rather than a heart.

#### Scenario: Compact presentation invokes existing actions
- **WHEN** an action is activated from the playlist-specific mobile presentation
- **THEN** it produces the same player, persisted-state, and notification outcome as that action in the existing episode-card presentation

#### Scenario: State changes remain synchronized
- **WHEN** playback, playlist membership, favorite state, progress, or listened state changes for an episode
- **THEN** the playlist-specific mobile presentation immediately reflects the same shared state as every other copy of that episode

### Requirement: Episode-card presentation is explicitly selected by context

The playlist-specific presentation SHALL be used only when the containing view explicitly selects it. Existing default and compact episode cards SHALL NOT infer playlist styling from viewport size, queue source, playlist membership, or other episode data.

#### Scenario: Playlist opts into its presentation
- **WHEN** the main playlist renders episode cards
- **THEN** those cards use the playlist-specific presentation below `sm` and the existing presentation at `sm` and wider

#### Scenario: Other views do not inherit playlist styling
- **WHEN** the channel episode list or history view renders the same episode
- **THEN** its card remains in its existing default or compact presentation regardless of viewport size or whether that episode belongs to the playlist

### Requirement: Played mark, resume hint, and progress strip on episode cards

The episode card SHALL render its playback state compactly, per the `playback-progress` capability: the top-right corner tinted green when the episode is listened (no label or icon), a resume hint for partially played episodes, and a read-only progress strip spanning the card's bottom edge that reflects the saved position (the live playhead for the currently playing episode) and ignores pointer interaction.

#### Scenario: Played mark on completed episodes
- **WHEN** an episode has `listen` true
- **THEN** the card's top-right corner is tinted green instead of showing a label or check

#### Scenario: Resume hint on partial episodes
- **WHEN** an episode has a stored position above 30 seconds and `listen` is false
- **THEN** the card shows a hint with the stored position (for example "Continue at MM:SS") and an affordance to start over

#### Scenario: Progress strip reflects the saved point
- **WHEN** an episode has a saved position
- **THEN** the card shows a bottom progress strip sized proportionally to `position_seconds` over the episode duration

#### Scenario: Progress strip is read-only
- **WHEN** the user clicks or drags on the card's progress strip
- **THEN** playback is unaffected (the strip has no interaction handlers)

#### Scenario: No indicator for untouched episodes
- **WHEN** an episode has never been played or its position is at most 30 seconds
- **THEN** the card shows neither the played mark nor a resume hint, and no progress strip

### Requirement: Add/remove toggle for the single playlist

Each episode card SHALL expose a toggle reflecting whether the episode is in the single playlist: adding when absent, removing when present, with a notification on each action.

#### Scenario: Adding an episode
- **WHEN** the episode is not in the playlist and the user activates the card's playlist toggle
- **THEN** the episode is appended to the end of the playlist and a success notification is shown

#### Scenario: Removing an episode
- **WHEN** the episode is in the playlist and the user activates the card's playlist toggle
- **THEN** the episode is removed from the playlist and reindexed, and a notification is shown

#### Scenario: Adding an already-present episode is prevented
- **WHEN** the episode is already in the playlist
- **THEN** the action fails with a message and the playlist is unchanged

### Requirement: Mark-as-not-listened control re-adds to the playlist

For an episode marked listened, the card SHALL expose a control that clears the listened state (resetting the stored position to zero) and appends the episode to the end of the playlist.

#### Scenario: Unmarking a listened episode
- **WHEN** the user activates the "mark as not listened" control on a listened episode
- **THEN** the episode's listened state clears, its position resets to zero, the card swaps back from the played mark, and the episode appears again at the end of the playlist

#### Scenario: Unmarking an episode already pending
- **WHEN** the intended episode is already in the playlist
- **THEN** the listened state still clears and the episode remains in the playlist exactly once


### Requirement: Favorite toggle on episode cards

Each episode card SHALL expose a favorite toggle rendered as a star icon reflecting the episode's stored favorite flag: a hollow (outline) star SHALL indicate a non-favorite and a filled star SHALL indicate a favorite. Activating the toggle on a non-favorite marks the episode as favorite; activating it on a favorite unmarks it, with a notification on each action. The toggle's state SHALL come from the episode's `favorite` field and SHALL update immediately when changed, without a list refetch.

#### Scenario: Non-favorite episodes show a hollow star
- **WHEN** an episode's `favorite` is false and the card is rendered
- **THEN** the card shows the favorite toggle as a hollow (outline) star icon

#### Scenario: Favorite episodes show a filled star
- **WHEN** an episode's `favorite` is true and the card is rendered
- **THEN** the card shows the favorite toggle as a filled star icon

#### Scenario: Marking an episode as favorite from the card
- **WHEN** the episode's `favorite` is false and the user activates the card's favorite toggle
- **THEN** the backend stores favorite true, the card's star becomes filled, and a success notification is shown

#### Scenario: Unmarking a favorite from the card
- **WHEN** the episode's `favorite` is true and the user activates the card's favorite toggle
- **THEN** the backend stores favorite false, the card's star becomes hollow, and a notification is shown

#### Scenario: Toggle stays in sync with the shared state
- **WHEN** the same episode is rendered in more than one card (e.g. channel view and episodes view)
- **THEN** toggling in one place updates the star state everywhere the episode is rendered
