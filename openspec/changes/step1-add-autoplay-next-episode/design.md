## Context

The web player is a single global Pinia store in `frontend/src/stores/player.ts` owning one `HTMLAudioElement`. The store currently registers an `ended` listener that calls `stop()`: `onEnded() { playing.value = false; stop(); }`. Each list view renders `EpisodeCard` items; the card calls `player.play(props.episode)` from its play button.

Two lists can seed playback:
- `frontend/src/views/EpisodesView.vue` — episodes of one channel, backend-ordered `published_at DESC`, then filtered client-side (`filterBySearchWords`) into `filteredEpisodes`.
- `frontend/src/views/HistoryView.vue` — the global feed (`GET /api/1.0/episodes/`), also filtered client-side into `filteredEpisodes`.

Both views already iterate `filteredEpisodes` (the exact visible order) to render cards. No backend, DB, or API work is required.

## Goals / Non-Goals

**Goals:**
- Play the next episode automatically when the current one ends.
- "Next" = the next entry of the visible list (respecting its current filter/sort) from which play was pressed.
- Keep behavior for single, "orphan" play identical to today (end → stop).

**Non-Goals:**
- Visible queue UI, previous/skip controls, persistence, shuffle/repeat (steps 2, 5).
- Server-side queue or changes to any API.
- Remembering playback position (step 3).

## Decisions

### Decision 1: Implicit queue seeded as a snapshot of the visible list

`play(episode: Episode, list?: Episode[])` gains an optional second argument. When supplied, the store builds `queue = list.slice(indexOf(episode) + 1)` — the entries that follow the played episode in the displayed order. `indexOf` uses episode `id` identity.

**Why**: the visible list is already the ordering the user is looking at (channel order or global feed order, with the current search filter applied). Slicing after the current index reproduces exactly "what would come next on this page".

**Alternative considered**: instructing the backend with a "next" query. Rejected: the visible order is a client-side artifact (client filtering, `ORDER BY published_at DESC`), so only the client has the full picture; a server round-trip per `ended` adds latency and complexity for no benefit at this stage.

### Decision 2: `onEnded` drains the queue, then falls back to today's stop

`onEnded` becomes:

```
advance():
  next = queue.shift()
  if next: play(next)         // same shared <audio>, new src
  else:    stop()             // existing behavior
```

The finished episode stays in `currentEpisode` until `play(next)` swaps the source, preserving the existing visual continuity. When the queue is empty, `stop()` is called exactly as today (reset position, auto-hide delay).

**Why**: minimal behavioral delta from today. The existing `play()` already handles source swap (`isSame` guard), metadata reload, and bar visibility, so advance reuses it unchanged.

### Decision 3: Cards receive the visible list as an optional prop

`EpisodeCard` gains `list?: Episode[]`. On play, the card calls `player.play(episode, props.list)`. Both `EpisodesView` and `HistoryView` pass `filteredEpisodes` (the list actually rendered). When `list` is absent/empty (e.g., future views, tests), the store behaves as today.

**Why**: keeps the store the single owner of queue logic while the views remain the source of the visible ordering — the card only forwards what it already iterates.

### Decision 4: Queue is an in-memory snapshot, not a live reference

The queue holds copies of the episode objects captured at play time, not a reference to the view's array.

**Why**: the view may change (new search query, data refresh) while playback progresses; a snapshot keeps "next" deterministic and stable for the whole listening session. Copying is cheap, and episodes are immutable-enough in practice.

## Risks / Trade-offs

- **[Risk] List changes mid-playback make the queue stale.** → Accepted by design: the snapshot matches the moment of play; refreshing the view does not silently alter what plays next. Re-seeding is possible by pressing play again.
- **[Risk] Huge filtered lists build a big in-memory queue.** → Lists are bounded by downloaded episodes per channel and by the global feed; holding references is negligible memory-wise.
- **[Trade-off] No "next" control (manual) yet.** → Deliberate: step 2 adds prev/next and the visible queue. Step 1 stays minimal and shippable.

## Migration Plan

1. Add `upNext` queue state and `play(episode, list?)` + `advance()` to the player store.
2. Wire `onEnded` to `advance()`.
3. Add `list` prop to `EpisodeCard`; pass `filteredEpisodes` from both views.
4. Add `frontend/src/stores/player.test.ts` covering seed, drain, and stop-on-empty.
5. `pnpm test` and `pnpm build` in `frontend/`; `cargo build` untouched but confirm.
6. Manually verify per `tasks.md` scenarios.

**Rollback**: revert frontend-only changes; no DB/API/config impact.

## Open Questions

None.