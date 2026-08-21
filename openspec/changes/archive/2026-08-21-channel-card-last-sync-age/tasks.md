## 1. Helper

- [x] 1.1 Add `frontend/src/lib/utils/channel.sync.age.ts` exporting `lastSyncAge(lastSyncAt: string | null): string` that returns '' when null, and truncated units from `last_sync_at`: `Nh` (<24h), `Nd` (<7d), `Nw` (<30d), `Nm` (<365d), `Ny` (>=365d), with `0h` for sub-hour ages
- [x] 1.2 Add `frontend/src/lib/utils/channel.sync.age.test.ts` mirroring `channel.age.test.ts` (fake timers) covering null, hours, sub-hour (0h), days, weeks, months, years

## 2. Component

- [x] 2.1 In `frontend/src/components/ChannelCard.vue`, import `lastSyncAge` and compute a `syncAgeLabel` from `props.channel.last_sync_at` via `computed`
- [x] 2.2 Add a non-interactive badge span in the bottom-left corner (`absolute bottom-4 left-4`), `v-if="syncAgeLabel"`, styled like the existing top-right age badge (pill, `bg-surface-high`), `z-10`
- [x] 2.3 Confirm the new badge does not overlap the sync-status dot (top-left) or the last-episode age badge (top-right) and does not clash with the bottom action row

## 3. Verification

- [x] 3.1 Run the frontend test suite (`pnpm test`) with the new helper tests passing
- [x] 3.2 Run `pnpm run build` (and type-check) with no errors
- [x] 3.3 Manually verify in the SPA: a never-synced channel shows no badge, a recently synced channel shows `Nh` (e.g. `1h`), and an older sync shows the escalated unit
