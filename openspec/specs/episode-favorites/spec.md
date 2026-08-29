# episode-favorites

## Purpose

Defines the favorite mark on episodes: how episodes are marked and unmarked through the API, how the flag travels in episode payloads, and the favorites-only filter in the episodes view and the history screen. Favorited episodes are exempt from the per-channel retention pruning (owned by `channel-retention-limit`).

## Requirements

### Requirement: Episodes can be marked and unmarked as favorite

The app SHALL expose a protected endpoint `PUT /api/1.0/episodes/{yt_id}/favorite/` accepting a JSON body with a boolean `favorite` field. The endpoint SHALL set the stored favorite flag on the episode identified by `yt_id` to exactly that value and SHALL be idempotent: repeating the same value leaves the flag unchanged. An unknown `yt_id` SHALL yield a 404 response, mirroring the progress endpoints.

#### Scenario: Marking an episode as favorite
- **WHEN** an authenticated user sends `PUT /api/1.0/episodes/{yt_id}/favorite/` with `{"favorite": true}` for an existing episode
- **THEN** the episode's stored favorite flag becomes true and the request succeeds

#### Scenario: Unmarking a favorite
- **WHEN** an authenticated user sends the same endpoint with `{"favorite": false}` for a favorited episode
- **THEN** the episode's stored favorite flag becomes false and the request succeeds

#### Scenario: Repeating the same value is idempotent
- **WHEN** the user sends `{"favorite": true}` for an episode already favorited
- **THEN** the flag stays true and the request still succeeds

#### Scenario: Unknown episode is rejected
- **WHEN** the user sends the endpoint for a `yt_id` that does not exist
- **THEN** the response is 404 and no row is changed

#### Scenario: Unauthenticated request is rejected
- **WHEN** a request without a valid session hits the favorite endpoint
- **THEN** the endpoint is protected by the session middleware and does not change any flag

### Requirement: Episode payloads carry the favorite flag

Every episode returned by the episode and all-episodes endpoints SHALL include a boolean `favorite` field reflecting the stored flag, so the frontend renders the toggle state without a separate lookup.

#### Scenario: Favorite flag in episode objects
- **WHEN** the API returns an episode that is stored as favorite
- **THEN** the episode object has `favorite: true`

#### Scenario: Non-favorite defaults to false
- **WHEN** the API returns an episode never marked as favorite
- **THEN** the episode object has `favorite: false`

### Requirement: Episodes and history views filter to favorites only

The episodes view and the history screen SHALL each provide a control that switches the list between all episodes and only favorited episodes. With the filter active, episodes whose favorite flag is false SHALL be hidden, combined with any active search query (an episode stays visible only when it is favorited AND matches the search words). When the filter is active and nothing remains, the view SHALL show an empty state indicating that there are no favorites yet (or none matching the search).

#### Scenario: Filter shows only favorites in the episodes view
- **WHEN** the user activates the favorites-only filter in the episodes view
- **THEN** only episodes with favorite true are shown

#### Scenario: Filter shows only favorites in the history screen
- **WHEN** the user activates the favorites-only filter in the history screen
- **THEN** only episodes with favorite true are shown, still ordered newest first

#### Scenario: Favorites filter combines with the search query
- **WHEN** an active favorites-only filter is combined with a text search query
- **THEN** only favorited episodes matching every search word remain visible

#### Scenario: Empty favorites state
- **WHEN** the filter is active and no episode is favorited, or none of the favorited episodes matches the search
- **THEN** the view shows an empty state explaining that no favorites exist

#### Scenario: Filter off shows everything
- **WHEN** the user deactivates the favorites-only filter
- **THEN** all episodes are shown again regardless of their favorite flag
