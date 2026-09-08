## ADDED Requirements

### Requirement: Session claims are revalidated against the users table

Every request protected by session authorization (`RequireSession`, and the session branch of feed/media access) SHALL resolve the session `user_id` against the current `users` table on each request. If the row no longer exists or its `active` flag is `false`, the request SHALL be rejected with `401 Unauthorized`. The request SHALL NOT be authorized purely on stale cookie claims.

#### Scenario: Deleted user loses access immediately
- **WHEN** a user is deleted from the `users` table while holding a valid session cookie
- **THEN** the next protected request with that cookie returns `401 Unauthorized`

#### Scenario: Deactivated user loses access immediately
- **WHEN** a user's `active` flag is set to `false` while their session cookie is still valid
- **THEN** the next protected request with that cookie returns `401 Unauthorized`

#### Scenario: Active user keeps working
- **WHEN** a request carries a session cookie whose `user_id` resolves to an existing active row
- **THEN** the request is authorized and behaves exactly as before (no latency or shape change beyond the lookup)

#### Scenario: Reseeded admin invalidates old cookies
- **WHEN** the app starts in seeded mode (all user rows replaced by a fresh admin) and an old cookie from a previous run is used
- **THEN** the request is rejected with `401` instead of succeeding against a stale session

### Requirement: Session claims reflect the database row

When a session is validated, the effective `role`/`active`/`name` used by subsequent logic SHALL come from the database row (refreshed), so a role or activation change never lingers in the cookie beyond the first request after the change.

#### Scenario: Role claim refreshed from DB
- **WHEN** the database row's role differs from the value baked into the cookie at login
- **THEN** protected requests use the database role for any authorization decision