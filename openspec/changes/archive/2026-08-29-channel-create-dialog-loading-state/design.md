## Context

The Add Channel dialog (`frontend/src/components/AddChannelDialog.vue`) is a controlled component: it emits `save` with a fully assembled `Channel` object, and the parent `ChannelsView.vue` performs the async API call (`api.createChannel` / `api.updateChannel`). Channel creation includes an upstream link check / metadata fetch on the backend (see `channel-metadata-fetch`), which takes a few seconds. During that window the dialog remains open (`showAddDialog` is only set to `false` after the await resolves) and both the primary and Cancel buttons stay clickable, enabling duplicate submissions and ambiguous UX.

The app already has a global loading overlay (`AppLoading.vue` driven by a Pinia `loading` store), but it renders a full-screen modal and blocks the whole UI — not the desired feedback for this dialog, which must stay visible and interactive (minus the two action buttons) while the request runs.

## Goals / Non-Goals

**Goals:**
- Disable both the primary action button and the Cancel button while the dialog's save request is in flight.
- Show an animated spinner in place of the primary button label during the pending state.
- Restore buttons and label once the request settles (success or error).
- Keep the change contained to the frontend; no backend or API-contract changes.

**Non-Goals:**
- Not introducing a new global loading overlay or blocking the whole screen.
- Not preventing dialog dismissal via the close (X) button, Escape, or clicking outside — the user asked only about the two action buttons; the request completes regardless and the dialog closes on settle.
- Not adding new i18n copy (the label is swapped for a spinner, not for text).
- Not refactoring where the API call lives (the parent keeps owning the request).

## Decisions

### D1: Parent-driven pending state via a `saving` prop

`ChannelsView` tracks a boolean around its awaited API calls (`saving = true` before `await`, `false` after, in a `finally`-style flow) and passes it to the dialog as `:saving`. The dialog only renders; it does not know about the request itself.

- **Why**: The request already lives in the parent; the dialog is a controlled component. Minimal, consistent with the existing data flow.
- **Alternatives considered**: (a) Dialog performs the API call itself — larger refactor, duplicates backend-communication logic already in `ChannelsView`, and splits responsibilities. (b) Reuse the global Pinia `loading` store — shows the full-screen overlay, contradicting the goal of keeping the dialog open with only its action buttons disabled.

### D2: Disable both buttons by binding `:disabled="saving"`

`AppButton.vue` already renders a native `<button>` with a `disabled:opacity-50` utility, so no styling changes are needed — only the prop binding. Cancel's `handleCancel` is naturally a no-op for disabled buttons; the request also continues because the save already fired.

- **Why**: Native `disabled` is the simplest correct mechanism; no extra guard logic needed in the handlers.
- **Alternative considered**: Guarding `handleSave`/`handleCancel` with an early return — unnecessary since `disabled` already blocks clicks, and it would not prevent double-submit from Enter-key form submission, which the disabled state does prevent.

### D3: Inline spinner SVG in the primary button when `saving`

Reuse the exact spinner markup from `AppLoading.vue` (an `animate-spin` SVG circle/path), sized for a button (`h-5 w-5`) and using `text-current` so it inherits the button color. The label renders via `v-if`/`v-else` so only one is shown at a time.

- **Why**: Zero new dependencies, and it visually matches the app's existing loading indicator.
- **Alternative considered**: A dedicated `AppSpinner.vue` component — over-engineering for one usage today; can be extracted later if more call sites appear.

## Risks / Trade-offs

- [Saving flag left `true` on request error] → Clear the flag in a `finally`-style flow (set `saving = false` after the awaited call regardless of `result.ok`), and add a component test asserting buttons re-enable after resolve/reject.
- [Spinner swap changes button width, causing layout shift] → Acceptable: the button is full-width (`w-full`), so the visible change is a centered spinner; low risk.
- [Close button still dismisses the dialog mid-request] → Accepted as out of scope (Non-Goals); the pending state applies only to the two action buttons per the request.

## Migration Plan

Single-commit frontend change, no data or backend migration. Deployable independently: update `AddChannelDialog.vue`, `ChannelsView.vue`, and the new test together. Rollback: revert the commit; behavior returns to the current state (buttons active during the request).

## Open Questions

None — the requirement is fully specified by the user's description and the `channel-create-dialog-pending` spec.