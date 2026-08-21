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