import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { lastEpisodeAge } from '@/lib/utils/channel.age';

const NOW = '2026-08-15T00:00:00.000Z';

function iso(daysAgo: number): string {
	return new Date(new Date(NOW).getTime() - daysAgo * 86_400_000).toISOString();
}

describe('lastEpisodeAge', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(NOW));
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('returns an empty string when last date is null', () => {
		expect(lastEpisodeAge(null)).toBe('');
	});

	it('formats age in days', () => {
		expect(lastEpisodeAge(iso(2))).toBe('2d');
	});

	it('truncates a week and a half to weeks', () => {
		expect(lastEpisodeAge(iso(11))).toBe('1w');
	});

	it('formats age in weeks', () => {
		expect(lastEpisodeAge(iso(21))).toBe('3w');
	});

	it('formats age in months', () => {
		expect(lastEpisodeAge(iso(180))).toBe('6m');
	});

	it('formats age in years', () => {
		expect(lastEpisodeAge(iso(365 * 3))).toBe('3y');
	});

	it('shows 0d for sub-day ages', () => {
		expect(lastEpisodeAge(new Date().toISOString())).toBe('0d');
	});
});
