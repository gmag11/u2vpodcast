## Context

`clean_channel` (`src/utils/worker.rs:78-101`) removes files+rows for every episode with index `>= max`, which makes `max` a destructive control. The backend binds it from JSON without validation; dialog uses `Number(max.value) || 5`, which passes negatives.

## Goals / Non-Goals

**Goals:**
- `max` always `>= 1` at the API boundary and in pruning.
- Existing invalid DB values neutralized (no wipe, no permanent failure).

**Non-Goals:**
- No new config or schema change.
- No change to the pruning algorithm itself for valid values.

## Decisions

- **Validate at the model boundary** (`Channel::new`/`Channel::update`): `max < 1` → `Error::new_with_status_code(..., BAD_REQUEST)`, so create/update return 4xx. Single choke point, covers both handlers.
- **Guard `clean_channel`:** if `usize::try_from(channel.max)` fails, log and skip pruning but keep the channel's sync successful (set status ok) — the wipe and the permanent-failure states both disappear. This also neutralizes already-stored bad values.
- **Clamp in the dialog** (`Math.max(1, Number(max.value))`) for UX, since backend now rejects.

## Risks / Trade-offs

- [Existing DB rows with `max <= 0` were previously causing failures; now they silently stop pruning] → Desired: the failure mode turns into "keep everything until the operator sets a valid max", which is the safe direction.
- [API consumers relying on permissive max] → They were relying on a data-destroying behavior; rejecting loudly is correct.

## Migration Plan

Code change only. No data migration; stale invalid values are neutralized by the clean guard.

## Open Questions

None.