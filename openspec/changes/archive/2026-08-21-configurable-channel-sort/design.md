## Context

`GET /api/1.0/channels/` already returns `last_date` per channel and the frontend
sorts in `ChannelsView.sortedChannels` by `last_date` descending, nulls last
(`channels-list-ordering` spec). That spec already anticipates adding `title`/`id`
sort keys as a frontend-only change. Small pure helpers in `frontend/src/lib/utils/`
(`list.filter.ts`, `channel.age.ts`) are the established pattern for testable
logic, and `stores/theme.ts` establishes the `localStorage` persistence pattern.

## Goals / Non-Goals

**Goals:**
- Configurable sort key: last episode (default), title, id.
- Configurable sort direction: ascending/descending.
- In-list controls for both, visible on the channels view.
- Persist the selection across reloads.
- Keep the sort logic pure and unit-testable.

**Non-Goals:**
- No backend/API changes; the payload already provides everything the frontend
  needs.
- No per-user server-side preferences or multi-device sync.
- No sort controls on the episodes or history views (only the channels list).

## Decisions

### 1. Pure `sortChannels` helper

New module `frontend/src/lib/utils/channel.sort.ts`:

```ts
export type ChannelSortKey = 'last_date' | 'title' | 'id';
export type SortDirection = 'asc' | 'desc';

export function sortChannels<T extends { last_date: string | null; title: string; id: number }>(
	channels: T[],
	key: ChannelSortKey,
	direction: SortDirection
): T[];
```

Semantics per key (direction flips the primary comparator):
- `last_date`: channels with a `null` `last_date` are treated as the oldest, so
  they sort first when ascending and last when descending. `desc` (default) →
  newest first; `asc` → oldest first.
- `title`: case-insensitive comparison (lowercased `localeCompare`).
- `id`: numeric comparison.

The function clones the input before sorting; the caller's array is never
mutated. A null key/direction falls back to the default (`last_date` / `desc`).

Rationale: mirrors `filterBySearchWords` — pure, framework-free, unit-testable,
and reusable by any future view without touching the component.

### 2. `ChannelsView` state and persistence

Replace the hardcoded `sortedChannels` with:

```ts
const sortKey = ref<ChannelSortKey>('last_date');
const sortDirection = ref<SortDirection>('desc');
const sortedChannels = computed(() =>
	sortChannels(channels.value, sortKey.value, sortDirection.value)
);
```

Persistence follows `theme.ts`: a single `localStorage` key (e.g. `channel-sort`)
holding a JSON object `{ key, direction }`. `resolveInitial` reads and validates
the stored values defensively (unknown values fall back to the defaults) and is
applied on mount; every change writes through.

Rationale: one key, validated on read, survives reloads, and degrades silently if
the stored value is corrupted or missing.

### 3. Sort controls in the toolbar row

A small `SortControl.vue` component placed next to `SearchInput` (above the grid):

- Key picker: three segmented buttons — "Last episode", "Title", "Id" — using the
  existing `glass-card` / `surface-input` design tokens (`@phosphor-icons/vue`
  icons where helpful, already a dependency).
- Direction: a single icon button toggling `asc` ↔ `desc` (arrow up/down),
  reflecting the current state with `aria-pressed` and an `aria-label`.
- Keyboard accessible and labeled (`role="group"`, visible labels or
  `aria-label`), matching the accessibility level of the existing components.

Rationale: a dedicated component keeps `ChannelsView` from growing further and
matches the component-per-concern convention already used by `SearchInput`,
`AppToggle`, etc.

### 4. Tests

`channel.sort.test.ts` (Vitest, same style as `HistoryView.test.ts`) covering:
- each key × each direction,
- `last_date` null handling in both directions,
- input immutability (source array unchanged).
