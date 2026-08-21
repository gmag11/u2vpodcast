## 1. Worker Filtering

- [ ] 1.1 Filter `Channel::read_all` to active channels only (or add a dedicated query), verifying every consumer of `read_all` still gets the channels it needs
- [ ] 1.2 Confirm `do_the_work` now iterates only active channels

## 2. Verification & Regression

- [ ] 2.1 Verify with an inactive channel: worker cycle does not touch it, toggling back on resumes processing next cycle; manual refresh unaffected
- [ ] 2.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #4 is resolved and no new bugs were introduced