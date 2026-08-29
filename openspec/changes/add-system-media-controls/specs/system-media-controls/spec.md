## Purpose

Defines predictable operating-system controls and now-playing information for the shared web audio player on supported mobile and desktop browsers, with safe fallback elsewhere.

## ADDED Requirements

### Requirement: System media controls degrade safely

When the user starts authenticated playback, the frontend SHALL expose system media controls through browser capabilities when available. The player SHALL retain all existing in-app playback behavior when the browser lacks system media controls or rejects an individual action registration. An unsupported system action SHALL NOT prevent other supported actions from being registered or used.

#### Scenario: Browser supports system media controls
- **WHEN** authenticated playback starts in a browser that supports system media controls
- **THEN** the current episode is exposed as the active media session and supported actions can control the shared player

#### Scenario: Browser lacks system media controls
- **WHEN** playback starts in a browser without system media control support
- **THEN** audio and every in-app player control continue to operate without an error

#### Scenario: Browser rejects one action
- **WHEN** the browser supports system media controls but rejects registration of an individual action
- **THEN** the frontend leaves that action unavailable and continues registering and serving the remaining supported actions

### Requirement: System play and pause use shared playback state

System play and pause actions SHALL control the same shared audio source and state used by episode cards and the persistent player. System play SHALL resume the current episode according to the existing stopped/resume behavior, and system pause SHALL pause it and persist progress. Native play and pause events initiated by the browser or operating system SHALL keep the frontend playing and stopped states consistent with the actual audio element.

#### Scenario: System pauses active playback
- **WHEN** the user activates pause from an operating-system control while an episode is playing
- **THEN** the shared audio pauses, the frontend shows the paused state, and the current progress is persisted

#### Scenario: System resumes a paused episode
- **WHEN** the user activates play from an operating-system control while the current episode is paused
- **THEN** the same episode resumes and the frontend shows the playing state

#### Scenario: Native playback event updates frontend state
- **WHEN** the browser directly starts or pauses the shared audio element in response to a native media control
- **THEN** the frontend state converges to the audio element's actual playing or paused state without creating a second audio source

#### Scenario: Play action has no current episode
- **WHEN** a system play action arrives while no authenticated current episode is available
- **THEN** no playback starts and the frontend remains stable

### Requirement: System track navigation follows the player queue

System next-track and previous-track actions SHALL use the same navigation behavior as a short press of the persistent player's next and previous controls. Next-track SHALL play the first queued episode without marking the departing episode listened. Previous-track SHALL restart the current episode when it is beyond the existing three-second threshold and otherwise navigate to playback history. An unavailable navigation target SHALL leave playback unchanged.

#### Scenario: System next-track consumes the queue
- **WHEN** a system next-track action arrives while an episode remains in the up-next queue
- **THEN** the first queued episode becomes current and starts playing without the departing episode being marked listened

#### Scenario: System next-track has no queued episode
- **WHEN** a system next-track action arrives with an empty up-next queue
- **THEN** the current episode and playback state remain unchanged

#### Scenario: System previous-track restarts after threshold
- **WHEN** a system previous-track action arrives more than three seconds into the current episode
- **THEN** the current episode restarts from zero

#### Scenario: System previous-track navigates within threshold
- **WHEN** a system previous-track action arrives at or before three seconds and playback history is available
- **THEN** the most recently played episode becomes current using the existing resume policy

#### Scenario: System previous-track has no history
- **WHEN** a system previous-track action arrives at or before three seconds with no playback history
- **THEN** the current episode and playback state remain unchanged

### Requirement: System seek actions preserve playback rules

System seek-backward, seek-forward, and absolute-seek actions SHALL move the shared playhead on the original media timeline, clamp it to valid episode bounds, and apply the same SponsorBlock rejected-interval behavior as in-app seeking. Relative system seeks SHALL use the offset supplied by the operating system and SHALL default to 15 seconds when no valid positive offset is supplied.

#### Scenario: System seeks forward with an offset
- **WHEN** the operating system requests a forward seek by 30 seconds
- **THEN** the shared playhead moves forward by 30 seconds, clamped to the episode duration and advanced past any rejected interval containing the target

#### Scenario: System seeks backward without an offset
- **WHEN** the operating system requests a backward seek without a valid positive offset
- **THEN** the shared playhead moves backward by 15 seconds, clamped to zero and advanced past any rejected interval containing the target

#### Scenario: System seeks to an absolute position
- **WHEN** the operating system requests an absolute seek to a valid time in the current episode
- **THEN** the shared playhead moves to that original-timeline position, subject to bounds and rejected-interval skipping

#### Scenario: System seek has no loaded duration
- **WHEN** a system seek action arrives before a usable episode duration is available
- **THEN** the player ignores any request that cannot be safely bounded and remains stable

### Requirement: Current episode metadata is exposed to the operating system

For an authenticated current episode, the media session SHALL expose its episode title and channel title, and SHALL include its artwork when a usable image URL is available. Metadata SHALL update when queue navigation or any other playback path changes the current episode. Missing artwork SHALL NOT prevent textual metadata or playback controls from working.

#### Scenario: Episode starts with complete metadata
- **WHEN** playback loads an episode with a title, channel title, and image
- **THEN** the operating system receives the episode title, channel title, and artwork as now-playing metadata

#### Scenario: Queue navigation changes metadata
- **WHEN** playback advances from one episode to another
- **THEN** the operating-system metadata changes to the new episode without retaining the previous episode's title or artwork

#### Scenario: Episode has no usable artwork
- **WHEN** the current episode has no usable image URL
- **THEN** the operating system still receives its title and channel title and playback continues normally

### Requirement: System playback status and position remain synchronized

When supported by the browser, the media session SHALL report whether the shared player is playing, paused, or inactive and SHALL publish a valid duration, position, and playback rate for the current episode. Synchronization SHALL occur after relevant playback, metadata, seek, time, speed, stop, and episode-change events. Invalid or incomplete media values SHALL be omitted rather than causing playback failure.

#### Scenario: Playback status changes
- **WHEN** the shared audio starts, pauses, stops, or loses its authenticated session
- **THEN** the operating-system playback status updates to playing, paused, or inactive as appropriate

#### Scenario: Position changes during playback
- **WHEN** a usable duration exists and playback time, seek position, or speed changes
- **THEN** the operating system receives a position state whose duration, position, and playback rate match the bounded shared-player values

#### Scenario: Position values are temporarily invalid
- **WHEN** duration, position, or playback rate cannot form a valid system position state
- **THEN** the frontend skips that position update without interrupting audio or other system actions

### Requirement: Authentication teardown removes the active media session

When authentication is lost, the frontend SHALL stop native audio, make the media session inactive, remove now-playing metadata, and prevent stale system actions from restarting protected playback. A later authenticated playback action SHALL establish a fresh media session.

#### Scenario: User logs out during playback
- **WHEN** authentication is lost while an episode is playing or paused
- **THEN** audio stops, now-playing information is removed, the system session becomes inactive, and subsequent stale system controls do not restart the episode

#### Scenario: Playback starts after a later login
- **WHEN** the user authenticates again and starts an episode
- **THEN** a fresh system media session is established for that episode
