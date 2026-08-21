## Why

`Episode::update` (`src/models/episode.rs:229`) runs:

```sql
UPDATE episodes SET ... FROM episodes WHERE id = $1 RETURNING *;
```

The same table appears as both the update target and in the `FROM` clause with no join condition. In SQLite (UPDATE…FROM, 3.33+) this is a self-join over the whole table: each target row is joined with every episode row, so the statement can produce multiple rows in `RETURNING *`, which breaks `fetch_one` (and does nonsensical work even when nobody notices). The intent is a simple single-row update by primary key. The path is rarely exercised today because the worker always creates episodes with `id=-1`, but `save()` on an existing episode hits it.

## What Changes

- Rewrite the `UPDATE` statement as a plain single-table update targetting `id`, keeping `RETURNING *` and the same bind order/semantics.
- Add a regression test that updates an existing episode through `Episode::save`/`update` and asserts a single updated row.

## Capabilities

### New Capabilities

- `episode-persistence`: Defines correct create/update semantics for the `episodes` table, including single-row updates without self-join side effects.

### Modified Capabilities

(none)

## Impact

- `src/models/episode.rs` (SQL statement in `update`).
- Test additions under the existing test facilities; no schema or API change.

## Non-Goals

- No change to the `save` create/update dispatch logic.
- No change to the `channels`-side SQL that was verified to be correct.