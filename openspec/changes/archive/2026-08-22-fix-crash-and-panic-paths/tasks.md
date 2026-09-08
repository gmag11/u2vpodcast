## 1. Eliminate the recursive `From` impls

- [x] 1.1 Delete `impl From<Channel> for Value` in `src/models/channel.rs` (or replace with `serde_json::to_value`) and remove any now-unused imports
- [x] 1.2 Delete `impl From<Episode> for Value` in `src/models/episode.rs` likewise
- [x] 1.3 Delete `impl From<CustomResponse<T>> for HttpResponse` in `src/models/response.rs` (the commented-out `Into` impl below it can stay or go with it)
- [x] 1.4 Add a test or grep-based guard asserting no `From` impl body contains a bare `into()` call

## 2. Validate the session secret key at startup

- [x] 2.1 Add a `validate_secret_key(key: &str) -> Result<(), String>` helper in `src/main.rs` (trim, require >= 64 bytes, actionable error message with generation hint)
- [x] 2.2 Call it before building the `SessionMiddleware`, in both production and development branches, mapping the error to a startup failure
- [x] 2.3 Add unit tests for short key, empty key, and valid key cases

## 3. Harden `Error` serialization

- [x] 3.1 In `src/models/error.rs`, replace `self.status_code.unwrap()` with `self.status_code()` in the `Serialize` impl
- [x] 3.2 Add a test that serializes an `Error::default(...)` and an `Error::new_with_status_code(...)` and asserts no panic and a numeric `status_code` field

## 4. Handle session insert failures gracefully

- [x] 4.1 In `src/handlers/login.rs`, replace the four `.expect(...)` session inserts with error handling that logs the failure and returns `CResponse::ko(StatusCode::INTERNAL_SERVER_ERROR, session)`
- [x] 4.2 Verify the login success path still writes all four session claims and the response shape is unchanged

## 5. Verification

- [x] 5.1 `cargo build` clean; full test suite passes
- [x] 5.2 Manual check: start with a short `secret_key` and confirm a clear startup error instead of a panic
- [x] 5.3 Manual check: login still works end-to-end with a valid key
- [x] 5.4 Re-run the bug review against `docs/bug-review-2026-08-21.md`; confirm no new panics introduced