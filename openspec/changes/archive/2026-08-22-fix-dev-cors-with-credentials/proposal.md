## Why

The development CORS branch (`src/main.rs:277-285`) combines `allow_any_origin()` with `supports_credentials()`. Browsers reject a literal wildcard with credentialed requests, but actix-cors reflects the request's `Origin` instead — effectively echoing **any** origin while sending the session cookie. In development the app binds `0.0.0.0`, so a malicious page on another origin can use the logged-in user's cookie against the local app (CSRF/credential-leaking risk for local tools and any LAN exposure). Production already restricts to an explicit allowlist; development should not be categorically weaker.

## What Changes

- Replace the wildcard dev policy with an explicit allowlist covering local development origins: the configured `url`, plus `http://localhost:{port}`, `http://127.0.0.1:{port}`, and the YouTube images host (same as production).
- Keep `.supports_credentials()` for the allowlisted origins.
- Reuse the existing `validate_origin` check on any origin added beyond the constants.
- No wildcard origin is ever combined with credentials in either mode.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `api-cors-policy`: now also defines the development-mode origin policy (explicit local allowlist, no wildcard with credentials).

## Impact

- `src/main.rs` (dev CORS builder).
- Developers who previously pointed a custom origin at the dev server from a different host must add it to the allowlist (see design).
- No production behavior change; no API contract change.

## Non-Goals

- No change to production CORS policy.
- No drop of `supports_credentials` (sessions must keep working from the SPA dev server).
- No environment-variable mechanism for extra origins in this iteration (a fixed dev list; config-driven origins may follow).