import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { usePlayerStore } from '@/stores/player';
import type { Episode } from '@/types';

/**
 * Minimal HTMLAudioElement stand-in. jsdom ships the class but does not
 * implement playback (its play() is a no-op warning), so the player store's
 * `ensureAudio()` wiring gets a controllable fake in these tests.
 */
class MockAudioElement {
	src = '';
	currentTime = 0;
	duration = 0;
	volume = 1;
	muted = false;
	playbackRate = 1;
	paused = true;
	preload = 'metadata';

	private listeners: Record<string, Array<() => void>> = {};

	play = vi.fn(async () => {
		this.paused = false;
	});

	pause = vi.fn(() => {
		this.paused = true;
	});

	load = vi.fn();

	addEventListener(event: string, listener: () => void) {
		(this.listeners[event] ??= []).push(listener);
	}

	removeEventListener(event: string, listener: () => void) {
		this.listeners[event] = (this.listeners[event] ?? []).filter((l) => l !== listener);
	}
}

const AudioClass = MockAudioElement as unknown as typeof HTMLAudioElement;

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

describe('player store auto-advance', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		setActivePinia(createPinia());
	});

	it('seeds the queue from the index after the played episode', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);
		expect(player.upNext.map((e) => e.id)).toEqual([2, 3]);
		expect(player.currentEpisode?.id).toBe(1);
	});

	it('plays from the middle of the list keeps only the tail in the queue', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4)];
		await player.play(episodes[1], episodes);
		expect(player.upNext.map((e) => e.id)).toEqual([3, 4]);
	});

	it('leaves the queue empty when playing without a list', async () => {
		const player = usePlayerStore();
		await player.play(episode(5));
		expect(player.upNext).toEqual([]);
	});

	it('advance plays the next episode and drains the queue', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);
		expect(player.upNext).toHaveLength(2);

		await player.advance();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.upNext.map((e) => e.id)).toEqual([3]);
	});

	it('advance on an empty queue stops playback and resets the position', async () => {
		const player = usePlayerStore();
		await player.play(episode(1));
		expect(player.upNext).toEqual([]);

		// simulate being mid-track: the shared element's currentTime is only
		// mirrored into state via the timeupdate event, so set the store
		// state directly (the setup-store ref is unwrapped and writable)
		player.currentTime = 42;
		expect(player.currentTime).toBe(42);

		await player.advance();
		expect(player.playing).toBe(false);
		expect(player.stopped).toBe(true);
		expect(player.currentTime).toBe(0);
		expect(player.upNext).toEqual([]);
	});
});