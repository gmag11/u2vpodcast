import { describe, expect, it } from 'vitest';
import { filterBySearchWords } from '@/lib/utils/list.filter';
import { toHHMMSS } from '@/lib/utils/formatter';

interface Item {
	title: string;
	description: string;
}

const items: Item[] = [
	{ title: 'Confesiones de Gasolinera', description: 'un canal de entretenimiento' },
	{ title: 'Linux y Tapas', description: 'kernel y cocina' }
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
