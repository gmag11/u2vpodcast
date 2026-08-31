# episode-chapters Specification

## Purpose

Captures per-episode chapter markers resolved by yt-dlp at download time, persists them independently of any per-request transformation, and exposes them through episode read APIs for downstream consumers such as the persistent player.

## Requirements

### Requirement: Chapters are captured from yt-dlp at download time
When yt-dlp's download output for a video includes a `chapters` array, the system SHALL parse each entry's `start_time`, `end_time`, and `title` and treat the ordered list as that episode's chapters. When yt-dlp's output has no `chapters` array or an empty one, the episode SHALL have no chapters.

#### Scenario: Video has chapters
- **WHEN** a downloaded video's yt-dlp output includes a non-empty `chapters` array
- **THEN** the created episode stores each chapter's start time, end time, and title in the same order

#### Scenario: Video has no chapters
- **WHEN** a downloaded video's yt-dlp output has no `chapters` key or an empty array
- **THEN** the created episode stores no chapters

### Requirement: Raw chapters are persisted per episode independently of SponsorBlock state
The system SHALL persist an episode's chapters, as captured at download time, as part of that episode's own stored data. This stored list SHALL represent the original, untrimmed video's timeline and SHALL NOT be recalculated when SponsorBlock configuration or segments change. Removing an episode SHALL remove its stored chapters.

#### Scenario: Chapters survive unrelated updates
- **WHEN** an episode with stored chapters is updated for any other reason (title, progress, listened mark, favorite, SponsorBlock snapshot)
- **THEN** its stored chapters remain unchanged

#### Scenario: Episode is deleted
- **WHEN** retention or channel deletion removes an episode that has stored chapters
- **THEN** its chapters are removed with no orphan relationship remaining

### Requirement: Episode APIs expose raw chapters
Episode payloads SHALL include the episode's stored chapters, each with `start`, `end`, and `title`, in ascending order. An episode with no stored chapters SHALL expose an empty chapter list. Exposed chapters SHALL always reflect the original, untrimmed video's timeline regardless of SponsorBlock configuration.

#### Scenario: Episode has chapters
- **WHEN** an authenticated client loads an episode that has stored chapters
- **THEN** the response includes every chapter with its original start time, end time, and title

#### Scenario: Episode has no chapters
- **WHEN** an authenticated client loads an episode with no chapters
- **THEN** the response includes an empty chapter list

#### Scenario: SponsorBlock is enabled
- **WHEN** SponsorBlock is enabled and processing removes part of an episode's audio
- **THEN** the episode API response still reports the episode's original, untranslated chapter times
