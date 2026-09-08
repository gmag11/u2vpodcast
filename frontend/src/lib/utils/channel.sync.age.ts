export function lastSyncAge(lastSyncAt: string | null): string {
	if (!lastSyncAt) return '';
	const hours = Math.floor((Date.now() - new Date(lastSyncAt).getTime()) / 3_600_000);
	if (hours < 24) return `${hours}h`;
	const days = Math.floor(hours / 24);
	if (days < 7) return `${days}d`;
	if (days < 30) return `${Math.floor(days / 7)}w`;
	if (days < 365) return `${Math.floor(days / 30)}m`;
	return `${Math.floor(days / 365)}y`;
}
