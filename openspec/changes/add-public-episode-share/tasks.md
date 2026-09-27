# Tasks

## 1. Dependencies and configuration

- [x] 1.1 Add the `hmac` crate to `Cargo.toml` and confirm `cargo build` resolves it alongside the existing `sha2`
- [x] 1.2 Add an optional `share_secret` to `Config` (absent or empty falls back to `secret_key`) and remove `share_ttl_days`, and add config parsing tests asserting the default fallback and an explicit override
- [x] 1.3 Document `share_secret` in `README.md` and the sample `config.yml` and remove the `share_ttl_days` row, then confirm the checked-in sample config still parses via the existing `checked_in_sample_config_parses` test

## 2. Share token and minting

- [x] 2.1 Rework the share-token module in `src/utils/` to produce the permanent token `{yt_id}-{hex(HMAC(share_secret, "u2vpodcast/share/v1|{webpage_url}"))}`, with no expiry and constant-time verification against a loaded episode's `webpage_url`; add unit tests for determinism, a valid token, a tampered hash, an unresolvable `yt_id`, a different secret, and a malformed token
- [x] 2.2 Update the authenticated `POST /api/1.0/episodes/{yt_id}/share/` handler to return `{ url }` with no expiry; keep tests for a stored episode, an unknown `yt_id` (`404`), and a missing session (`401`)

## 3. Public tokenized routes

- [x] 3.1 Extract the range/`ETag`/`Last-Modified`/`304` streaming core out of `serve_media` in `src/handlers/media.rs` so a resolved on-disk file can be served by both handlers, and confirm the existing `media.rs` tests still pass unchanged
- [x] 3.2 Update the public `GET /s/{token}/episode.json` handler to return only `{ title, channel_title, description, image, duration, published_at, audio_url }` (no expiry); add tests for a valid token (no expiry/user/progress/favorite fields present) and for malformed and forged tokens (`404`)
- [x] 3.3 Update the public `GET`/`HEAD /s/{token}/audio.mp3` handler to resolve the episode's current representation and stream it through the extracted core with `Cache-Control: private`, responding `404` for invalid tokens or missing media; add tests for `200`, a satisfiable `Range` (`206`), `HEAD`, and both `404` cases (no expiry case)
- [x] 3.4 Register the `/s` scope outside every auth-wrapped scope in `src/handlers/mod.rs` and add tests proving a valid token is served with no cookie and no `Authorization` header while `with_authentication` is `true`, and that an invalid token returns `404` rather than `401`

## 4. Public share page

- [x] 4.1 Update `frontend/src/router/index.ts` so `meta.guestOnly` marks the login route (authenticated users are redirected away) and `meta.public` marks no-auth routes that authenticated users may still open, and add the `/share/:token` public route pointing at the new view; add router tests for an anonymous visitor, an authenticated visitor not being redirected, and login still bouncing an authenticated user
- [x] 4.2 Update `ShareView.vue` to render cover, title, channel title, description, and an inline `<audio controls>` player from the tokenized metadata, and show an unavailable state on `404`; add the public fetch helper to `frontend/src/lib/api/client.ts` and drop `expires_at` from the `SharedEpisode` type; add component tests for the loaded and unavailable states
- [x] 4.3 Add the English and Spanish i18n keys used by the share page and confirm the locale parity test passes

## 5. Authenticated share action

- [x] 5.1 Keep `createShareLink(ytId)` calling `POST /api/1.0/episodes/{ytId}/share/` and the episode-card share action that copies the returned URL, and drop `expires_at` from the `ShareLink` type; add component tests for the success (copied) and failure (error notification, nothing copied) paths

## 6. Integration verification

- [x] 6.1 Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test`, and run the frontend lint, test, and build commands
- [x] 6.2 End-to-end check: log in, mint a share link from an episode card, confirm the same URL is returned on a second mint, open `/app/share/{token}` in a private window with no session and confirm metadata and audio play with seek, then confirm a tampered token shows the unavailable state, and that existing feeds and `/media/**` still require credentials
