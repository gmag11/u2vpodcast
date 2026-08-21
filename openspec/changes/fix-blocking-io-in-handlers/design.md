## Context

`YTInfo::new` (`src/models/ytinfo.rs`) is `async` on paper but performs a blocking `ureq` request with no timeout. Server runs `.workers(2)` (actix default worker per CPU/2 as configured) with `SqlitePoolOptions::max_connections(2)`, so blocking threads starve everything else.

## Goals / Non-Goals

**Goals:**
- Metadata fetch never blocks async worker threads.
- Hung upstream bounded by a timeout.

**Non-Goals:**
- No change to the extraction logic or the returned fields.
- No migration to a different HTTP library unless trivially safe.

## Decisions

- **Wrap `YTInfo::new` bodies in `actix_web::rt::task::spawn_blocking` at the call boundary** (or inside `new` if self-contained) so blocking code uses the blocking thread pool. This is the minimal change and `ureq` stays.
  - Alternative considered: switch to `reqwest` async client; bigger refactor, changes error type surface, more risk for the same outcome today.
- **Set an explicit ureq timeout** (e.g. configured seconds or a sane constant) so a hung host cannot block even blocking threads for long.
- Keep `YTInfo::new`'s `async` signature so call sites (`Channel::new`, `Channel::update_image`) compile unchanged; internally available to await the blocking task.

## Risks / Trade-offs

- [spawn_blocking still occupies a blocking-pool thread per request] → Accepted and bounded by the timeout; actix blocking pool is sized separately from the 2 workers.
- [Timeout value too aggressive could fail slow-but-fine fetches] → Mitigated: pick a generous default and keep it configurable if desired.

## Migration Plan

Code change only. Behavior for healthy upstreams is identical.

## Open Questions

Whether the timeout should be config-driven (new config key) or a hardcoded constant — implementation may choose a hardcoded generous default unless config plumbing is trivial.