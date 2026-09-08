## Context

`(page - 1) * per_page` with `page <= 0` yields negative offsets. `users.rs` handler accepts raw query param; models `Channel`/`Episode` expose the same helper pattern. Only the users endpoint is currently reachable, but all three use identical arithmetic.

## Goals / Non-Goals

**Goals:**
- No paginated endpoint can produce a negative offset.
- Identical behavior for valid pages.

**Non-Goals:**
- No UI changes, no per-page cap redesign, no new validation framework.

## Decisions

- **Clamp at the model boundary with `page.max(1)`** before computing the offset in each `read_with_pagination` (users/channels/episodes). Single-point fix that covers every handler using them; the handler no longer needs its own guard.
- Keep `per_page` handling as-is (already bounded by config).
- Alternative considered (rejecting `page<1` with 400) — clamping is friendlier and matches the "malformed input degrades gracefully" spirit of the fix list.

## Risks / Trade-offs

- [Consumers could confuse clamped pages with data emptiness] → Returned first page is correct; no ambiguity beyond ordinary pagination.
- [Three call sites to keep in sync] → Same pattern in three model files is accepted; a shared helper is optional follow-up, not required.

## Migration Plan

Code change only.

## Open Questions

None.