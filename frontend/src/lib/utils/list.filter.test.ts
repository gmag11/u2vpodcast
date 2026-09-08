import { describe, expect, it } from 'vitest';
import { filterBySearchWords, filterByFavorites } from '@/lib/utils/list.filter';
import { toHHMMSS } from '@/lib/utils/formatter';

interface Item {
	title: string;
	description: string;
}

const items: Item[] = [
	{ title: 'Confesiones de Gasolinera', description: 'un canal de entretenimiento' },
	{ title: 'Linux y Tapas', description: 'kernel y cocina' },
	{ title: 'Otro Podcast', description: 'tercer episodio' }
];

const haystack = (item: Item) => `${item.title} ${item.description}`;

describe('filterBySearchWords', () => {
	it('returns all items for an empty query', () => {
		expect(filterBySearchWords(items, '', haystack)).toEqual(items);
		expect(filterBySearchWords(items, '   ', haystack)).toEqual(items);
	});

	it('filters case-insensitively by a single word', () => {
		const result = filterBySearchWords(items, 'gasolinera', haystack);
		expect(result).toHaveLength(1);
		expect(result[0].title).toBe('Confesiones de Gasolinera');
	});

	it('requires every word of a multi-word query', () => {
		const result = filterBySearchWords(items, 'linux tapas', haystack);
		expect(result).toHaveLength(1);
		expect(result[0].title).toBe('Linux y Tapas');
	});

	it('matches across all haystack fields', () => {
		const result = filterBySearchWords(items, 'cocina', haystack);
		expect(result).toHaveLength(1);
		expect(result[0].title).toBe('Linux y Tapas');
	});

	it('returns an empty list when nothing matches', () => {
		expect(filterBySearchWords(items, 'zzz', haystack)).toHaveLength(0);
	});
});

describe('filterByFavorites', () => {
	const favorited = [1, 3];
	const withIds = items.map((item, i) => ({ ...item, id: i + 1 }));

	it('keeps only the items whose id is favorited', () => {
		const result = filterByFavorites(withIds, (id) => favorited.includes(id));
		expect(result.map((r) => r.id)).toEqual([1, 3]);
	});

	it('returns an empty list when nothing is favorited', () => {
		expect(filterByFavorites(withIds, () => false)).toHaveLength(0);
	});
});

describe('toHHMMSS', () => {
	it('formats seconds as HH:MM:SS', () => {
		expect(toHHMMSS(3661)).toBe('01:01:01');
	});

	it('pads single digits with a leading zero', () => {
		expect(toHHMMSS(65)).toBe('01:05');
	});

	it('returns NaN for invalid input', () => {
		expect(toHHMMSS(NaN)).toBeNaN();
	});
});
