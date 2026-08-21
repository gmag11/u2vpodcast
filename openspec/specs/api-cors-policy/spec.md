## Purpose

Defines the production CORS policy for `/api/1.0/**`. In production, credentialed cross-origin access is restricted to an explicit origin allowlist (the configured deployment `url` plus the YouTube images host) — no wildcard origin may be combined with credentials. Misconfigured origins fail fast at startup.

## Requirements

### Requirement: Production CORS restricts credentialed cross-origin access to an explicit allowlist

In production mode, the API SHALL answer requests whose `Origin` header is NOT on the configured allowlist without echoing that origin and WITHOUT setting `Access-Control-Allow-Credentials: true`. The allowlist SHALL be built exclusively from explicit `allowed_origin(...)` entries; the builder SHALL NOT include `allow_any_origin()` in the production branch. Requests with no `Origin` header (curl, same-site clients) SHALL continue to work unaffected.

#### Scenario: Allowed origin receives echoed origin with credentials
- **WHEN** a preflighted or simple request to `/api/1.0/channels/` carries an `Origin` header that matches a configured allowed origin, plus the session cookie
- **THEN** the response includes `Access-Control-Allow-Origin: <that origin>` and `Access-Control-Allow-Credentials: true`

#### Scenario: Disallowed origin is not reflected
- **WHEN** a request to `/api/1.0/channels/` carries an `Origin` header that is not on the allowlist, plus the session cookie
- **THEN** the response does NOT echo the request origin in `Access-Control-Allow-Origin` and does NOT set `Access-Control-Allow-Credentials: true`, so the browser blocks the cross-origin read

#### Scenario: Wildcard origin is never configured in production
- **WHEN** a production build starts and its CORS middleware is inspected
- **THEN** no wildcard (any-origin) policy is active; only the explicit allowlist applies

### Requirement: Configured CORS origins are valid and fail fast on misconfiguration

The configured `url` allowed origin SHALL be a valid origin (`scheme://host` with optional `:port`, no trailing slash or path). At startup, production mode SHALL validate every allowed origin string; an invalid origin SHALL abort startup with a clear error message rather than silently deploying a broken or permissive policy.

#### Scenario: Valid origin in config
- **WHEN** `config.yml` sets `url: https://podcasts.example.com` and the app starts in production mode
- **THEN** the app starts and echoes only that origin for credentialed cross-origin requests

#### Scenario: Invalid origin in config
- **WHEN** `config.yml` sets `url: localhost` (no scheme) and the app starts in production mode
- **THEN** startup aborts with an explanatory error naming the offending value

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