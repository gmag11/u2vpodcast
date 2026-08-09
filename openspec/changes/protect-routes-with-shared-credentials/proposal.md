## Why

The application exposes three surfaces today with inconsistent protection. The JSON API (`/api/1.0/*`) is effectively unguarded server-side: a commented-out `.wrap(Authentication)` in `handlers/mod.rs` and a client-side redirect in `+page.ts` are the only "protection", so any HTTP client can read channel data without a session. The RSS feed (`/channels/{id}/feed.xml`) is fully public, leaking the whole episode catalogue. Static media (`/media/**`) is fully public too, leaking every downloaded MP3. All three surfaces should require the credentials that already exist admin-only in `config.yml`. Doing this now closes a real information leak before someone other than the operator subscribes to the instance.

## What Changes

- **Guard the JSON API with the existing session mechanism.** Add a `require_session` middleware that rejects requests without a valid `user_id` in the cookie session and returns a `CustomResponse` body with `user: null` so the existing SvelteKit redirect flow keeps working verbatim. Apply it to an inner scope that excludes `login`, `logout`, `status`, and `session`, which must remain anonymous.
- **Guard the RSS feed and media files with HTTP Basic Auth.** Add a `basic_auth` middleware (backed by `actix-web-httpauth::extractors::basic::BasicAuth`) that resolves credentials against the `users` table via `User::get_by_name` + `verify_password` (argon2), returning `401` with `WWW-Authenticate: Basic realm="u2vpodcast"` on failure. Apply it to `/channels/{id}/feed.xml` and to `/media/**`.
- **Add a `with_authentication` config flag** (`boolean`, read from `config.yml`) that toggles the feed and media Basic Auth guard: when `true` both surfaces require valid Basic Auth; when `false` they are served without credential check (the pre-change public behavior). The JSON API session guard is unaffected by this flag and stays enforced. The flag lets operators deploy this change without breaking already-subscribed podcast clients and flip the protection on later.
- **Share a single source of truth.** Both guards verify against the same admin user already seeded by `User::default` at startup from `config.yml`; no second store of credentials is introduced.
- **Add `actix-web-httpauth` to `Cargo.toml`.** New dependency for Basic Auth parsing.
- **No frontend changes** for the happy path: the SvelteKit `+page.ts` already redirects when `response.user == null`, which is exactly the body `require_session` will return on rejection.
- **BREAKING**: any podcast client currently subscribed without authentication will stop receiving the feed until credentials are configured; any curl/automation hitting `/api/1.0/*` without a session cookie will now get `401`.

## Capabilities

### New Capabilities
- `route-protection`: server-side access control for JSON API (session-guarded), RSS feed, and static media (HTTP Basic Auth), all sharing the single admin account seeded from `config.yml`.

### Modified Capabilities
<!-- None: no existing specs in openspec/specs/ today. -->

## Impact

- **Code**: `src/handlers/mod.rs` (scope restructuring + media scope), `src/main.rs` (media files moved out), new `src/utils/middleware.rs` (two `wrap_fn` middlewares), `src/utils/mod.rs` (exports), `src/models/mod.rs` (re-exports `from_session`/`SessionUser`), `src/handlers/feed.rs` (feed resource wrapped with `basic_auth`), `src/models/config.rs` (add `with_authentication` field), `config.yml` (add `with_authentication: true`).
- **Dependencies**: `actix-web-httpauth = "0.8"` added to `Cargo.toml`. Docker rebuild (`Dockerfile`, `Dockerfile.arm64` via `build-arm64.sh`) invalidated; layer cache for the `cargo fetch` step will be cold.
- **APIs**: `/api/1.0/channels|episodes|config/*` now reject anonymous requests with `401` (JSON `CustomResponse`). `/channels/{id}/feed.xml` and `/media/**` now reject anonymous requests with `401` + `WWW-Authenticate: Basic`.
- **Podcast clients**: any client previously subscribing anonymously must be re-configured with the admin username/password. Most clients (Apple Podcasts, Pocket Casts, AntennaPod, gPodder, AntennaPod, Overcast) support HTTP Basic Auth for feeds and propagate credentials to enclosure downloads; clients that do not will break and require the token-in-URL fallback (out of scope for this change, flagged as a risk in `design.md`).
- **Security**: Basic Auth transmits base64 credentials on every request; the existing Caddy TLS setup (`docker-compose.caddy.yml`) is required for production. `verify_password` (argon2 default) runs on every feed/media request; on parallel enclosure downloads this can be CPU-busy for a few hundred ms per file.
- **No DB migrations**: reuses the existing `users` table and the existing admin seeding in `main.rs:142-147`.
- **Frontend**: untouched. `+page.ts` flow (`if response.user == null -> redirect to /app/login`) keeps working because rejection bodies keep the `CustomResponse` shape.