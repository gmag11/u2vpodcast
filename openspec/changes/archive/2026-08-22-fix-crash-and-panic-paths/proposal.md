## Why

Four independent code paths can crash the server or a worker with a panic or stack overflow. Each is latent today, but all four are reachable through the right misconfiguration or a future call site:

- The `From` conversion impls in `src/models/channel.rs`, `src/models/episode.rs` and `src/models/response.rs` recurse into themselves with no base case (`channel.into()` → `From::from(channel)` → ...). First use = stack overflow. Dead code now, live crash later.
- `SessionMiddleware` builds `Key::from(config.secret_key.as_bytes())`, which panics immediately when the configured key is shorter than 64 bytes. A short key in `config.yml` kills the server at startup.
- `Error`'s manual `Serialize` impl unwraps `self.status_code`, but `Error::default()` stores `None` there. Serializing such an error panics instead of producing a response.
- Session `.insert(...).expect(...)` calls in the login handler turn a transient serialization failure into a panic on the request handler thread.

## What Changes

- Remove the recursive `From` impls, replacing them with correct implementations (`serde_json::to_value`) or deleting them when unused.
- Validate the configured `secret_key` length at startup in both production and development mode, failing with a clear, actionable error instead of panicking.
- Make `Error` serialization use the safe `status_code()` accessor so a missing explicit status falls back to 500 instead of panicking.
- Replace `.expect(...)` on session inserts with graceful error handling that fails the request with a 500 response.

## Capabilities

### New Capabilities

- `crash-safety`: Guarantees that the code paths enumerated above never crash the process, worker, or request handler.

### Modified Capabilities

(none)

## Impact

- `src/models/channel.rs`, `src/models/episode.rs`, `src/models/response.rs` (recursive `From` impls).
- `src/main.rs` (secret-key validation at startup).
- `src/models/error.rs` (`Serialize` impl of `Error`).
- `src/handlers/login.rs` (session insert error handling).
- No API contract change; no schema change.

## Non-Goals

- No change to the session storage mechanism, cookie parameters, or key derivation format beyond length validation.
- No refactor of the `Error` type beyond the panic fix.