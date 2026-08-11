## ADDED Requirements

### Requirement: Client-side redirect on missing session

The frontend SHALL handle anonymous access to protected routes in the Vue SPA instead of server-side SvelteKit loaders. When a request to a protected JSON endpoint (`/api/1.0/channels/*`, `/api/1.0/channels/{id}/episodes/`, `/api/1.0/config/*`) returns `401` with `user: null`, or when the user has no session at all, the SPA SHALL redirect to `/login` preserving the intended destination as a `next` parameter for post-login return. The backend `401` contract (`CustomResponse` with `status: false`, `status_code: 401`, `user: null`, `data: null`) is unchanged.

#### Scenario: Frontend redirect on session loss
- **WHEN** a user without a valid session requests `/` and the SPA fetches `/api/1.0/channels/`, receiving `401` with `user: null`
- **THEN** the SPA redirects to `/login?next=/` instead of rendering the channel list

#### Scenario: Post-login return to the original destination
- **WHEN** an anonymous user was redirected to `/login?next=/42` and then logs in successfully
- **THEN** the SPA navigates to `/42` after a successful login

#### Scenario: Session expires during use of a protected route
- **WHEN** a user is browsing `/` and a subsequent API call (e.g., delete channel) returns `401` with `user: null`
- **THEN** the SPA clears its auth state and redirects to `/login`
