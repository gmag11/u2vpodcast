## MODIFIED Requirements

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

## ADDED Requirements

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