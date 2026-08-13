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
