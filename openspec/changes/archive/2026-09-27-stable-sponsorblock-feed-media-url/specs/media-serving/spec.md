## Purpose

Serves episode audio through stable, client-cacheable URLs and resolves the stable episode media URL to the correct audio representation (SponsorBlock-processed or original) at request time.

## ADDED Requirements

### Requirement: Stable episode media URL selects the active representation
The system SHALL serve `GET` and `HEAD` requests for `/media/{slug}/{yt_id}.mp3`. When SponsorBlock is enabled and the episode identified by `{slug}` and `{yt_id}` has an active processed MP3 that exists on disk, the system SHALL serve that processed MP3. In every other case — SponsorBlock disabled, no active processed MP3, or the recorded processed file missing — the system SHALL serve the original `{yt_id}.mp3`. A request for an episode with no stored record SHALL serve the original `{yt_id}.mp3` when that file exists. When neither representation exists, the system SHALL respond `404 Not Found`.

#### Scenario: Active processed media is served at the stable URL
- **WHEN** SponsorBlock is enabled and episode `abc123` has an existing active processed file
- **THEN** `GET /media/{slug}/abc123.mp3` returns the processed MP3 bytes and a `Content-Length` matching that file

#### Scenario: Original is served when no processed media exists
- **WHEN** SponsorBlock is enabled but episode `abc123` has no active processed file
- **THEN** `GET /media/{slug}/abc123.mp3` returns the original `abc123.mp3` bytes

#### Scenario: Original is served when SponsorBlock is disabled
- **WHEN** SponsorBlock is disabled and episode `abc123` has an existing processed file
- **THEN** `GET /media/{slug}/abc123.mp3` returns the original `abc123.mp3` bytes

#### Scenario: Missing recorded derivative falls back to the original
- **WHEN** SponsorBlock is enabled and the recorded active processed file for episode `abc123` is missing on disk
- **THEN** `GET /media/{slug}/abc123.mp3` returns the original `abc123.mp3` bytes

#### Scenario: No representation exists
- **WHEN** neither the original nor a processed file exists for the requested episode
- **THEN** the system responds `404 Not Found`

### Requirement: Dedicated original-media route
The system SHALL serve `GET` and `HEAD` requests for `/media/{slug}/{yt_id}.original.mp3` from the original on-disk `{yt_id}.mp3` regardless of whether a processed MP3 is active. It SHALL NOT serve a processed MP3 from this route. When the original file is missing, the system SHALL respond `404 Not Found`.

#### Scenario: Original route ignores active processing
- **WHEN** SponsorBlock is enabled and episode `abc123` has an existing active processed file
- **THEN** `GET /media/{slug}/abc123.original.mp3` returns the original `abc123.mp3` bytes

#### Scenario: Original route supports range requests
- **WHEN** a client sends a satisfiable `Range` header for `/media/{slug}/abc123.original.mp3`
- **THEN** the system responds `206 Partial Content` with the corresponding byte range of the original file

#### Scenario: Original file is missing
- **WHEN** the original `abc123.mp3` does not exist
- **THEN** the system responds `404 Not Found`

### Requirement: Explicit processed-media routes remain available
The system SHALL continue to serve `GET` and `HEAD` requests for `/media/{slug}/{yt_id}.sponsorblock.{hash}.mp3` by streaming the named processed file when it exists, independent of which processing hash is currently active, so that feeds and downloads cached before this change keep working. When the named file does not exist, the system SHALL respond `404 Not Found`.

#### Scenario: Cached hash-versioned URL still downloads
- **WHEN** a client requests `/media/{slug}/abc123.sponsorblock.a81f302c.mp3` and that file exists
- **THEN** the system streams that file

#### Scenario: Unknown hash-versioned file
- **WHEN** a client requests a `{yt_id}.sponsorblock.{hash}.mp3` file that does not exist
- **THEN** the system responds `404 Not Found`

### Requirement: Response validators describe the served file
For every media response, the system SHALL derive `Content-Length`, `ETag`, `Last-Modified`, byte-range responses, and conditional-request handling from the physical file actually served. When the active representation for a stable URL changes, the validators for that URL SHALL change accordingly.

#### Scenario: Validators follow the active representation
- **WHEN** the active processed representation served for episode `abc123` changes to a different file
- **THEN** a subsequent `GET /media/{slug}/abc123.mp3` returns the new file's length and `ETag`
