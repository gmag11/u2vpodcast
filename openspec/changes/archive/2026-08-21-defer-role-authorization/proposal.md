## Why

The `Role` enum and session role claim exist but no endpoint checks them. Reviewers flagged this as privilege escalation. However, the product has **no role-management system yet**: there is exactly one user, every user is an administrator, and users cannot be created or deleted. Single-user, all-admin operation is intentional and a full role system is planned for a later stage. Adding ad-hoc guards now would build a half-baked model on top of infrastructure that does not exist.

## What Changes

- No runtime behavior change in this change.
- Formalize the current invariant: the deployment is single-user and that user is an administrator.
- Document the decision that per-route role authorization SHALL be introduced together with the future role-management stage, with explicit anti-escalation requirements.
- Add a code comment at `Role`/`from_session` recording the invariant and the planned stage, so future work does not assume a guard that never existed.

## Capabilities

### New Capabilities

- `role-authorization`: Defines the current single-admin model and the deferred contract for role-based enforcement in the future role-management stage.

### Modified Capabilities

(none)

## Impact

- Documentation and spec only: this change does not touch runtime code paths.
- Guardrail: when the future role-management stage lands, it must not create new bugs; the same regression re-analysis workflow (using `docs/bug-review-2026-08-21.md` as reference) applies to that stage.