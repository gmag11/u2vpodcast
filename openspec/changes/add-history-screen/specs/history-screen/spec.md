## ADDED Requirements

### Requirement: History screen lists all episodes newest first

The SPA SHALL expose a protected `history` route reachable from the navigation. The history screen SHALL load every episode across all channels via the all-episodes endpoint and render them in a single list ordered from newest to oldest by `published_at`.

#### Scenario: Navigating to the history screen
- **WHEN** an authenticated user navigates to the history route
- **THEN** the screen renders a list containing episodes from multiple channels, ordered newest first

#### Scenario: Unauthenticated user is redirected to login
- **WHEN** a user without a valid session requests the history route
- **THEN** the router redirects to `/login`, matching the other protected routes

#### Scenario: No episodes exist yet
- **WHEN** the history screen loads and there are no episodes
- **THEN** the screen renders an empty-state message instead of an empty list

### Requirement: Each history card identifies its channel

Every episode card on the history screen SHALL display the name of the channel the episode belongs to, in addition to the episode's own title, description, date, and playback controls.

#### Scenario: Channel name shown on the card
- **WHEN** the history screen renders an episode belonging to channel `Linux y Tapas`
- **THEN** the card shows `Linux y Tapas` as the channel name alongside the episode's own title

### Requirement: History cards use a compact, wider layout

Episode cards on the history screen SHALL use a compact layout that reduces vertical height while making use of the wider available horizontal space, while SHALL retaining the same playback controls (play/pause, seek, stop, speed, volume) as the channel episodes list.

#### Scenario: Card is compact vertically
- **WHEN** the history screen renders an episode card
- **THEN** the card is shorter in vertical extent than the channel episodes list card for the same content

#### Scenario: Card still plays audio
- **WHEN** the user activates play on a history card
- **THEN** the shared player plays the episode's audio, with the same seek/stop/speed/volume controls available

### Requirement: History list is filterable by a live search input

The history screen SHALL display a text search input at the top of the list. As the user types, the visible episode cards SHALL update immediately on every keystroke (no reload, no server request), keeping an episode visible only when every whitespace-separated word of the query appears, case-insensitively, in at least one of its `title`, `description`, or `yt_id` fields. Clearing the input SHALL restore the full list.

#### Scenario: Filtering history as the user types
- **WHEN** the history list contains episodes titled `Episodio 10` and `Episodio 42`, and the user types `42`
- **THEN** only the `Episodio 42` card is visible, immediately, without any reload

#### Scenario: Multi-word query matches all words
- **WHEN** the user types `linux kernel` in the history search input
- **THEN** only episodes whose matched fields contain both `linux` and `kernel` remain visible

#### Scenario: No matches shows a message
- **WHEN** the user types a query matching no episode in the history search input
- **THEN** the list is replaced by a message stating that no results match the search
