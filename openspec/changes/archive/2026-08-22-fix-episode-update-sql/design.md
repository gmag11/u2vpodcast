## Context

`Episode::update` builds `UPDATE episodes SET channel_id=$2, title=$3, ... updated_at=$10 FROM episodes WHERE id=$1 RETURNING *`. Bind indices run 1..10 with `episode.id` first at `$1`.

Problems:

- SQLite supports `UPDATE ... FROM` since 3.33. Because the source table is the same as the target with no `ON`/WHERE join clause, every row of the target (after the `WHERE id = $1` filter is applied to the join) pairs with every source row — a cartesian self-join. The number of rows matched into the target is the number of source rows, so `RETURNING *` yields one row per episode in the table (e.g. hundreds).
- `sqlx`'s `fetch_one` errors when the query returns zero or more than one row — so the update would fail with a "returned 100 rows" style error, not a crash, but a guaranteed functional failure whenever more than one episode exists.
- Even if a given sqlx version tolerated it, the update would run `N` times per target row, needlessly rewriting rows.

The bind order itself is accidental: `$1` is the `id` (used in `WHERE`), and `updated_at` is the last bind. The rewrite keeps this order to minimize churn.

## Goals / Non-Goals

**Goals:**
- `Episode::update` updates exactly the episode with the given `id`.
- `RETURNING *` returns exactly one row on success.
- Update semantics of the `save()` dispatch are unchanged.

**Non-Goals:**
- No schema change.
- No change to how `updated_at` is computed.
- No migration of existing data (nothing is corrupted: the statement errors, it does not corrupt rows).

## Decisions

- **Plain single-table UPDATE** without `FROM`:

  ```sql
  UPDATE episodes SET channel_id = $2, title = $3, description = $4, yt_id = $5,
         published_at = $6, duration = $7, image = $8, listen = $9,
         updated_at = $10 WHERE id = $1 RETURNING *;
  ```

  Bind order (`id` first, `updated_at` last) is preserved so both the statement and `from_row` map line up untouched.
- **Regression test:** create a channel + episode through the existing model methods, build an `Episode` copy with the same `id` and modified fields, call `save()`, and assert: `Ok`, returned row id matches, changed column persisted, `updated_at` refreshed, and exactly one row touched (the test DB has ≥2 episodes at that point to make the old query fail).
- **Keep `create` untouched** — it is already a correct single-table insert.

## Risks / Trade-offs

- [A future requirement could legitimately need UPDATE…FROM] → Then it needs an explicit join condition; this change only removes the accidental implicit cartesian join.
- [Test DB setup cost] → Reuse the existing in-memory test DB pattern (migrations + model helpers); negligible.

## Migration Plan

1. Rewrite the SQL string in `Episode::update`.
2. Add the regression test.
3. Run the full suite (all other queries already assertions-covered).
4. No production data migration required.

## Open Questions

None.