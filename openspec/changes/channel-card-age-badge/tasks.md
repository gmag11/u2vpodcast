## 1. Implement the age badge

- [x] 1.1 Add a `lastEpisodeAge(lastDate: string | null): string` helper (truncated `d`/`w`/`m`/`y`, `''` for null) to `frontend/src/lib/utils/channel.age.ts`
- [x] 1.2 Make the card root `relative` and render an absolutely-positioned badge (`v-if="ageLabel"`) in the top-right corner with the computed label
- [x] 1.3 Confirm cards without `last_date` render no badge

## 2. Verify

- [x] 2.1 Run `pnpm test`, `pnpm run lint`, and `pnpm run build` in `frontend/`
- [x] 2.2 Add unit tests for `lastEpisodeAge` (days/weeks/months/years truncation, null, sub-day) and visually confirm badge placement on the channels screen
