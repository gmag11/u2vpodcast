## 1. Implement the last-admin guard in the model

- [x] 1.1 Refactor `User::delete` (`src/models/user.rs`) to a transaction that: reads the target user (missing → 404), counts active admins (`role = 'admin' AND active = 1`), and refuses with `StatusCode::CONFLICT` ("cannot delete the last active admin") when the target is an active admin and the count is ≤ 1
- [x] 1.2 Keep the delete and commit inside the same transaction to close the count→delete race
- [x] 1.3 Leave `User::delete_all` (startup seeding) untouched

## 2. Tests

- [x] 2.1 Deleting the only active admin → `409` and the user still exists
- [x] 2.2 Deleting a regular (non-admin) user → succeeds as before
- [x] 2.3 Deleting an admin when a second active admin exists → succeeds
- [x] 2.4 Deleting a non-existent user → `404` (existing behavior preserved)
- [x] 2.5 Full test suite passes

## 3. Verification

- [x] 3.1 Manual API check with the seed credentials: create a second admin, verify both can be removed one-by-one but the last one cannot
- [x] 3.2 Confirm the SPA surfaces the refusal message without frontend changes