## MODIFIED Requirements

### Requirement: Single shared audio source owned by a global store

The app SHALL own exactly one `<audio>` element managed by a global audio player Pinia store. The store SHALL hold the currently loaded episode (its media URL, title, thumbnail, channel slug, and yt_id) and the live playback state (playing, current time, duration, volume, muted, playback rate, loading). All player UI in the app SHALL drive and read this single store; there SHALL NOT be multiple concurrent `<audio>` elements playing different sources.

#### Scenario: Starting playback loads the shared element
- **WHEN** the user presses play on any episode
- **THEN** the shared store sets that episode as the current source, loads its `/media/{slug}/{yt_id}.original.mp3` URL into the single `<audio>` element, and playback starts

#### Scenario: Playing a second episode swaps the source
- **WHEN** the user presses play on a different episode while another is playing
- **THEN** the shared element stops the previous source and loads the new episode's media URL

### Requirement: Web playback skips configured rejected intervals on the original timeline
The shared player SHALL continue loading the original `/media/{slug}/{yt_id}.original.mp3` source. When SponsorBlock is enabled, it SHALL use the normalized categorized SponsorBlock segments included in the episode payload. Whenever the playhead enters a segment marked as rejected, the player SHALL seek to the end of the complete overlapping rejected interval. Segments not marked as rejected SHALL remain playable. Playback position, duration, seeking, completion, and persisted progress SHALL remain expressed on the original MP3 timeline. Episode-card and persistent-player progress tracks SHALL display all SponsorBlock segments whenever enabled data is available, including before playback and while paused; this applies to both the interactive wide-composition scrubber and the read-only compact-composition track, which SHALL use the same segment colors and positions. `sponsor` segments SHALL use the existing sponsor color and every other category SHALL use a second color distinct from both sponsor markers and playback progress. When SponsorBlock is disabled, the player SHALL perform no SponsorBlock skips and SHALL render no SponsorBlock markers.

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
