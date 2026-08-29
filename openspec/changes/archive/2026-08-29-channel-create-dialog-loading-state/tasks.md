## 1. Add pending state to the Add Channel dialog

- [x] 1.1 Add a `saving?: boolean` prop (default `false`) to `frontend/src/components/AddChannelDialog.vue`
- [x] 1.2 Bind `:disabled="saving"` to the primary action button (Create Channel / Save Changes label) and to the Cancel button in the dialog template
- [x] 1.3 In the primary button, replace the label with an inline animated spinner (`animate-spin` SVG, `h-5 w-5`, `text-current`, same markup as `AppLoading.vue`) when `saving` is true, using `v-if`/`v-else` so only spinner or label renders

## 2. Wire the pending state from the parent view

- [x] 2.1 In `frontend/src/views/ChannelsView.vue`, add a `saving` ref initialized to `false`
- [x] 2.2 In `saveChannel`, set `saving.value = true` before the awaited `api.createChannel` / `api.updateChannel` call and reset it to `false` after the await settles (use a try/finally or equivalent so it clears on success and error)
- [x] 2.3 Pass `:saving="saving"` to `<AddChannelDialog>` in the template

## 3. Tests

- [x] 3.1 Create `frontend/src/components/AddChannelDialog.test.ts` (mirroring the setup pattern from `AppHeader.test.ts`: `@vue/test-utils` `mount`, `testI18n`, Pinia if required)
- [x] 3.2 Add a test asserting both buttons are disabled when the `saving` prop is true
- [x] 3.3 Add a test asserting the primary button shows the spinner and hides its label text when `saving` is true
- [x] 3.4 Add a test asserting buttons are enabled and the label is shown again when `saving` is false

## 4. Verify

- [x] 4.1 Run `pnpm test` in `frontend/` and ensure the new tests pass
- [x] 4.2 Run `pnpm typecheck` and `pnpm lint` in `frontend/` and fix any issues
- [x] 4.3 Run `openspec validate` and confirm the change artifacts are valid