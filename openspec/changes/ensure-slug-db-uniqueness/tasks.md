## 1. Migration

- [ ] 1.1 Add a reversible migration: deterministic dedupe of existing duplicate slugs (append `-N` suffixes), then `CREATE UNIQUE INDEX` on `channels.slug`; down migration drops the index
- [ ] 1.2 Test migration on a seeded DB with a deliberately duplicated slug; assert dedupe + index creation and clean rollback

## 2. Conflict-Resilient Slug Creation

- [ ] 2.1 Update `unique_slug`/insert path to retry with the next suffix on a DB unique-violation, preserving existing behavior for the normal path

## 3. Delete Ownership Guard

- [ ] 3.1 Before `remove_dir_all`, verify no other channel row shares the slug directory; on conflict/suspicion log a warning and skip directory removal instead of wiping foreign files

## 4. Verification & Regression

- [ ] 4.1 Create two channels with the same title (also concurrently) and assert distinct slugs; delete one and assert the other's files survive
- [ ] 4.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #8 resolved and no new bugs introduced