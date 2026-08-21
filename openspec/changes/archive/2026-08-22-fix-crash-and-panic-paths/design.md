## Context

The three `From` impls at `src/models/channel.rs:491`, `src/models/episode.rs:277` and `src/models/response.rs:82` each call `x.into()` inside their own `from`, which resolves to the impl itself — infinite recursion. They are currently unused, which is why they compile and run fine. Any future `let v: Value = channel.into()` or `HttpResponse::from(response)` recurses until the stack overflows, which for tokio worker threads aborts the whole worker rather than just failing a request.

`Key::from` (actix-session / aes-gcm) requires at least 64 bytes and panics otherwise with a `Key is too small` error. The `secret_key` comes from `config.yml` with no validation, so a deployment with a short key cannot start (panic inside `wrap(...)` at `src/main.rs:243` and `:256`).

`Error::serialize` (`src/models/error.rs:38`) calls `self.status_code.unwrap()` after `Error::default()` stores `None`. The `ResponseError` path (used for handlers) never serializes directly, so this is latent, but any code path that serializes an `Error` — e.g. a future wrapper or a log formatter — panics.

`post_login` (`src/handlers/login.rs:47-54`) inserts session claims with `.expect(...)`. `Session::insert` fails only if serialization of the value fails (rare), but a panic here returns no response at all.

## Goals / Non-Goals

**Goals:**
- No reachable code path may panic due to recursion, short keys, or missing error metadata.
- Bugs fail fast with clear messages when caused by configuration.

**Non-Goals:**
- No change to authentication/session semantics.
- No change to the error response shape in the HTTP layer (`ResponseError` behavior is preserved).
- No introduction of a general panic-catch harness.

## Decisions

- **Recursion:** delete the two unused model `From` impls (`channel.rs`, `episode.rs`); the `From<CustomResponse<T>> for HttpResponse` impl in `response.rs` is unused and also deleted. If any of them is ever needed again, implement it with `serde_json::to_value(...)` / explicit construction, never `into()`. A grep test asserts no `impl From<_> for Value` / `for HttpResponse` bodies contain a bare `into()`.
- **Secret key:** validate at startup alongside the existing CORS origin validation (`validate_origin`). Add a `validate_secret_key` helper: trim the value, require length >= 64 bytes, and return a descriptive error naming the configured `secret_key` issue plus a hint to generate one with `openssl rand -base64 48`. Run in both modes (the key is also used in development).
- **Error::Serialize:** replace `self.status_code.unwrap().as_u16()` with `self.status_code().as_u16()` (the existing accessor already falls back to 500). This also aligns the field with the effective response status.
- **Login session inserts:** replace each `.expect(...)` with a match that logs and returns `CResponse::ko(StatusCode::INTERNAL_SERVER_ERROR, session)`. Because `session` is moved on success, restructure to insert into a cloned session or build the error response from the owned session.

## Risks / Trade-offs

- [Changing `Error` Debug/serialize field semantics] → None: the status-code fallback matches what `status_code()` already returns for HTTP.
- [Startup validation rejects previously "working" configs] → Intended: a short key is broken today (panic), a clear error is strictly better.
- [Deleting unused impls may break hidden call sites] → Compile-checked; the crate compiles only if nothing references them. Grep test guards against reintroduction.

## Migration Plan

1. Fix `Error::serialize` fallback.
2. Remove/replace the three recursive `From` impls; add the no-recursion grep test.
3. Add `validate_secret_key` in `src/main.rs` and invoke before building `SessionMiddleware`.
4. Rework login session inserts to return 500 on failure.
5. Verify: `cargo build` + full test suite; manual startup with a short key shows the new error and exits cleanly.

## Open Questions

None.