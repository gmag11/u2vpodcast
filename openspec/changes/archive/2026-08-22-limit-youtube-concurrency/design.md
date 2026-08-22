## Context

Today every YouTube interaction opens its own connection: `YTInfo::new` (metadata + cover images, `src/models/ytinfo.rs`, already moved to `spawn_blocking`), and yt-dlp executions in the worker (`do_the_work` → `update_channel` → `process_channel`, plus the periodic `Ytdlp::auto_update`). Nothing serializes them. The `channel-metadata-fetch` change bounded latency; this change bounds concurrency and adds politeness.

## Goals / Non-Goals

**Goals:**
- A single global slot serializing every YouTube-bound connection.
- Fixed cooldown between connections (configurable, default applied).
- Applies to metadata, images, yt-dlp downloads, and yt-dlp updates, regardless of trigger.

**Non-Goals:**
- No change to the sync/scheduling algorithm beyond serialization.
- No caching of YouTube responses (separate concern).
- No per-channel priority or queue limits beyond the single slot.

## Decisions

- **Global `tokio::sync::Semaphore::new(1)` as the single slot**, stored in a process-wide `OnceLock` so every call site (model layer, worker) shares it without plumbing `AppState` through non-handler code. `YTInfo::new` currently runs its fetch inside `spawn_blocking` (from `fix-blocking-io-in-handlers`); the semaphore is acquired *before* spawning and held across the fetch **and** the cooldown, so no second connection can sneak in while the first waits or cools down.
  - Alternatives rejected: per-operation locks (would not serialize across operation types), `AppState`-resident semaphore (would require threading state into `Channel::new`/`update_image`, which have no access today), and no throttle (status quo).
- **Cooldown as a sleep inside the slot:** after the connection ends (ok or err), the holder `sleep`s the configured duration before releasing the permit. This guarantees the gap even when the next waiters are unrelated callers. Default `3s`, overridable via a new optional `config.yml` key (`cooldown_seconds`, int); a `OnceLock<Duration>` initialized at startup.
- **yt-dlp wrapper:** a shared `throttled_run` helper acquires the slot, runs the `tokio::process` yt-dlp command, enforces cooldown, returns the exit result. Both `process_channel`'s download runs and `auto_update` go through it. `auto_update` actually hits GitHub, but the user asked to gate all yt-dlp executions and it is cheap to include; keeping one path avoids future YouTube-flavored updates bypassing the throttle.
- **Deadlock safety:** the semaphore permit is held by an `async` guard; release happens on drop (including panics/unwinds and error branches in the holder). The cooldown sleep is bounded (a few seconds), and acquiring is `await`, so waiters always resume in bounded time.

## Risks / Trade-offs

- [Longer wall-clock for multi-channel refresh (N channels × (runtime + cooldown))] → Accepted; guarantees a "human-like" cadence, which is the point. Document in the proposal/README notes.
- [A 30s metadata timeout plus cooldown means a hung upstream still stalls the slot for ~33s] → Bounded and safe: waiters wait, but nothing deadlocks; the API remains responsive because the fetch sits on the blocking pool.
- [Configuration coupling] → Cooldown enters `Config` and must be mirrored in `Config` load/serde defaults; small, contained.
- [Semaphore in OnceLock cannot be tested in isolation easily] → Extract the throttle guard into its own small module so it can be unit-tested with tiny cooldowns.

## Migration Plan

1. Add throttle module + config key (default if absent).
2. Wire `YTInfo::new` and yt-dlp execution paths.
3. Runtime verification: force-refresh N channels and observe strictly sequential yt-dlp runs with the configured gap; concurrent creates observe sequential metadata fetches; failure path still enforces cooldown.
4. Rollback: config default (absent) behaves like old code except for the fixed 1-at-a-time + cooldown; no migration needed.

## Open Questions

None.