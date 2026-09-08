export const filterBySearchWords = <T>(
	items: T[],
	query: string,
	getHaystack: (item: T) => string
): T[] => {
	const words = query
		.trim()
		.split(/\s+/)
		.filter(Boolean)
		.map((word) => word.toLowerCase());
	if (words.length === 0) return items;
	return items.filter((item) => {
		const haystack = getHaystack(item).toLowerCase();
		return words.every((word) => haystack.includes(word));
	});
};

// Keeps only the items whose id is favorited per the given predicate. Views
// pass the shared favorites store's `favoriteIdSet.has`, so a toggle in any
// card is reflected immediately without a refetch (episode-favorites).
export const filterByFavorites = <T extends { id: number }>(
	items: T[],
	isFavorite: (id: number) => boolean
): T[] => items.filter((item) => isFavorite(item.id));
