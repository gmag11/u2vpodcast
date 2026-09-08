## Context

See `proposal.md` for motivation. The Vue/Pinia frontend owns one detached `HTMLAudioElement` inside `frontend/src/stores/player.ts`. That store already centralizes play, pause, queue navigation, original-timeline seeking, SponsorBlock adjustment, progress persistence, playback speed, and audio event handling. The browser currently receives no explicit Media Session metadata or action handlers, although its native controls may directly play or pause the audio element.

Media Session support varies by browser, operating system, installation mode, and action. Some implementations expose only a subset of actions or throw `NotSupportedError` while registering unsupported handlers. Position state also rejects non-finite durations, out-of-range positions, and invalid playback rates. The integration must therefore be optional at every boundary.

## Goals / Non-Goals

**Goals:**

- Keep the player store as the single authority for both in-app and system-originated playback actions.
- Support play, pause, queue navigation, relative seek, absolute seek, metadata, playback state, and position state where the browser exposes them.
- Keep state coherent when the browser directly changes the audio element rather than invoking an application handler.
- Prevent an operating-system control from reviving protected media after logout.
- Make browser capability differences isolated, testable, and non-fatal.

**Non-Goals:**

- Guarantee that every operating system displays every registered action; display policy remains controlled by the browser and OS.
- Add background downloads, offline playback, a service worker, native applications, or a new PWA installation flow.
- Add backend endpoints, database fields, or a Media Session polyfill.
- Change queue, previous-button, resume, SponsorBlock, listened-mark, or progress-persistence semantics.
- Treat a system next-track action as the persistent player's long-press “mark listened” gesture.

## Decisions

### 1. The player store owns Media Session integration

Media Session setup and synchronization will live beside the shared audio element in the player store. Handlers will call the existing public/internal player operations rather than manipulate queue arrays or audio state independently:

- `play` uses the store's resume/toggle path.
- `pause` uses the store's pause path.
- `nexttrack` uses short `skipNext()` semantics.
- `previoustrack` uses `playPrevious()` semantics.
- `seekforward` and `seekbackward` use relative seeking.
- `seekto` uses absolute seeking.

This preserves one behavior path for in-app controls, natural completion, media keys, lock-screen controls, and headsets. It also automatically retains queue persistence, resume lookup, progress flushing, playback modes, and SponsorBlock handling.

**Alternative considered:** Add integration to `PersistentPlayer.vue`. Rejected because the component is a presentation surface that can unmount on logout, while the audio element and native events are store-owned. Splitting action ownership would make hidden/background behavior depend on component lifecycle.

### 2. Registration is lazy, idempotent, and isolated per action

The store will initialize Media Session when the shared audio element is first needed. Initialization checks for `navigator.mediaSession` and is safe to call repeatedly. Each action registration is wrapped independently so a browser rejecting one action does not suppress the others. Async handlers will contain their promise failures so an operating-system gesture cannot create an unhandled rejection.

A small internal adapter boundary will isolate reads/writes to the browser API. Production uses `navigator.mediaSession`; tests provide a deterministic mock that stores handlers and records metadata, playback state, and position updates.

**Alternative considered:** Register all actions eagerly at application startup. Rejected because no authenticated/current episode exists then, and lazy registration follows the existing lazy audio lifecycle while avoiding stale media ownership before playback.

**Alternative considered:** Add a Media Session library or polyfill. Rejected because unsupported browsers must retain ordinary audio behavior, and a wrapper cannot make the OS expose controls that the browser does not implement.

### 3. Native audio events reconcile actual playback state

Application action handlers route through store methods, but the existing `play` and `pause` audio events remain authoritative evidence of what the element actually did. Their handlers will synchronize `playing`, `loading`, and `stopped` so direct browser control cannot leave combinations such as `playing=true` and `stopped=true`. Pause continues to trigger progress persistence.

Programmatic stop will explicitly publish an inactive Media Session playback state after the resulting pause event, because an audio `pause` event alone cannot distinguish stopped from paused. Ordinary pause publishes `paused`; successful play publishes `playing`.

**Alternative considered:** Update frontend state only inside Media Session action handlers. Rejected because browsers may apply default controls directly to the media element and because playback can also change through in-app controls, autoplay advance, or natural completion.

### 4. Metadata follows the current episode lifecycle

Whenever `loadEpisode` selects an episode, the store will publish `MediaMetadata` with:

- `title`: episode title;
- `artist`: channel title;
- `artwork`: the episode image when it is a non-empty usable URL.

Metadata is replaced before or while loading the new source so stale queue-item information is not retained. Failure to construct metadata with optional artwork falls back to text-only metadata. Metadata is cleared when there is no authenticated media session.

**Alternative considered:** Let the browser infer metadata from the document title and audio URL. Rejected because the detached audio element has no reliable episode/channel semantics and queue changes do not necessarily change page title.

### 5. Seek actions share the existing original-timeline helpers

Relative action details use a finite positive `seekOffset`, defaulting to the existing 15-second keyboard step otherwise. Absolute seek accepts only finite positions. Targets are clamped to `[0, duration]` when duration is usable and are passed through the existing SponsorBlock skip resolver. Requests that cannot be safely bounded are ignored.

The `fastSeek` hint will not select a separate path initially. Exact store state and SponsorBlock behavior are more important than an implementation-specific approximate seek; the regular seek path remains compatible across browsers.

**Alternative considered:** Assign `audio.currentTime` directly in each action. Rejected because that would duplicate clamping, omit immediate Pinia synchronization, and bypass SponsorBlock rules.

### 6. Playback and position state updates are event-driven and validated

A synchronization helper will publish Media Session playback state from store/audio transitions. A second helper will call `setPositionState` only when all values satisfy the API contract:

- finite `duration > 0`;
- finite `playbackRate > 0`;
- finite `position`, clamped to `0..duration`.

It will run after loaded metadata, time updates, explicit seeks, resume seeks, source changes, speed changes, play, pause, stop, and completion. Calls are guarded independently because browsers can expose Media Session while omitting or rejecting position state.

This event-driven approach reuses existing audio events and avoids a second polling loop.

**Alternative considered:** Update position only on a timer. Rejected because it would lag immediately after seek/source/speed changes and duplicate the audio element's existing time-update cadence.

### 7. Logout performs a dedicated native-media teardown

The auth-loss path in `App.vue` will call a player teardown operation that first flushes progress, then pauses and unloads the protected audio source, publishes inactive playback state, clears metadata, and unregisters Media Session action handlers. Queue/current-episode persistence may remain available to the authenticated application, but no loaded native source or stale handler may restart it while logged out.

On later authenticated playback, lazy initialization registers a fresh session and reloads the current episode through the existing playback path.

**Alternative considered:** Only set `playbackState = "none"` on logout. Rejected because metadata and handlers could remain visible, and a browser-default play command could still target the loaded audio element.

## Risks / Trade-offs

- [OS/browser action availability differs despite successful registration] -> Feature-detect and catch each action independently; document results with a representative manual compatibility matrix rather than promising identical chrome.
- [Native play events race with store action completion] -> Treat audio events as convergence points and keep registration handlers thin and idempotent.
- [Position state throws during source transitions] -> Validate and clamp every value, skip incomplete updates, and isolate browser API failures from audio playback.
- [Artwork URLs cannot be fetched by the OS surface] -> Keep artwork optional and fall back to title/channel metadata without failing the session.
- [Logout teardown changes restoration timing] -> Preserve queue/current episode data but unload only the native source; the existing stopped replay path reloads it after authentication.
- [Tests model API contracts but not OS display policy] -> Add real-device checks for desktop and mobile alongside deterministic unit tests.

## Migration Plan

1. Add Media Session test doubles and focused player-store tests before connecting browser actions.
2. Add guarded registration, action routing, metadata, and state synchronization to the shared player lifecycle.
3. Connect auth loss to native-media teardown and verify later authenticated replay.
4. Run the existing frontend suite, type checking, and production build using the locally available dependency cache; no registry access is required when dependencies are already installed.
5. Manually verify representative desktop and mobile environments, including media keys or system controls, lock-screen/notification controls, queue boundaries, seeks, and logout.

Rollback consists of removing the optional Media Session adapter and auth teardown connection; no stored data, server contract, or migration needs reversal.
