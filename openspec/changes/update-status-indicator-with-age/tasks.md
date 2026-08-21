## 1. Channel Card Update

- [ ] 1.1 In `frontend/src/components/ChannelCard.vue`, remove the standalone top-left status dot span and the `title` attribute from the bottom-left age badge
- [ ] 1.2 Wrap dot and badge in a single `div.group.relative` in the bottom-left corner: dot (left) then age badge (right); render group only when sync status or age exists
- [ ] 1.3 Add computed `syncStatusTooltip` producing `Updated <age> ago. Status: Ok|Error` from `syncAgeLabel` and `channel.last_sync_ok`, rendered as one shared hover tooltip on the group

## 2. Verification

- [ ] 2.1 Run frontend lint/type-check (`cd frontend && npm run build` or equivalent) and existing tests
- [ ] 2.2 Manually verify in browser: green card shows dot left of badge with "Status: Ok" tooltip; failed-sync card shows red dot with "Status: Error"; never-synced channel shows neither; top-left corner is clean
