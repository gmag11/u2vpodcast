## 1. Worker Filtering

- [x] 1.1 Add `Channel::read_active` (mirrors `read_all` with `WHERE c.active = 1`), auditing consumers: SPA list endpoint and `migrate_slugs` need all channels, so `read_all` stays unfiltered
- [x] 1.2 Use `Channel::read_active` in `do_the_work` so the scheduled worker iterates only active channels

## 2. Verification & Regression

- [x] 2.1 Verify with an inactive channel: worker cycle does not touch it, toggling back on resumes processing next cycle; manual refresh unaffected
- [x] 2.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #4 is resolved and no new bugs were introduced