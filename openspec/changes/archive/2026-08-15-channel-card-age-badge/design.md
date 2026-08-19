## Context

`ChannelCard.vue` renders the channel list card: a `glass-card` root with padding, cover thumbnail, title, description, and a bottom action bar. The `Channel` type carries `last_date: string | null` (ISO timestamp of the newest episode, `null` when the channel has no episodes), added in `channels-by-last-episode`. The badge needs to sit in the top-right corner without disturbing the existing layout.

## Goals / Non-Goals

**Goals:**
- Show the truncated age of the last episode in the card's top-right corner.
- Reuse `last_date`; no new API fields.
- Keep the badge purely decorative (no click handling).

**Non-Goals:**
- No tooltip/exact date, no sorting/UI change, no backend change.
- No badge for channels without `last_date`.

## Decisions

### 1. Format helper with truncated units

Add a small pure function that converts `last_date` to an age label:

```ts
function lastEpisodeAge(lastDate: string | null): string {
	if (!lastDate) return '';
	const days = Math.floor((Date.now() - new Date(lastDate).getTime()) / 86_400_000);
	if (days < 7) return `${days}d`;
	if (days < 30) return `${Math.floor(days / 7)}w`;
	if (days < 365) return `${Math.floor(days / 30)}m`;
	return `${Math.floor(days / 365)}y`;
}
```

Unit precedence: weeks below 30 days, months below 365 days, years above. All divisions floor, so 1.5 weeks → `1w`. `null`/missing → `''` (no badge). Age < 1 day → `0d`.

Rationale: matches the requested examples (`2d`, `3w`, `6m`, `3y`) and the "no fractions" rule. Constant day lengths (30/365) are consistent with "months/years" shorthand and avoid calendar math complexity.

### 2. Absolute-positioned badge in the card corner

Make the card root `relative` and render the badge as an absolutely-positioned element at `top-3 right-3` (or `top-4 right-4`) inside the card padding:

```html
<span
	v-if="ageLabel"
	class="absolute right-4 top-4 z-10 rounded-full bg-surface-high px-2.5 py-1 text-xs font-semibold text-text shadow"
>{{ ageLabel }}</span>
```

`bg-surface-high` matches the existing tooltip surface so it reads as an overlay chip.

Rationale: absolute positioning keeps the flex layout untouched; `v-if` hides it when the channel has no episodes.

### 3. No new deps, no store

The helper lives in `ChannelCard.vue` (or `lib/utils` if preferred for tests); uses only `Date.now()`. Card stays a single component; badge carries no events.

## Risks / Trade-offs

- **`Date.now()` in component**: label computed once per render; fine for a list of cards. Refresh on long-lived screens would need a timer — out of scope, cards re-render on data changes.
- **`0d` for sub-day ages**: consistent truncation; acceptable and unambiguous.
- **Approximate month length (30d)**: standard shorthand, acceptable for a relative badge.
