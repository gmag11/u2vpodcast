## 1. Keyboard Playback Behavior

- [x] 1.1 Extend the player store's window keydown handling so spacebar pauses playing audio, resumes paused audio, and does nothing when stopped or unloaded; verify focused editable and native interactive controls retain their normal behavior with focused unit tests.

## 2. Verification

- [x] 2.1 Add player store tests covering playing, paused, stopped, unloaded, unfocused-document, editable-field, and native interactive-control spacebar cases; verify with `pnpm test -- src/stores/player.test.ts` from `frontend/`.
- [x] 2.2 Run `pnpm typecheck` from `frontend/` and verify the keyboard handler changes introduce no TypeScript errors.
