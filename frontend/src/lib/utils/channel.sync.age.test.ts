import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { lastSyncAge } from '@/lib/utils/channel.sync.age';

const NOW = '2026-08-15T00:00:00.000Z';

function hoursAgo(h: number): string {
	return new Date(new Date(NOW).getTime() - h * 3_600_000).toISOString();
}

describe('lastSyncAge', () => {
	beforeEach(() => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(NOW));
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('returns an empty string when last sync is null', () => {
		expect(lastSyncAge(null)).toBe('');
	});

	it('shows 0h for a fresh sync', () => {
		expect(lastSyncAge(new Date().toISOString())).toBe('0h');
	});

	it('formats age in hours', () => {
		expect(lastSyncAge(hoursAgo(1))).toBe('1h');
	});

	it('truncates one and a half hours to hours', () => {
		expect(lastSyncAge(hoursAgo(1.5))).toBe('1h');
	});

	it('formats age in days', () => {
		expect(lastSyncAge(hoursAgo(2 * 24))).toBe('2d');
	});

	it('truncates a week and a half to weeks', () => {
		expect(lastSyncAge(hoursAgo(11 * 24))).toBe('1w');
	});

	it('formats age in weeks', () => {
		expect(lastSyncAge(hoursAgo(21 * 24))).toBe('3w');
	});

	it('formats age in months', () => {
		expect(lastSyncAge(hoursAgo(180 * 24))).toBe('6m');
	});

	it('formats age in years', () => {
		expect(lastSyncAge(hoursAgo(365 * 3 * 24))).toBe('3y');
	});
});
