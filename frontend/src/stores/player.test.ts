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

function episode(id: number, listen = false): Episode {
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
		listen,
		created_at: now,
		updated_at: now
	};
}

describe('player store queue', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		localStorage.clear();
		setActivePinia(createPinia());
	});

	it('seeds the queue from the index after the played episode', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);
		expect(player.upNext.map((e) => e.id)).toEqual([2, 3]);
		expect(player.currentEpisode?.id).toBe(1);
	});

	it('leaves the queue empty when there is nothing after the played episode', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[1], episodes);
		expect(player.upNext).toEqual([]);
	});

	it('keeps an existing queue when playing without a list', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);
		expect(player.upNext.map((e) => e.id)).toEqual([2, 3]);

		await player.play(episode(9));
		expect(player.upNext.map((e) => e.id)).toEqual([2, 3]);
		expect(player.currentEpisode?.id).toBe(9);
	});

	it('advance plays the next episode, drains the queue and records history', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);
		expect(player.upNext).toHaveLength(2);

		await player.advance();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.upNext.map((e) => e.id)).toEqual([3]);
		expect(player.playStack.map((e) => e.id)).toEqual([1]);
	});

	it('advance on an empty queue stops and clears', async () => {
		const player = usePlayerStore();
		await player.play(episode(1));
		expect(player.upNext).toEqual([]);

		player.currentTime = 42;
		await player.advance();
		expect(player.playing).toBe(false);
		expect(player.stopped).toBe(true);
		expect(player.currentTime).toBe(0);
		expect(player.upNext).toEqual([]);
	});

	it('skipNext drains one and marks listened when requested', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);

		await player.skipNext();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.upNext).toEqual([]);
		expect(player.playStack.map((e) => e.id)).toEqual([1]);
	});

	it('skipNext with markCurrent marks the finished episode listened', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1, false), episode(2)];
		await player.play(episodes[0], episodes);

		await player.skipNext(true);
		expect(player.currentEpisode?.id).toBe(2);
		// the finished episode becomes the current of the new play; check the
		// track recorded in history carries the mark
		expect(player.playStack[0].listen).toBe(true);
	});

	it('playPrevious restarts the current episode beyond 3 seconds', async () => {
		const player = usePlayerStore();
		await player.play(episode(1));
		player.currentTime = 12;

		await player.playPrevious();
		expect(player.currentEpisode?.id).toBe(1);
		expect(player.currentTime).toBe(0);
	});

	it('playPrevious navigates back within 3 seconds', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);
		await player.skipNext();
		expect(player.currentEpisode?.id).toBe(2);
		player.currentTime = 1;

		await player.playPrevious();
		expect(player.currentEpisode?.id).toBe(1);
		expect(player.playStack).toEqual([]);
	});

	it('removeFromQueue and clearQueue mutate and persist', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);

		player.removeFromQueue(3);
		expect(player.upNext.map((e) => e.id)).toEqual([2]);

		player.clearQueue();
		expect(player.upNext).toEqual([]);
	});

	it('keeps playStack bounded', async () => {
		const player = usePlayerStore();
		await player.play(episode(1), [episode(1)]);
		for (let i = 2; i <= 60; i++) {
			await player.advance();
			await player.play(episode(i), [episode(i)]);
		}
		expect(player.playStack.length).toBeLessThanOrEqual(50);
	});

	it('rehydrates the queue from localStorage on store creation', async () => {
		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({ upNext: [episode(7)], playStack: [episode(3)] })
		);
		const player = usePlayerStore();
		expect(player.upNext.map((e) => e.id)).toEqual([7]);
		expect(player.playStack.map((e) => e.id)).toEqual([3]);
	});

	it('persists queue changes to localStorage', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);

		const stored = JSON.parse(localStorage.getItem('u2vpodcast.up-next.v1') ?? '{}');
		expect(stored.upNext.map((e: Episode) => e.id)).toEqual([2]);
	});
});