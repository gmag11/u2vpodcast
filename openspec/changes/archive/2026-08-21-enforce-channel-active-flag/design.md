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

- **Add a dedicated `Channel::read_active` query used only by the worker, leaving `read_all` unfiltered.** The initial design assumed the worker was `read_all`'s only consumer, but auditing found two more: the SPA channel list endpoint (`GET /channels/`) must still show inactive channels so operators can re-enable them, and `migrate_slugs` must process every channel for backfill/rename. Filtering `read_all` would have hidden deactivated channels from the UI and skipped their slug migration — so the filter lives in a separate query used solely by `do_the_work`.
- **Do not daemonize anyway:** if a fully disabled-but-kept channel should stop being listenable too, that is out of scope; the toggle concerns syncing.

## Risks / Trade-offs

- [A future non-worker consumer of `read_all` might expect all channels] → Mitigated by checking consumers during implementation and by naming the filtered method explicitly if needed.
- [Inactive channels keep serving old episodes/feeds] → Accepted: disabling stops updates, not access; final policy can be tuned later.

## Migration Plan

Code change only. Toggle behavior flips to effective on next cycle; no operator action needed.

## Open Questions

None.