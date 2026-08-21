## 1. Fix the update statement

- [x] 1.1 Rewrite `Episode::update` SQL (`src/models/episode.rs`) as a single-table `UPDATE episodes SET ... WHERE id = $1 RETURNING *` with no `FROM` clause, preserving the existing bind order
- [x] 1.2 Confirm `updated_at = $10` binding still matches the current bind sequence

## 2. Regression coverage

- [x] 2.1 Add a test that: creates a channel and ≥2 episodes; calls `Episode::save` on an episode with an existing `id` and modified fields; asserts `Ok`, the row id, the persisted changes, and that exactly one row was updated
- [x] 2.2 Confirm the previously-broken cartesian query would fail the same test (i.e. the test is a real regression guard)
- [x] 2.3 Run the full test suite

## 3. Verification

- [x] 3.1 Confirm the worker create path (`Episode::new` → `create`) is unaffected
- [x] 3.2 Spot-check the SPA channel/episode detail rendering still uses unchanged read paths