# Tasks

## 1. Dependencies and configuration

- [x] 1.1 Add the `hmac` crate to `Cargo.toml` and confirm `cargo build` resolves it alongside the existing `sha2`
- [x] 1.2 Add `share_ttl_days` to `Config` in `src/models/config.rs` with a default of `30` and a non-positive fallback to `30`, and add a config parsing test asserting the default and an explicit override
- [x] 1.3 Document `share_ttl_days` in `README.md` and the sample `config.yml`, then confirm the checked-in sample config still parses via the existing `checked_in_sample_config_parses` test

## 2. Share token and minting

- [x] 2.1 Implement a share-token module in `src/utils/` that derives the signing key from `secret_key` with domain separation, signs `"v1|{yt_id}|{exp}"`, produces the `{exp}-{yt_id}-{sig}` token, and verifies it in constant time; add unit tests for a valid token, a tampered signature, a payload/`yt_id` mismatch, an expired token, and a malformed token
- [x] 2.2 Add the authenticated `POST /api/1.0/episodes/{yt_id}/share/` handler that resolves the episode, mints a token with the configured lifetime, and returns `{ url: "{config.url}/app/share/{token}", expires_at }`; add handler tests for a stored episode, an unknown `yt_id` (`404`), and a missing session (`401`)

## 3. Public tokenized routes

- [x] 3.1 Extract the range/`ETag`/`Last-Modified`/`304` streaming core out of `serve_media` in `src/handlers/media.rs` so a resolved on-disk file can be served by both handlers, and confirm the existing `media.rs` tests still pass unchanged
- [x] 3.2 Add the public `GET /s/{token}/episode.json` handler returning only `{ title, channel_title, description, image, duration, published_at, expires_at, audio_url }`; add tests for a valid token (no user/progress/favorite fields present) and for malformed, forged, and expired tokens (`404`)
- [x] 3.3 Add the public `GET`/`HEAD /s/{token}/audio.mp3` handler that resolves the episode's current representation (processed when SponsorBlock is enabled and a valid derivative exists, otherwise the original), streams it through the extracted core, sets `Cache-Control: private`, and responds `404` for invalid/expired tokens or missing media; add tests for `200`, a satisfiable `Range` (`206`), `HEAD`, and both `404` cases
- [x] 3.4 Register the `/s` scope outside every auth-wrapped scope in `src/handlers/mod.rs` and add tests proving a valid token is served with no cookie and no `Authorization` header while `with_authentication` is `true`, and that an invalid token returns `404` rather than `401`

## 4. Public share page

- [x] 4.1 Update `frontend/src/router/index.ts` so `meta.guestOnly` marks the login route (authenticated users are redirected away) and `meta.public` marks no-auth routes that authenticated users may still open, and add the `/share/:token` public route pointing at the new view; add router tests for an anonymous visitor, an authenticated visitor not being redirected, and login still bouncing an authenticated user
- [x] 4.2 Add `ShareView.vue` that reads the token, fetches `/s/{token}/episode.json`, renders cover, title, channel title, description, and an inline `<audio controls>` player, and shows an unavailable state on `404`; add the public fetch helper to `frontend/src/lib/api/client.ts` and component tests for the loaded and unavailable states
- [x] 4.3 Add the English and Spanish i18n keys used by the share page and confirm the locale parity test passes

## 5. Authenticated share action

- [x] 5.1 Add `createShareLink(ytId)` to `frontend/src/lib/api/client.ts` calling `POST /api/1.0/episodes/{ytId}/share/` and add an episode-card share action that copies the returned URL and notifies on success or error; add component tests for the success (copied) and failure (error notification, nothing copied) paths

## 6. Integration verification

- [x] 6.1 Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test`, and run the frontend lint, test, and build commands
- [x] 6.2 End-to-end check: log in, mint a share link from an episode card, open `/app/share/{token}` in a private window with no session and confirm metadata and audio play with seek, then confirm an expired/tampered token shows the unavailable state, and that existing feeds and `/media/**` still require credentials
