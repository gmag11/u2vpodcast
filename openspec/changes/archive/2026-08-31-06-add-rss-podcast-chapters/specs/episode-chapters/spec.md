## MODIFIED Requirements

### Requirement: Raw chapters are persisted per episode independently of SponsorBlock state
The system SHALL persist an episode's chapters, as captured at download time, as part of that episode's own stored data. This stored list SHALL represent the original, untrimmed video's timeline and SHALL NOT be recalculated when SponsorBlock configuration or segments change. The yt-dlp download that produces the original MP3 SHALL also embed those raw chapters in that MP3. Removing an episode SHALL remove its stored chapters.

#### Scenario: Original MP3 is downloaded with chapters
- **WHEN** yt-dlp reports chapters while downloading an episode's original MP3
- **THEN** the system persists those chapters in the episode record and requests that yt-dlp embed them in the original MP3

#### Scenario: Chapters survive unrelated updates
- **WHEN** an episode with stored chapters is updated for any other reason (title, progress, listened mark, favorite, SponsorBlock snapshot)
- **THEN** its stored chapters remain unchanged

#### Scenario: Episode is deleted
- **WHEN** retention or channel deletion removes an episode that has stored chapters
- **THEN** its chapters are removed with no orphan relationship remaining
