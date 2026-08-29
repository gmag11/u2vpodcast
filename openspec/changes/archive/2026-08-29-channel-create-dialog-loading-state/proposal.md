## Why

When a user accepts the channel creation dialog, the request takes a few seconds (the backend checks/fetches the channel link upstream) and during that time the dialog stays open with the Create and Cancel buttons still active. This can confuse users: double-submissions are possible, and it is unclear whether anything is happening.

## What Changes

- The Add Channel dialog gains a pending state driven from the parent view while the save request is in flight.
- While pending:
  - Both the primary action button (Create/Save) and the Cancel button are disabled, preventing duplicate submissions and premature cancel during the request.
  - The primary button's label is replaced by a loading spinner indicating the request is being processed.
- The buttons and label return to their normal state when the request completes, whether it succeeds or fails.
- No backend or API changes; this is a purely frontend UI-behavior change.

## Capabilities

### New Capabilities
- `channel-create-dialog-pending`: Defines the pending state of the Add Channel dialog while its save request (including the upstream link check) is in flight — both action buttons disabled and a spinner shown in place of the primary button label.

### Modified Capabilities
<!-- None: no existing spec-level behavior changes. -->

## Impact

- Frontend component: `frontend/src/components/AddChannelDialog.vue` (new `saving` prop, disabled buttons, spinner in primary button).
- Frontend view: `frontend/src/views/ChannelsView.vue` (tracks in-flight state around the awaited `api.createChannel` / `api.updateChannel` calls and passes it to the dialog).
- Reuses the spinner style already present in `frontend/src/components/AppLoading.vue` (no new dependency).
- Test: new component test for the dialog pending state; existing Vitest setup (`@vue/test-utils`, `testI18n`) covers mounting.
- Not affected: backend, API contract, i18n texts (label is replaced by a spinner, not new copy), or the `unified-design-system` tokens.