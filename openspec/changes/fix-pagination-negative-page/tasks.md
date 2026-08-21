## 1. Pagination Clamp

- [ ] 1.1 Apply `page.max(1)` before computing the offset in `User::read_with_pagination`
- [ ] 1.2 Apply the same clamp in `Channel::read_with_pagination` and `Episode::read_with_pagination`
- [ ] 1.3 Remove/harmonize any now-redundant guard in `src/handlers/users.rs`

## 2. Verification & Regression

- [ ] 2.1 Request `page=0` and `page=-3` on the reachable paginated endpoint(s) and assert `200` with first-page results; assert `page=2` returns the same items as before
- [ ] 2.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #10 resolved and no new bugs introduced