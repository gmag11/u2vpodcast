## Purpose

Adds live, client-side word-based search filtering to the channel list on the homepage and the episodes list on each channel's page, so users can narrow down long lists to the items they care about by typing in a search box.

## Requirements

### Requirement: Channel list is filterable by a live search input

The homepage SHALL display a text search input above the channel list. As the user types, the list of visible channel cards SHALL update immediately on every keystroke (no page reload, no server request). A channel SHALL remain visible when every whitespace-separated word of the query appears, case-insensitively, in at least one of its `title`, `description`, `url`, or `slug` fields. Clearing the input SHALL restore the full channel list.

#### Scenario: Filtering channels as the user types
- **WHEN** the channel list contains channels titled `Confesiones de Gasolinera` and `Linux y Tapas`, and the user types `gasolinera` in the search input
- **THEN** only the `Confesiones de Gasolinera` card is visible, immediately, without any reload

#### Scenario: Multi-word query matches all words
- **WHEN** the user types `linux tapas` in the channel search input
- **THEN** only channels whose matched fields contain both `linux` and `tapas` (in any order) remain visible

#### Scenario: Case-insensitive match
- **WHEN** the user types `LINUX` in the channel search input
- **THEN** channels whose matched fields contain `linux` (regardless of case) remain visible

#### Scenario: Clearing the search restores the list
- **WHEN** the user clears the channel search input after filtering
- **THEN** the full, unfiltered channel list is shown again

### Requirement: Episode list is filterable by a live search input

Each channel's episodes page SHALL display a text search input above the episodes list. As the user types, the list of visible episode cards SHALL update immediately on every keystroke (no page reload, no server request). An episode SHALL remain visible when every whitespace-separated word of the query appears, case-insensitively, in at least one of its `title`, `description`, or `yt_id` fields. Clearing the input SHALL restore the full episodes list.

#### Scenario: Filtering episodes as the user types
- **WHEN** the episodes list contains episodes titled `Episodio 10` and `Episodio 42`, and the user types `42` in the search input
- **THEN** only the `Episodio 42` card is visible, immediately, without any reload

#### Scenario: Episode matched by video id
- **WHEN** the user types a value matching an episode's `yt_id` in the search input
- **THEN** the episode whose `yt_id` contains that value remains visible even if its title does not

#### Scenario: Multi-word query matches all words
- **WHEN** the user types `linux kernel` in the episode search input
- **THEN** only episodes whose matched fields contain both `linux` and `kernel` (in any order) remain visible

#### Scenario: Clearing the search restores the list
- **WHEN** the user clears the episode search input after filtering
- **THEN** the full, unfiltered episodes list is shown again

### Requirement: Empty search results show a no-matches message

When a search query is non-empty and no channel or episode matches it, the affected page SHALL display a message indicating that no results match the search, instead of rendering an empty list.

#### Scenario: No channel matches the query
- **WHEN** the user types a query that matches no channel in the channel search input
- **THEN** the channel list is replaced by a message stating that no results match the search

#### Scenario: No episode matches the query
- **WHEN** the user types a query that matches no episode in the episode search input
- **THEN** the episodes list is replaced by a message stating that no results match the search
