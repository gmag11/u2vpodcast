import { test, expect } from 'vitest';
import { filterBySearchWords } from './list.filter';

interface Item {
    title: string;
    description: string;
    id: string;
}

const items: Item[] = [
    { title: 'Confesiones de Gasolinera', description: 'Historias de carretera', id: 'abc123' },
    { title: 'Linux y Tapas', description: 'Kernel y gastronomía', id: 'def456' },
    { title: 'Episodio 42', description: 'La respuesta', id: 'ghi789' }
];

const haystack = (item: Item) => [item.title, item.description, item.id].join(' ');

test('empty or blank query returns all items', () => {
    expect(filterBySearchWords(items, '', haystack)).toEqual(items);
    expect(filterBySearchWords(items, '   ', haystack)).toEqual(items);
});

test('single-word query matches case-insensitively', () => {
    expect(filterBySearchWords(items, 'gasolinera', haystack).map((i) => i.id)).toEqual(['abc123']);
    expect(filterBySearchWords(items, 'GASOLINERA', haystack).map((i) => i.id)).toEqual(['abc123']);
});

test('multi-word query requires all words in any order', () => {
    expect(filterBySearchWords(items, 'linux tapas', haystack).map((i) => i.id)).toEqual([
        'def456'
    ]);
    expect(filterBySearchWords(items, 'tapas linux', haystack).map((i) => i.id)).toEqual([
        'def456'
    ]);
});

test('no-match query returns an empty array', () => {
    expect(filterBySearchWords(items, 'zombies', haystack)).toEqual([]);
});

test('matches against any of the haystack fields', () => {
    expect(filterBySearchWords(items, 'ghi789', haystack).map((i) => i.id)).toEqual(['ghi789']);
});
