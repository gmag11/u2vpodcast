## ADDED Requirements

### Requirement: Downloaded episodes are appended to the playlist

When the background download worker successfully stores a new episode (a chapter finished downloading and was persisted), the system SHALL append that episode to the end of the playlist using the same add semantics as the playlist API: duplicates are rejected (an episode already in the playlist stays at its current position exactly once) and the episode's stored position is the new last position. Episodes that are discarded during the download (e.g. published before the retention floor) SHALL NOT be appended.

#### Scenario: Successful download appends at the end
- **WHEN** a download completes and the episode is persisted while the playlist already holds two episodes
- **THEN** the new episode is appended as the third item at the end of the playlist

#### Scenario: Already-playlisted episode is not duplicated
- **WHEN** a download completes for an episode that is already in the playlist
- **THEN** the episode remains in the playlist exactly once at its existing position

#### Scenario: Discarded episode is not appended
- **WHEN** a downloaded episode is discarded because it is published before the retention floor
- **THEN** the episode is not added to the playlist

### Requirement: Auto-append never blocks or breaks the download

The playlist append SHALL be best-effort from the download flow: a failure to append (e.g. database error) SHALL be logged but SHALL NOT fail the download worker run or corrupt the playlist. A successful append SHALL persist before the worker moves on to the next item in the sync.

#### Scenario: Playlist append failure is tolerated
- **WHEN** appending a freshly downloaded episode to the playlist fails with a database error
- **THEN** the error is logged and the download run continues without aborting

#### Scenario: Append persists before the next download
- **WHEN** an episode has been appended to the playlist during a sync run
- **THEN** the playlist read endpoint reflects the new episode before the worker proceeds to the next download