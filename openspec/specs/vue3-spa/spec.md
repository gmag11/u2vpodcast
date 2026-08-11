## Purpose

Defines the Vue 3 single-page application that replaces the current SvelteKit frontend. The SPA mirrors the existing pages and routes, moves auth guards and search filtering client-side, restores the session on reload, extracts channel cover images, and talks to the same backend API endpoints with the same JSON contract.

## Requirements

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

The SPA SHALL talk to the same backend endpoints with the same JSON contract: `POST /api/1.0/login/`, `GET /api/1.0/logout/`, `GET /api/1.0/session/`, `GET|POST /api/1.0/channels/`, `PUT|DELETE /api/1.0/channels/{slug}/`, `GET /api/1.0/channels/{id}/episodes/`, `GET /api/1.0/config/`. Base URL SHALL be configurable (dev `http://localhost:6996`, production same-origin), matching the current `base_endpoint` behavior.

#### Scenario: Channel CRUD uses the same endpoints
- **WHEN** the user creates, edits, or deletes a channel
- **THEN** the SPA issues the corresponding `POST`/`PUT`/`DELETE` request to the channel endpoints and reflects the result in the list

### Requirement: Session is restored on page reload

Before the Vue app mounts, the SPA SHALL call `GET /api/1.0/session/` to restore the authenticated user from the session cookie. When the session is valid, the app SHALL set the auth state from the returned `user` before mounting, so that a page reload does not bounce the user back to the login screen. When the session is invalid or absent, the auth state SHALL remain unauthenticated and the router guard redirects to `/login`.

#### Scenario: Reload keeps the user logged in
- **WHEN** an authenticated user reloads the page
- **THEN** the app calls `/api/1.0/session/`, restores the user in the auth store before mount, and stays on the protected route without showing the login screen

#### Scenario: Reload without a session redirects to login
- **WHEN** a user without a valid session cookie reloads the page
- **THEN** `/api/1.0/session/` returns `user: null` and the router guard redirects to `/login`

### Requirement: Channel cover image is extracted and rendered

When a channel is created, the backend SHALL extract the channel title, description, and cover image from the channel URL using an HTTP client that sends a browser-like `User-Agent` and `Accept-Language` header, so YouTube serves the page metadata instead of a block page. A channel whose URL cannot be resolved SHALL be created with an empty image. The channel card SHALL render the cover image when present and SHALL show a neutral placeholder icon when the image is empty.

#### Scenario: Valid YouTube channel resolves a cover image
- **WHEN** the user creates a channel with a valid YouTube channel URL
- **THEN** the backend fetches the page with a browser-like User-Agent, extracts the `og:image`, and the channel card renders it

#### Scenario: Unresolvable URL stores an empty image
- **WHEN** the user creates a channel whose URL returns an error (e.g., 404)
- **THEN** the channel is created with `image: ""` and its card shows the placeholder icon

### Requirement: Notification and loading feedback

The SPA SHALL surface loading state during requests and show user-facing notifications (success/error) for login, create, update, and delete actions, preserving the feedback the current app provides.

#### Scenario: Login shows loading then a result notification
- **WHEN** the user submits the login form
- **THEN** a loading indicator appears during the request and a success or error notification is shown afterward

### Requirement: Development server proxies the backend for same-origin sessions

In development, the Vite dev server SHALL proxy API and media requests (`/api`, `/media`, `/channels`) to the backend at `http://localhost:6996` instead of the SPA calling the backend cross-origin. The SPA SHALL use a relative `baseEndpoint` (`''`) in both dev and production. This keeps every browser request same-origin so the session cookie (set with `SameSite=Lax` when `production: false`) is sent on reloads, preventing the login screen from reappearing after a page refresh.

#### Scenario: Login persists across a dev reload
- **WHEN** an authenticated user reloads the app served by the Vite dev server
- **THEN** the request for `/api/1.0/session/` is proxied same-origin, the cookie is sent, the user is restored, and the app does not redirect to `/login`

### Requirement: Channel card navigates to its episodes

Clicking anywhere on a channel card SHALL navigate to that channel's episodes list (`/:channelId`). The card's action buttons (feed link, edit, delete) SHALL stop event propagation so they do not trigger the navigation.

#### Scenario: Clicking a channel card opens its episodes
- **WHEN** the user clicks on a channel card (cover, title, or description area)
- **THEN** the router navigates to `/app/{channelId}` and the episodes list for that channel renders

#### Scenario: Action buttons do not navigate
- **WHEN** the user clicks the feed link, edit, or delete button on a channel card
- **THEN** the button action runs (opens feed, opens dialog, prompts delete) and no navigation to the episodes page occurs

### Requirement: Episode list is populated by the local worker

The backend episode worker SHALL resolve the audios directory, yt-dlp binary, and cookies file from the runtime environment (Docker paths `/app/audios`, `/app/.local/bin/yt-dlp` when present; local `audios`, `yt-dlp` from PATH otherwise) so that episodes are downloaded and stored in the database when running locally, not only in Docker. A channel page SHALL display its episodes once the worker has downloaded them.

#### Scenario: Episodes appear after the worker downloads them
- **WHEN** the worker resolves the local `audios` dir and `yt-dlp` from PATH, downloads an episode's audio, and stores the episode row
- **THEN** `GET /api/1.0/channels/{id}/episodes/` returns the episode and the Chapters screen lists it with its thumbnail

#### Scenario: Media files are served from the resolved audios directory
- **WHEN** a client requests `/media/{slug}/{yt_id}.mp3` for a downloaded episode
- **THEN** the file is served from the environment-resolved audios directory (local `audios/`, Docker `/app/audios`)
