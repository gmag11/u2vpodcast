/**
 * Filters a list by a whitespace-separated word query, case-insensitively.
 * Every word of the query must appear in the item's haystack for the item to be kept.
 * @file lib/utils/helpers/list.filter.ts
 * @template T - The item type
 * @param {T[]} items - The list to filter
 * @param {string} query - The search query, split into words on whitespace
 * @param {(item: T) => string} getHaystack - Maps an item to the string searched against
 * @returns {T[]} The items matching every word of the query; `items` unchanged for an empty query
 */
export const filterBySearchWords = <T>(
    items: T[],
    query: string,
    getHaystack: (item: T) => string
): T[] => {
    const words = query.trim().split(/\s+/).filter(Boolean).map((word) => word.toLowerCase());
    if (words.length === 0) return items;
    return items.filter((item) => {
        const haystack = getHaystack(item).toLowerCase();
        return words.every((word) => haystack.includes(word));
    });
};
