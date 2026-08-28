## ADDED Requirements

### Requirement: Mobile playlist presents dense episode rows

Below the `sm` breakpoint, the main playlist SHALL present each episode as a dense horizontal row based on the Stitch screen "Rediseño playlist movil". Each row SHALL expose its independent reduced drag affordance, compact episode image, bold single-line title, smaller normal-weight channel name, total duration, publication date, playback progress or listened state when applicable, read-only state icons, and an overflow trigger without horizontal page overflow. The episode description and standalone playback controls SHALL NOT occupy space in this presentation.

#### Scenario: Pending episode is easy to scan on a phone
- **WHEN** the main playlist is viewed below the `sm` breakpoint
- **THEN** each episode appears as a compact horizontal row containing its image-based play/pause control, scrolling title, static channel, duration, date, read-only state icons, and overflow trigger
- **AND** its description is not displayed

#### Scenario: Long title can be read in full
- **WHEN** an episode title exceeds its available single-line width
- **THEN** the bold title scrolls horizontally so its full text can be read without moving the channel name or causing horizontal page overflow

#### Scenario: Channel name remains static
- **WHEN** an episode has a channel name on a narrow viewport
- **THEN** the channel name appears below the title in smaller normal-weight text without scrolling or overlapping adjacent controls

#### Scenario: Playback state remains visible
- **WHEN** a playlist episode has saved or live playback progress, is listened, or is currently playing
- **THEN** the compact row communicates the applicable state using its progress treatment and read-only icons

#### Scenario: Favorite uses the existing star
- **WHEN** favorite state is represented in the mobile playlist row or menu
- **THEN** it uses the application's existing star icon and does not use the heart shown in the Stitch reference

#### Scenario: State icons are informational only
- **WHEN** the user taps, clicks, or focuses a favorite or playlist-state icon shown directly in the row
- **THEN** no state-changing action is performed and the corresponding action remains available only from the overflow menu

#### Scenario: Reduced drag affordance remains operable
- **WHEN** the playlist can be reordered on mobile
- **THEN** the six-dot affordance is visually smaller than before while retaining pointer, touch, keyboard, and assistive-technology operation

### Requirement: Mobile playlist row retains all episode actions

Below the `sm` breakpoint, the compact playlist row SHALL use its episode image as the only direct play/pause control and SHALL NOT expose a stop control. Its accessible overflow menu SHALL contain exactly these actions in this order: Favourite, Remove from playlist, Original link, Reset progress, and Channel view. Opening or activating an episode action SHALL NOT initiate dragging.

#### Scenario: User opens secondary actions
- **WHEN** the user activates a mobile playlist row's overflow trigger
- **THEN** Favourite, Remove from playlist, Original link, Reset progress, and Channel view are presented in that order with accessible names
- **AND** no additional action is present

#### Scenario: User controls playback through the image
- **WHEN** the user activates the episode image in a mobile playlist row
- **THEN** that episode starts or toggles play/pause through the existing shared player and playlist queue

#### Scenario: Stop is absent from the mobile row
- **WHEN** the mobile playlist row or its overflow menu is rendered
- **THEN** no stop control or stop action is available

#### Scenario: User changes episode state from the overflow menu
- **WHEN** the user marks the episode as favorite, removes it from the playlist, opens its original link, resets its playback progress, or opens its channel view from the overflow menu
- **THEN** the same outcome and notification behavior as the existing episode-card action occurs

#### Scenario: Action interaction does not reorder
- **WHEN** the user taps, clicks, or uses the keyboard on the row, its playback control, or its overflow menu
- **THEN** no drag operation starts unless the independent drag handle was activated

### Requirement: Playlist redesign is isolated by breakpoint

The dense playlist-row redesign SHALL apply only to the main playlist below the `sm` breakpoint. At `sm` and wider, the playlist SHALL retain its existing card presentation. Episode cards outside the playlist SHALL retain their existing presentation at every viewport width.

#### Scenario: Desktop playlist is unchanged
- **WHEN** the playlist is viewed at `sm` or wider
- **THEN** its episode cards, controls, metadata, spacing, and drag-handle arrangement match the pre-redesign desktop presentation

#### Scenario: Other episode lists are unchanged on mobile
- **WHEN** the channel episode list or history view is viewed below the `sm` breakpoint
- **THEN** its episode cards retain their existing presentation and actions
