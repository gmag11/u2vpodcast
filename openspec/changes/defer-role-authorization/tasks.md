## 1. Document the Decision

- [ ] 1.1 Add a code comment at the `Role` enum and/or `from_session` stating the single-admin invariant and pointing to the `role-authorization` spec
- [ ] 1.2 Audit the current route map and record (in the change notes/spec) that no user create/delete/role-change endpoint exists, so role checks are not implementable or needed yet

## 2. Verify No Partial Guards

- [ ] 2.1 Confirm no half-implemented role checks were introduced meanwhile; run cargo build/test
- [ ] 2.2 Re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #2 is documented as deferred-by-design and no new bugs appeared

## 3. Future Stage Contract

- [ ] 3.1 When the role-management stage opens, require: admin checks on user-management endpoints, rejection of non-admin sessions, and a last-active-admin deletion guard