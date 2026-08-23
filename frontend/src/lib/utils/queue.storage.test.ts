import { beforeEach, describe, expect, it } from 'vitest';
import { loadQueue, saveQueue } from '@/lib/utils/queue.storage';
import type { Episode } from '@/types';

function episode(id: number): Episode {
	const now = new Date();
	return {
		id,
		channel_id: 1,
		channel_slug: 'c',
		channel_title: 'Channel',
		title: `Episode ${id}`,
		description: '',
		yt_id: `yt${id}`,
		webpage_url: 'https://www.youtube.com/watch',
		published_at: now,
		duration: '00:10:00',
		image: '',
		listen: false,
		created_at: now,
		updated_at: now
	};
}

describe('queue.storage', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	it('round-trips upNext and playStack', () => {
		saveQueue({ upNext: [episode(1), episode(2)], playStack: [episode(3)] });
		const loaded = loadQueue();
		expect(loaded).not.toBeNull();
		expect(loaded!.upNext.map((e) => e.id)).toEqual([1, 2]);
		expect(loaded!.playStack.map((e) => e.id)).toEqual([3]);
	});

	it('returns null when nothing is stored', () => {
		expect(loadQueue()).toBeNull();
	});

	it('discards malformed payloads', () => {
		localStorage.setItem('u2vpodcast.up-next.v1', 'not json');
		expect(loadQueue()).toBeNull();

		localStorage.setItem('u2vpodcast.up-next.v1', JSON.stringify({ upNext: 'nope' }));
		expect(loadQueue()).toBeNull();

		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({ upNext: [{ bad: true }], playStack: [] })
		);
		// Arrays are shape-checked; item contents are trusted from our own writer
		expect(loadQueue()).not.toBeNull();
	});

	it('survives an unreadable payload produced by older versions', () => {
		localStorage.setItem('u2vpodcast.up-next.v1', JSON.stringify({ upNext: [] }));
		expect(loadQueue()).toBeNull();
	});
});