## ADDED Requirements

### Requirement: All episodes endpoint returns every episode newest first

The backend SHALL expose a protected endpoint `GET /api/1.0/episodes/` that returns every episode from every channel in a single list, ordered by `published_at` descending (newest first). The endpoint SHALL require an authenticated session, consistent with the existing protected channel and episode endpoints.

#### Scenario: Authenticated request returns all episodes
- **WHEN** an authenticated user requests `GET /api/1.0/episodes/`
- **THEN** the response `data` is an array of every episode in the database, ordered from newest `published_at` to oldest

#### Scenario: Unauthenticated request is rejected
- **WHEN** a request without a valid session hits `GET /api/1.0/episodes/`
- **THEN** the endpoint is protected by the session middleware and does not return the episode list

### Requirement: Each episode carries its channel slug and title

Each episode returned by the all-episodes endpoint SHALL include a `channel_slug` field and a `channel_title` field identifying the owning channel, so the frontend can resolve the media URL and label the episode's channel without a second lookup.

#### Scenario: Episode is annotated with its channel
- **WHEN** an episode belongs to a channel with slug `confesiones` and title `Confesiones de Gasolinera`
- **THEN** the returned episode object has `channel_slug: "confesiones"` and `channel_title: "Confesiones de Gasolinera"`

#### Scenario: Channel title is absent
- **WHEN** an episode's owning channel cannot be resolved to a title
- **THEN** the episode object still serializes with an empty (not missing) `channel_title` field
