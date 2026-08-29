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
		playback_speed: 1,
		title: `Episode ${id}`,
		description: '',
		yt_id: `yt${id}`,
		webpage_url: 'https://www.youtube.com/watch',
		published_at: now,
		duration: '00:10:00',
		image: '',
		listen: false,
		position_seconds: 0,
		listened_at: null,
		favorite: false,
		created_at: now,
		updated_at: now
	};
}

describe('queue.storage', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	it('round-trips upNext, playStack and currentEpisode', () => {
		saveQueue({
			upNext: [episode(1), episode(2)],
			playStack: [episode(3)],
			currentEpisode: episode(9)
		});
		const loaded = loadQueue();
		expect(loaded).not.toBeNull();
		expect(loaded!.upNext.map((e) => e.id)).toEqual([1, 2]);
		expect(loaded!.playStack.map((e) => e.id)).toEqual([3]);
		expect(loaded!.currentEpisode?.id).toBe(9);
	});

	it('round-trips playback modes and the authored seed order', () => {
		saveQueue({
			upNext: [episode(2), episode(1)],
			playStack: [],
			currentEpisode: null,
			// the queue is shuffled, the seed keeps the authored order
			seedOrder: [episode(1), episode(2)],
			shuffle: true,
			repeat: 'all'
		});
		const loaded = loadQueue();
		expect(loaded!).not.toBeNull();
		expect(loaded!.shuffle).toBe(true);
		expect(loaded!.repeat).toBe('all');
		expect(loaded!.seedOrder!.map((e) => e.id)).toEqual([1, 2]);
		expect(loaded!.upNext.map((e) => e.id)).toEqual([2, 1]);
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

	it('normalizes legacy payloads without currentEpisode to null', () => {
		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({ upNext: [episode(1)], playStack: [] })
		);
		const loaded = loadQueue();
		expect(loaded).not.toBeNull();
		expect(loaded!.upNext.map((e) => e.id)).toEqual([1]);
		expect(loaded!.currentEpisode).toBeNull();
	});

	it('defaults modes and seed order for legacy payloads', () => {
		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({ upNext: [episode(1), episode(2)], playStack: [], currentEpisode: null })
		);
		const loaded = loadQueue();
		expect(loaded!.shuffle).toBe(false);
		expect(loaded!.repeat).toBe('none');
		// without a stored seed, the queue itself becomes the source
		expect(loaded!.seedOrder!.map((e) => e.id)).toEqual([1, 2]);
	});
});
