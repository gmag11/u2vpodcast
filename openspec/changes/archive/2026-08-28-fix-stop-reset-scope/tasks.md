# Fix: Scope Stop Progress Reset to the Card Control — Tasks

## 1. Player store

- [x] 1.1 Restore `resetPosition(episode)` in `frontend/src/stores/player.ts` (writes 0 via the API, keeps the listened mark, no-op at 0)
- [x] 1.2 Rewrite `stop(target?)`: reproducing current episode → `haltPlayback` (keep position); card path (`target` given) not reproducing → `resetPosition(target)`; persistent-bar path (no target) not reproducing → converge to stopped state without reset

## 2. Episode card

- [x] 2.1 In `frontend/src/components/EpisodeCard.vue`, set both stop buttons back to `:disabled="isCurrent && player.loading"` so a non-current card's stop remains clickable (reset affordance)

## 3. Tests

- [x] 3.1 Restore/keep the card-path reset tests in `player.test.ts` (non-current target resets; card stop on a paused/stopped current episode resets keeping the listened mark)
- [x] 3.2 Keep the persistent-bar keep tests (second press keep, paused current keep, listened keep) calling `stop()` with no target
- [x] 3.3 Restore the `EpisodeCard.test.ts` test asserting stop is enabled on a non-current episode and forwards the episode to `player.stop`

## 4. Verification

- [x] 4.1 Run `npm run typecheck` and `npx vitest run` in `frontend/`
- [x] 4.2 Run `npx prettier --check .` on the changed files
- [x] 4.3 Manual check: play to a point — the persistent bar stop keeps the resume point; a card's stop on a non-reproducing episode clears that episode's position