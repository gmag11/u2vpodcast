## MODIFIED Requirements

### Requirement: Web playback skips configured rejected intervals on the original timeline
The shared player SHALL continue loading the original `/media/{slug}/{yt_id}.mp3` source. When SponsorBlock is enabled, it SHALL use the normalized categorized SponsorBlock segments included in the episode payload. Whenever the playhead enters a segment marked as rejected, the player SHALL seek to the end of the complete overlapping rejected interval. Segments not marked as rejected SHALL remain playable. Playback position, duration, seeking, completion, and persisted progress SHALL remain expressed on the original MP3 timeline. Episode-card and persistent-player progress tracks SHALL display all SponsorBlock segments whenever enabled data is available, including before playback and while paused; `sponsor` segments SHALL use the existing sponsor color and every other category SHALL use a second color distinct from both sponsor markers and playback progress. When SponsorBlock is disabled, the player SHALL perform no SponsorBlock skips and SHALL render no SponsorBlock markers.

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