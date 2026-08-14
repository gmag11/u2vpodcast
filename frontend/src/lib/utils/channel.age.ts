export function lastEpisodeAge(lastDate: string | null): string {
	if (!lastDate) return '';
	const days = Math.floor((Date.now() - new Date(lastDate).getTime()) / 86_400_000);
	if (days < 7) return `${days}d`;
	if (days < 30) return `${Math.floor(days / 7)}w`;
	if (days < 365) return `${Math.floor(days / 30)}m`;
	return `${Math.floor(days / 365)}y`;
}
