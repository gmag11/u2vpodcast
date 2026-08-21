## 1. DB-Backed Session Validation

- [ ] 1.1 Add/extend a user lookup by id (`User::get_by_id`) used by the session path
- [ ] 1.2 Make `RequireSession` resolve `user_id` against the `users` table and return `401` on missing or inactive rows; refresh role/name/active claims from the row
- [ ] 1.3 Apply the same lookup to the session branch of `SessionOrBasicAuth` (feed/media)

## 2. Verification & Regression

- [ ] 2.1 Test: delete a user mid-session → next request 401; deactivate → 401; reseed → old cookie 401; active user → normal behavior
- [ ] 2.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #9 resolved and no new bugs introduced (especially default/seed/bootstrap flows)