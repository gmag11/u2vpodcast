## ADDED Requirements

### Requirement: Vue 3 single-page application shell

The frontend SHALL be a Vue 3 SPA built with Vite, using Vue Router for routing and Pinia for state management. The app SHALL be served as static files (no server-side rendering), mirroring the static-file serving of the current SvelteKit build.

#### Scenario: App boots as a static SPA
- **WHEN** a browser loads the served index.html
- **THEN** the Vue app mounts client-side and routes are resolved by Vue Router without a page reload

### Requirement: Routes mirror the existing pages

The SPA SHALL expose the same three routes as the current frontend: `/` (Channels dashboard), `/login` (Login), and `/:channelId` (Chapters for one channel).

#### Scenario: Navigating to the channels dashboard
- **WHEN** an authenticated user visits `/`
- **THEN** the Channels screen renders the channel list

#### Scenario: Navigating to a channel's chapters
- **WHEN** an authenticated user visits `/42`
- **THEN** the Chapters screen renders the episodes of channel with id `42`

#### Scenario: Navigating to login
- **WHEN** any user visits `/login`
- **THEN** the Login screen renders

### Requirement: Client-side auth guards

Route protection SHALL move to the client. Protected routes (`/` and `/:channelId`) SHALL redirect to `/login` when the user is not authenticated, and the Login route SHALL redirect to `/` when the user is already authenticated. Authentication state SHALL be derived from the API responses (`user` field and `401` status), matching the backend `route-protection` contract.

#### Scenario: Anonymous user hits a protected route
- **WHEN** a user without a valid session requests `/`
- **THEN** the router redirects to `/login` (with `next` preserved for post-login return)

#### Scenario: Authenticated user hits the login route
- **WHEN** an authenticated user requests `/login`
- **THEN** the router redirects to `/`

#### Scenario: Session expires while browsing
- **WHEN** an API call on a protected route returns `401` / `user: null`
- **THEN** the app clears the auth state and redirects to `/login`

### Requirement: API client matches the existing endpoints

The SPA SHALL talk to the same backend endpoints with the same JSON contract: `POST /api/1.0/login/`, `GET /api/1.0/logout/`, `GET|POST /api/1.0/channels/`, `PUT|DELETE /api/1.0/channels/{slug}/`, `GET /api/1.0/channels/{id}/episodes/`, `GET /api/1.0/config/`. Base URL SHALL be configurable (dev `http://localhost:6996`, production same-origin), matching the current `base_endpoint` behavior.

#### Scenario: Channel CRUD uses the same endpoints
- **WHEN** the user creates, edits, or deletes a channel
- **THEN** the SPA issues the corresponding `POST`/`PUT`/`DELETE` request to the channel endpoints and reflects the result in the list

### Requirement: Notification and loading feedback

The SPA SHALL surface loading state during requests and show user-facing notifications (success/error) for login, create, update, and delete actions, preserving the feedback the current app provides.

#### Scenario: Login shows loading then a result notification
- **WHEN** the user submits the login form
- **THEN** a loading indicator appears during the request and a success or error notification is shown afterward
