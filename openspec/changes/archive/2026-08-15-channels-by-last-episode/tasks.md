## 1. Backend: expose last episode date

- [x] 1.1 Add `pub last_date: Option<DateTime<Utc>>` to the `Channel` struct in `src/models/channel.rs`
- [x] 1.2 Update `Channel::from_row` to read `last_date` via `row.try_get("last_date").unwrap_or(None)` so queries without the column (create/update/read) yield `None` instead of failing
- [x] 1.3 Update `read_all` to select `c.*, e.last_date` from a `LEFT JOIN` over `MAX(episodes.published_at)`, keeping the existing `ORDER BY e.last_date IS NULL, e.last_date DESC`
- [x] 1.4 Run `cargo check` to confirm the backend compiles

## 2. Frontend: sort the channel list

- [x] 2.1 Add `last_date: string | null` to the `Channel` interface in `frontend/src/types.ts`
- [x] 2.2 Add a `sortedChannels` computed in `frontend/src/views/ChannelsView.vue` that sorts by `last_date` descending with nulls last, and feed it to `filteredChannels`

## 3. Verify

- [x] 3.1 Run `pnpm test`, `pnpm run lint`, and `pnpm run build` in `frontend/`
- [x] 3.2 Confirm the SQL (channel list with and without episodes) returns the expected `last_date` values and ordering via a SQLite check
