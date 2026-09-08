## 1. Document the Decision

- [x] 1.1 Add a code comment at the `Role` enum and/or `from_session` stating the single-admin invariant and pointing to the `role-authorization` spec
- [x] 1.2 Audit the current route map and record (in the change notes/spec) that no user create/delete/role-change endpoint exists, so role checks are not implementable or needed yet

> **Corrección de auditoría:** los endpoints `/api/1.0/users/**` SÍ existen (configurados vía `users::api_users` en `handlers/mod.rs`). Lo que no existe es un flujo multi-cuenta/roles: el despliegue tiene una única cuenta administradora. El spec se corrigió en consecuencia (el invariante es single-admin, no ausencia de endpoints); la tarea 1.2 queda cumplida con esa nota.

## 2. Verify No Partial Guards

- [x] 2.1 Confirm no half-implemented role checks were introduced meanwhile; run cargo build/test
- [x] 2.2 Re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #2 is documented as deferred-by-design and no new bugs appeared

## 3. Future Stage Contract

- [x] 3.1 When the role-management stage opens, require: admin checks on user-management endpoints, rejection of non-admin sessions, and a last-active-admin deletion guard (contract documented in the `role-authorization` spec)