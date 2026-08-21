## 1. Backend Validation & Guard

- [ ] 1.1 Validate `max >= 1` in `Channel::new` and `Channel::update`, returning a 4xx error for lower values
- [ ] 1.2 Guard `clean_channel`: when `max < 1`, skip pruning, log a notice, and do not mark the channel sync as failed for that reason

## 2. Frontend Clamp

- [ ] 2.1 Clamp the `max` input in `AddChannelDialog.vue` to `>= 1` before submit

## 3. Verification & Regression

- [ ] 3.1 Submit `max: 0` and `max: -5` via API and assert 4xx with no data change; verify a channel already stored with `max = -1` syncs without deletions or sync failures
- [ ] 3.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #7 resolved and no new bugs introduced