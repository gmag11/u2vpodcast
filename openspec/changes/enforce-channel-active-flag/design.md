## Context

`Channel::read_all` (`src/models/channel.rs`) returns every channel and `do_the_work` (`src/utils/worker.rs`) iterates them all. `active` is stored and returned to the UI but never consulted.

## Goals / Non-Goals

**Goals:**
- Scheduled worker only processes active channels.
- Zero change to manual refresh behavior.

**Non-Goals:**
- No change to the UI toggle.
- No change to sync status recording semantics.

## Decisions

- **Filter in `Channel::read_all` (add `WHERE active = 1`).** Single source of truth for "channels the scheduler should process", and avoids touching the worker loop. The API read path and other callers of `read_all` must be checked: if any non-worker consumer needs inactive channels, use a separate query instead. Current consumers: worker only, so the filter is safe.
- **Do not daemonize anyway:** if a fully disabled-but-kept channel should stop being listenable too, that is out of scope; the toggle concerns syncing.

## Risks / Trade-offs

- [A future non-worker consumer of `read_all` might expect all channels] → Mitigated by checking consumers during implementation and by naming the filtered method explicitly if needed.
- [Inactive channels keep serving old episodes/feeds] → Accepted: disabling stops updates, not access; final policy can be tuned later.

## Migration Plan

Code change only. Toggle behavior flips to effective on next cycle; no operator action needed.

## Open Questions

None.