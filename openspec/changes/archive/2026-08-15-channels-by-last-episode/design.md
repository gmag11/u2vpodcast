## Context

`GET /api/1.0/channels/` returns all channels via `Channel::read_all`. The prior backend-only ordering approach (ORDER BY last episode in `read_all`) is superseded: sorting must live in the frontend so additional sort keys (alphabetical, id) can be added later. The frontend `ChannelsView` currently renders channels in API order and the `Channel` type carries no last-episode date, so a frontend sort requires the API to expose that date. `episodes.published_at` is `DATETIME NOT NULL` and `get_max_date` (`channel.rs`) already computes `MAX(published_at)` per channel.

## Goals / Non-Goals

**Goals:**
- Expose each channel's last episode date in the channel list API response.
- Sort the channel list in the frontend by that date, newest first, empty channels last.
- Isolate the sort so future keys (alphabetical, id) are additive.

**Non-Goals:**
- No sort-selection UI; "by last update" is the only ordering for now.
- No schema migration; `last_date` is computed from existing `episodes`.
- No change to `read_with_pagination` (created_at ASC) or the single-channel `read` handler beyond the new nullable field.

## Decisions

### 1. `last_date` as an `Option` field on `Channel`, read tolerantly

Add `pub last_date: Option<DateTime<Utc>>` to the `Channel` struct and change `from_row` to read it via `row.try_get("last_date").unwrap_or(None)`. Queries that include the column (the list query) populate it; queries that do not (`INSERT/UPDATE ... RETURNING *`, `read`, `read_by_slug`) get `None` instead of panicking.

Rationale: adding the field to every `RETURNING *` query would be invasive; `try_get` makes it a soft column everywhere. `Option` naturally encodes "channel has no episodes yet".

### 2. Populate `last_date` only in the list query

`read_all` becomes:

```sql
SELECT c.*, e.last_date
FROM channels c
LEFT JOIN (
    SELECT channel_id, MAX(published_at) AS last_date
    FROM episodes GROUP BY channel_id
) e ON e.channel_id = c.id
ORDER BY e.last_date IS NULL, e.last_date DESC
```

The `ORDER BY` remains as a sane API default; the frontend sort is authoritative for the UI.

Rationale: one JOIN gives every channel its latest episode date in a single round-trip, reusing the same subquery shape already validated against SQLite.

### 3. Frontend sorts in a dedicated computed

In `ChannelsView`, add a `sortedChannels` computed that clones and sorts `channels` by `last_date` descending with nulls last, and feed it to the existing `filteredChannels` computed (which preserves input order). Refresh/create mutations keep working because the computed re-derives order.

Comparator:

```ts
const sortedChannels = computed(() =>
	[...channels.value].sort((a, b) => {
		if (!a.last_date && !b.last_date) return 0;
		if (!a.last_date) return 1;
		if (!b.last_date) return -1;
		return new Date(b.last_date).getTime() - new Date(a.last_date).getTime();
	})
);
```

Rationale: a single sort site makes future keys (title, id) a one-line comparator swap. Sorting the clone avoids mutating `channels`.

### 4. Type the new field on the frontend

`Channel` in `frontend/src/types.ts` gains `last_date: string | null`. Backend responses for create/update/read single channels will serialize `last_date: null` until their queries include the column — frontend code tolerates `null`.

## Risks / Trade-offs

- **`null` payload on create/update/read single-channel responses**: consumers must tolerate it. → All consumers either ignore the field or are updated to the new type.
- **Two orderings (API + frontend)**: redundant but harmless; the frontend sort is deterministic and authoritative.
- **Date parsing in JS**: `last_date` arrives as an RFC3339 string; `new Date(...)` handles it. → Comparator uses epoch millis.
- **JOIN cost on every list request**: same as the prior approach, accepted at this scale.
