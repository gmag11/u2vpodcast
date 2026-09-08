## Context

Production CORS (`config.production == true`) builds from `allowed_origin(&url)` + `YT_IMAGE_ORIGIN`. Development instead calls `Cors::default().allow_any_origin().supports_credentials()`. In actix-cors, `allow_any_origin()` grants any Origin; combined with `supports_credentials()`, responses carry `Access-Control-Allow-Origin: <echoed origin>` + `Access-Control-Allow-Credentials: true` for every origin. Because the dev server listens on `0.0.0.0`, any website the developer visits can (if it learns the local port) issue credentialed requests against the API and read channel/audio metadata the session cookie unlocks. The SPA in development runs on its own dev server (e.g. `localhost:5173`), so a narrow local allowlist does not break the intended workflow.

## Goals / Non-Goals

**Goals:**
- Credentialed CORS in development is restricted to local dev origins instead of "any origin".
- No wildcard-with-credentials combination in any mode.

**Non-Goals:**
- No change to production policy or `validate_origin` semantics.
- No UI change; the SPA dev flow keeps working.
- No dynamic per-request origin allowlists (keep the fixed list; revisit if a real need appears).

## Decisions

- **Shared builder:** extract the CORS construction into one function parameterized by the origin set. Both branches start from `Cors::default()`, add the same `allowed_methods`/`allowed_headers`/`expose_headers`/`max_age`, and call `supports_credentials()`.
- **Dev origin set:** `url` (config), `http://localhost:{port}`, `http://127.0.0.1:{port}`, `https://yt3.googleusercontent.com`. Validate every non-constant origin through `validate_origin`, fail fast at startup exactly like production does — so a misconfigured dev `url` cannot silently fall back to a broad policy.
- **Port handling:** build the localhost origins from `config.port`; web browsers treat `http://localhost:PORT` and `http://127.0.0.1:PORT` as distinct origins, so both are needed. Use `to_string()` formatting exactly as the SPA sends it (`Origin` headers carry no trailing slash).
- **Rejected alternative — `allow_any_origin()` when credentials are off:** the session cookie flow needs credentials, and toggling credential support based on config adds branches; the simplest correct posture is identical explicit allowlisting in both modes.
- **Test coverage:** extend the existing `cors_tests` module with the dev origin strings (validity checks), and assert via the running app in dev mode that a non-allowlisted origin is not echoed.

## Risks / Trade-offs

- [A developer using a LAN IP or custom dev host is blocked] → Documented; fix is adding the origin to the dev allowlist in code (single line) or promoting the allowlist to config later.
- [Behavioral difference between dev/prod remains] → Yes, but only in *which* allowed hosts exist, not in the policy shape (allowlist + credentials in both).

## Migration Plan

1. Extract shared CORS builder; apply dev origin set.
2. Run dev stack: SPA login + channel load must keep working from `localhost`.
3. Negative check: request with a foreign Origin (e.g. `http://evil.example`) in dev mode → no `Access-Control-Allow-Origin` echo.
4. Confirm production mode unchanged (existing `api-cors-policy` scenarios re-run).

## Open Questions

None.