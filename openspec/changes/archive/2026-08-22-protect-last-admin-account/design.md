## Context

`User::delete` (`src/models/user.rs:187`) runs `DELETE FROM users WHERE id = $1 RETURNING *` directly. The handler `DELETE /api/1.0/users/` (`src/handlers/users.rs:98`) takes `user_id` as a query param — it does not compare with the acting session's own id. So an admin can delete their own account, or another account that is the only remaining admin.

Current facts:
- `role-authorization` explicitly defers "prevent deleting the last active administrator" to the future role-management stage.
- In the meantime, the single-admin invariant means the guard logic is exactly: "an active admin is being deleted AND it is the last active admin → refuse".
- Seeding every startup (`User::delete_all` + `User::default`) is protected by `admin_credentials_present()`; deployments that keep only the DB row are the ones that would be stranded.

This change implements only the guard; it does not build any of the authorization machinery the deferred stage will add (role checks on endpoints, non-admin rejection). It is safe to ship now and is subsumed later.

## Goals / Non-Goals

**Goals:**
- Deleting the last active admin is impossible through the API.
- The guard applies no matter what the acting session is (self-delete or cross-delete).
- Clear error message and status for the blocked case.

**Non-Goals:**
- No role-management UI/API.
- No change to startup reseeding (`delete_all` at boot stays, since credentials are present by definition there).
- No change to normal-user deletion (still allowed when an admin remains).

## Decisions

- **Where the guard lives:** in `User::delete` (model). Rationale: every caller shares it; adding it in the handler alone would miss future call sites. Add an enum/typed error path: introduce `Error::new_with_status_code("Cannot delete the last active admin", StatusCode::CONFLICT)` (or `BAD_REQUEST`) so the API surfaces `409` with the message.
- **Detection query:** count active admins. `SELECT count(*) FROM users WHERE role = 'admin' AND active = 1`. Read the target user first (fetch_one already returns `RowNotFound`→404 for missing ids). Refuse when `target.active && target.role == Admin && active_admin_count <= 1`.
  - Note: `role` is stored lowercase via `#[sqlx(rename_all = "lowercase")]`, so compare against `'admin'`.
- **Alternative considered — session-based "prevent self-delete":** rejected; the actual invariant is about the account state (last admin), not about who asks. Cross-session deletion of the last admin is equally harmful.
- **Error contract:** the handler already passes `Err(e)` up through the actix `Responder`; the new `409` flows through `error_response()` unchanged. No new response shape.

## Risks / Trade-offs

- [The future role-management stage may reimplement this differently] → Accepted. This guard becomes a precondition of that stage, which is allowed to check the same invariant with richer role data.
- [Race between count and delete] → The app has a single sqlite pool with 2 connections and no concurrent admin management traffic; still, run detection inside a transaction (`BEGIN IMMEDIATE` semantics via `tx`) to close the race window and keep the DELETE in the same tx.
- [Delete-all at startup unaffected] → Confirmed: `delete_all` is `DELETE FROM users` with no rows returned; it does not pass through `User::delete` (model), so seeding is untouched.

## Migration Plan

1. Read target user; if missing, 404 (existing behavior).
2. In a transaction: count active admins; if the target is the last one, return the new `409` error (tx rolled back).
3. Delete within the same transaction; commit.
4. Add tests: last-admin delete refused (409), normal user delete allowed, admin-with-second-admin delete allowed, non-existent id still 404.
5. Verify the SPA shows the refusal message without changes.

## Open Questions

None.