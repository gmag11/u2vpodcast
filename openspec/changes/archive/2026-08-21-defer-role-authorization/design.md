## Context

`Role` (`src/models/role.rs`), the session role claim (`USER_ROLE_KEY`) and the `user_role` payload field exist, but no handler consults them. A bug review flagged this; verification shows the deployment is deliberately single-user with no user CRUD endpoints. The product plan places role management later.

## Goals / Non-Goals

**Goals:**
- Record the current single-admin invariant explicitly.
- Define the contract the future role system MUST satisfy, so it cannot land without authorization checks.
- Zero runtime change in this iteration.

**Non-Goals:**
- No new endpoints, no middleware, no role checks in this change.
- No removal of the existing `Role` enum or role claim (they stay for the future stage).

## Decisions

- **Defer enforcement but codify the invariant.** The current state is not a bug: every account is an admin and there is one account. Adding guards against "non-admin escalation" would be dead code with no real attack path and no way to test non-admin flows.
- **Mitigate the realistic risk instead:** the only escalation path today would arrive *with* the role-management stage itself, so the requirement that ships that stage also mandates its guards (admin check + last-admin protection).
- **Document in code:** a comment on `Role`/`from_session` naming the invariant and pointing to this spec, so the next implementer does not assume role checks exist.

## Risks / Trade-offs

- [If a future release adds user management without reading this contract, guards could be forgotten] → Mitigated by the spec requirement (shipping that stage requires its authorization contract) and the code comment.
- [Review tooling will keep flagging "missing role checks"] → Accepted; the deferred spec makes the intent explicit for reviewers.

## Migration Plan

None (documentation change). When the role-management stage starts, it SHALL open a new change referencing `role-authorization`.

## Open Questions

None.