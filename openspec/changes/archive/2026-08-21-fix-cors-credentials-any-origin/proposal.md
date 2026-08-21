## Why

`src/main.rs` production CORS config chains `.allow_any_origin()` after the explicit `.allowed_origin(...)` allowlist. In actix-cors 0.7 `allow_any_origin()` resets the origin list to "all", silently killing the whitelist, and because `supports_credentials()` is enabled the middleware echoes any request `Origin` with `Access-Control-Allow-Credentials: true`. The session cookie is `SameSite=None; Secure`, so any website can issue credentialed cross-origin requests to the API and read the responses.

## What Changes

- Remove `.allow_any_origin()` from the production CORS branch.
- Keep only explicit, corrected `allowed_origin(...)` entries (full origins `scheme://host[:port]`, no trailing slash; include the YouTube images host for the cover-image fetch).
- Validate the configured `url` as a proper CORS origin at startup and fail fast with a clear message when it is not.
- Leave the development branch behavior unchanged (any origin is acceptable in dev).

## Capabilities

### New Capabilities

- `api-cors-policy`: Defines the production CORS policy for `/api/1.0/**`, restricting credentialed cross-origin access to an explicit allowlist.

### Modified Capabilities

(none)

## Impact

- `src/main.rs` (CORS builder, config handling).
- `config.yml` (`url` field must be a valid origin in production deployments).
- Manual browser/curl verification of preflight and credentialed requests.
- Regression guard applies: this fix must not introduce new bugs — a post-implementation re-analysis against `docs/bug-review-2026-08-21.md` is required.