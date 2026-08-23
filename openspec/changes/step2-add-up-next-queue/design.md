## Context

After step 1 (`auto-advance`) the store drains a private, implicit queue on `ended`. The user cannot see which episodes are next, cannot remove or skip entries, and a page reload wipes the queue. The persistent bar (`frontend/src/components/PersistentPlayer.vue`) currently offers play/pause, stop, scrubber, volume, and speed — no queue surface.

The frontend is Vue 3 + Pinia; `radix-vue` is already a dependency (used for dropdowns in `AppHeader.vue`). Tests run with Vitest (`frontend/vitest.config.ts`) and `@vue/test-utils`.

## Goals / Non-Goals

**Goals:**
- Expose the queue as public store state with mutations: seed, `skipNext`, `playPrevious`, `removeFromQueue`, `clearQueue`.
- Add next/previous controls and a visible "Up next" panel to the persistent bar.
- Persist the queue in `localStorage` and rehydrate it on load.
- Keep `ended` consuming the queue and fall back to stop when empty.

**Non-Goals:**
- Manual reordering of the queue (step 4 covers saved playlists + reorder).
- Server-side queue storage, shuffle/repeat (steps 4, 5).
- Playback position persistence (step 3).

## Decisions

### Decision 1: Public `upNext` state + mutation API in the store

The store exposes `upNext: Ref<Episode[]>` plus functions that all mutate through a single helper so persistence hooks in once.

```
seedQueue(list, index)   // replaces upNext = list.slice(index+1)
skipNext()               // upNext.shift() → play it
playPrevious()           // dual: currentTime > 3s → seek(0) restart current;
                         //       otherwise pop playStack → play (applies step-3 resume)
removeFromQueue(id)      // filter out by episode id
clearQueue()
```

A private `playStack: Episode[]` records episodes as playback advances, enabling `playPrevious`. Pushing onto the stack happens in `advance()` right before switching source.

**Dual previous behavior**: matching podcast-client conventions, the previous control restarts the current episode when it has played more than 3 seconds, and only navigates to the previous episode within the first 3 seconds. This also means the previous control SHALL NOT be disabled while an episode is loaded (a restart is always possible), only the navigation half depends on `playStack`.

**Resume integration (step 3)**: navigating back to a previous episode reuses the standard play path (`play()`), so the step-3 resume policy (seek when saved position > 30s and < 95% of duration) applies automatically to the newly loaded episode — a freshly played or never-resumed episode starts at zero.

**Why**: a single mutation chokepoint keeps the queue consistent and gives one place to persist. `playPrevious` needs its own stack because the queue only holds *upcoming* items.

### Decision 2: Persistence in `localStorage` with full episode objects

A small helper (`frontend/src/lib/utils/queue.storage.ts`) serializes `upNext` (and `playStack`) to `localStorage` under key `u2vpodcast.up-next.v1` using full episode JSON; rehydration happens once in the store setup. The store persists after every queue mutation and on `watch` over the queue.

**Why**: full objects let the bar render title/thumbnail/channel without extra fetches; episode metadata (title, image, slug, yt_id) is stable. `localStorage` matches the "session/ephemeral" nature of the queue. Survives reload only within the same browser, which is exactly the scope.

**Alternative considered**: persisting only ids and refetching. Rejected: adds async rehydration and coupling to API latency for zero practical gain.

### Decision 3: Play with an explicit list re-seeds; orphan play keeps the queue

`play(episode, list?)`:
- with `list` → `seedQueue(list, index)` (replace as in step 1);
- without `list` → keep the existing `upNext` untouched.

**Why**: re-seeding from a list expresses "start listening this list from here". Keeping the queue on orphan play (e.g., replaying an episode from a card while a playlist is queued) preserves the planned flow. This differs from step 1 (where no list meant an empty queue) — the explicit, persisted queue is now the source of truth for `ended`.

### Decision 4: `ended` consumes and persists, repeat modes deferred

`onEnded` → `advance()`:
```
if repeat handling (step 5) ...
next = upNext.shift()
persist()
if next: play(next); playStack.push(current)
else: stop(); clearQueue()
```

**Why**: shift-then-persist keeps storage in lockstep with in-memory state; clearing on stop avoids a zombie queue after finishing. Repeat/shuffle deliberately left to step 5 to keep this change reviewable.

### Decision 5: Bar surfaces queue via next/prev + "Up next" popover

In `PersistentPlayer.vue`:
- prev/next buttons flanking the play/pause button (disabled when the respective stack/queue is empty).
- A queue button (list icon, `PhList`) toggles a `radix-vue` popover/dropdown anchored to the bar listing upcoming episodes (thumbnail, title, channel, remove `×`), plus "Clear all" and a count badge.

**Why**: a popover keeps the bar compact and does not require new routing; `radix-vue` is already used for menus so placement/focus behavior is consistent.

## Risks / Trade-offs

- **[Risk] Stale episode objects after data refresh.** Episode titles/images are stable once downloaded; worst case the bar shows an outdated thumbnail until next play. → Accepted; mitigable later by rehydrating from API.
- **[Risk] localStorage quota/serialization errors.** The list is bounded (tens of items); serialization wrapped in try/catch, malformed payloads discarded on load.
- **[Trade-off] Queue is per-browser, not per-user.** → Deliberate for this step; saved playlists (step 4) are the server-persisted, cross-device layer.

## Migration Plan

1. Store: public queue state, mutation API, `playStack`, persistence helper, rehydration.
2. `PersistentPlayer.vue`: prev/next buttons + "Up next" popover with remove/clear.
3. Views/cards: reuse step-1 `list` prop to seed on play.
4. Unit tests: queue mutations, localStorage round-trip, malformed-payload handling.
5. Manual verification per `tasks.md`.

**Rollback**: revert frontend changes; no DB/API impact.

## Open Questions

None.