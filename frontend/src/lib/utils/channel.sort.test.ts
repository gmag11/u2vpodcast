import { describe, expect, it } from 'vitest';
import { sortChannels } from '@/lib/utils/channel.sort';
import type { ChannelSortKey, SortDirection } from '@/lib/utils/channel.sort';

interface Item {
	id: number;
	title: string;
	last_date: string | null;
}

const items: Item[] = [
	{ id: 2, title: 'Linux y Tapas', last_date: '2026-08-10' },
	{ id: 3, title: 'bPodcast', last_date: '2026-08-05' },
	{ id: 1, title: 'Alpha', last_date: '2026-08-01' },
	{ id: 4, title: 'NoEpisodes', last_date: null }
];

describe('sortChannels', () => {
	it('sorts by last_date descending by default', () => {
		const result = sortChannels(items, 'last_date', 'desc');
		expect(result.map((i) => i.id)).toEqual([2, 3, 1, 4]);
	});

	it('sorts by last_date ascending, nulls treated as oldest (first)', () => {
		const result = sortChannels(items, 'last_date', 'asc');
		expect(result.map((i) => i.id)).toEqual([4, 1, 3, 2]);
	});

	it('treats channels without last_date as oldest in both directions', () => {
		const desc = sortChannels(items, 'last_date', 'desc');
		const asc = sortChannels(items, 'last_date', 'asc');
		expect(desc[desc.length - 1].id).toBe(4);
		expect(asc[0].id).toBe(4);
	});

	it('sorts by title case-insensitively', () => {
		const result = sortChannels(items, 'title', 'asc');
		expect(result.map((i) => i.id)).toEqual([1, 3, 2, 4]);
	});

	it('sorts by title case-insensitively descending', () => {
		const result = sortChannels(items, 'title', 'desc');
		expect(result.map((i) => i.id)).toEqual([4, 2, 3, 1]);
	});

	it('sorts by id numerically', () => {
		const result = sortChannels(items, 'id', 'asc');
		expect(result.map((i) => i.id)).toEqual([1, 2, 3, 4]);
	});

	it('sorts by id numerically descending', () => {
		const result = sortChannels(items, 'id', 'desc');
		expect(result.map((i) => i.id)).toEqual([4, 3, 2, 1]);
	});

	it('treats title sorting case-insensitively', () => {
		const mixed = [
			{ id: 1, title: 'Beta', last_date: null },
			{ id: 2, title: 'alpha', last_date: null },
			{ id: 3, title: 'ALPHA', last_date: null }
		];
		const result = sortChannels(mixed, 'title', 'asc');
		expect(result.map((i) => i.id)).toEqual([2, 3, 1]);
	});

	it('does not mutate the input array', () => {
		const input = [...items];
		sortChannels(items, 'last_date', 'desc');
		expect(items).toEqual(input);
	});

	it('falls back to defaults on unknown key and direction', () => {
		const result = sortChannels(items, 'unknown' as ChannelSortKey, 'invalid' as SortDirection);
		expect(result.map((i) => i.id)).toEqual([2, 3, 1, 4]);
	});
});
