## 1. Serialization Fix

- [x] 1.1 Add `#[serde(skip_serializing)]` to `hashed_password` in `src/models/user.rs` (keep Deserialize)
- [x] 1.2 Audit user handlers and any other `User`-returning path; confirm no response carries the hash (login, create, delete, read, list, session)

## 2. Verification & Regression

- [x] 2.1 Run cargo build/test; exercise login, user list, create and delete responses and assert no `hashed_password` key appears while the server still authenticates correctly
- [x] 2.2 Re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #3 is resolved and no new bugs or changed response shapes broke the SPA