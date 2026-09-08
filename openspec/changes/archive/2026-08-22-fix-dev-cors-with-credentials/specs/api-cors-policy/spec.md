## ADDED Requirements

### Requirement: Development CORS uses an explicit local allowlist, never a wildcard with credentials

In development mode (non-production), credentialed cross-origin requests SHALL be restricted to an explicit allowlist: the configured `url`, `http://localhost:{port}`, `http://127.0.0.1:{port}`, and the YouTube images host. The development branch SHALL NOT combine `allow_any_origin()` (or any wildcard) with `supports_credentials()`. Every allowlisted origin SHALL be validated at startup with fail-fast behavior, exactly as in production.

#### Scenario: Dev SPA calls the API from localhost with credentials
- **WHEN** the SPA dev server sends a credentialed request to the API from `http://localhost:{port}` or `http://127.0.0.1:{port}`
- **THEN** the response echoes that origin and sets `Access-Control-Allow-Credentials: true`

#### Scenario: Foreign origin is not reflected in dev mode
- **WHEN** a dev-mode request carries an `Origin` that is not on the explicit allowlist
- **THEN** the response does NOT echo the origin and does NOT grant credentials, so the browser blocks the cross-origin read

#### Scenario: Invalid configured origin fails fast
- **WHEN** the configured `url` in dev mode is not a valid origin (e.g. `localhost` with no scheme)
- **THEN** startup aborts with an explanatory error rather than deploying a permissive policy