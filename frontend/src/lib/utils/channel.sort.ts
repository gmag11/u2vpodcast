export type ChannelSortKey = 'last_date' | 'title' | 'id';
export type SortDirection = 'asc' | 'desc';

export const DEFAULT_SORT_KEY: ChannelSortKey = 'last_date';
export const DEFAULT_SORT_DIRECTION: SortDirection = 'desc';

interface SortableChannel {
	last_date: string | null;
	title: string;
	id: number;
}

const asKey = (key: unknown): ChannelSortKey =>
	key === 'title' || key === 'id' || key === 'last_date' ? key : DEFAULT_SORT_KEY;

const asDirection = (direction: unknown): SortDirection =>
	direction === 'asc' || direction === 'desc' ? direction : DEFAULT_SORT_DIRECTION;

export function sortChannels<T extends SortableChannel>(
	channels: T[],
	key: ChannelSortKey,
	direction: SortDirection
): T[] {
	const sortKey = asKey(key);
	const sortDirection = asDirection(direction);
	const multiplier = sortDirection === 'asc' ? 1 : -1;

	return [...channels].sort((a, b) => {
		if (sortKey === 'title') {
			return a.title.toLowerCase().localeCompare(b.title.toLowerCase()) * multiplier;
		}
		if (sortKey === 'id') {
			return (a.id - b.id) * multiplier;
		}
		if (!a.last_date && !b.last_date) return 0;
		if (!a.last_date) return 1;
		if (!b.last_date) return -1;
		return (new Date(a.last_date).getTime() - new Date(b.last_date).getTime()) * multiplier;
	});
}
