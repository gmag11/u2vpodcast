## ADDED Requirements

### Requirement: Web playback skips stored sponsor intervals on the original timeline
The shared player SHALL continue loading the original `/media/{slug}/{yt_id}.mp3` source and SHALL use the normalized SponsorBlock segments included in the episode payload. Whenever the playhead enters a sponsor interval, the player SHALL seek to that interval's end. Playback position, duration, seeking, completion, and persisted progress SHALL remain expressed on the original MP3 timeline.

#### Scenario: Playback enters a sponsor interval
- **WHEN** normal playback reaches the start of a stored sponsor interval `[120, 150]`
- **THEN** the shared player seeks to original-media time 150 and continues playback

#### Scenario: User seeks into a sponsor interval
- **WHEN** the user moves the scrubber or uses a relative seek to a time inside `[120, 150]`
- **THEN** the player advances to original-media time 150

#### Scenario: Playback resumes inside a sponsor interval
- **WHEN** persisted progress points inside a stored sponsor interval
- **THEN** resume advances to the end of that interval instead of playing the sponsor segment

#### Scenario: Progress is persisted after a skip
- **WHEN** the player skips a sponsor interval ending at original-media time 150
- **THEN** subsequent progress writes and labels continue using the original timeline at or after 150

#### Scenario: Episode has no stored segments
- **WHEN** an episode payload has an empty or unavailable SponsorBlock snapshot
- **THEN** the shared player behaves exactly as ordinary original-MP3 playback

### Requirement: Refreshed segment snapshots take effect without replacing the source
When an authenticated refresh returns a different SponsorBlock hash for the current episode, the player SHALL replace its active segment set with the returned normalized segments without reloading or replacing the original MP3 source. An identical hash SHALL leave player state unchanged.

#### Scenario: Manual refresh changes current segments
- **WHEN** refresh returns a new hash and normalized intervals for the currently loaded episode
- **THEN** later playback and seeks use the new intervals while the current original media source and playhead are retained

#### Scenario: Manual refresh is unchanged
- **WHEN** refresh returns the same hash as the episode already holds
- **THEN** the player performs no source reload or playhead change