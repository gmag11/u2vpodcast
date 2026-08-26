import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { usePlayerStore, setRandomSource } from '@/stores/player';
import type { Episode } from '@/types';
import { api } from '@/lib/api/client';

vi.mock('@/lib/api/client', () => ({
	api: {
		updateEpisodeProgress: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		),
		getEpisodeProgress: vi.fn(() =>
			Promise.resolve({ ok: false, data: null, user: null, status: false })
		),
		getPlaylist: vi.fn(() => Promise.resolve({ ok: true, data: [], user: null, status: true })),
		addEpisodeToPlaylist: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		),
		removeEpisodeFromPlaylist: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		),
		reorderPlaylist: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		)
	}
}));

/**
 * Minimal HTMLAudioElement stand-in. jsdom ships the class but does not
 * implement playback (its play() is a no-op warning), so the player store's
 * `ensureAudio()` wiring gets a controllable fake in these tests.
 */
class MockAudioElement {
	static instances: MockAudioElement[] = [];
	static defaultClamp = 0;

	src = '';
	_currentTime = 0;
	// Simulates a browser that only allows seeking within its buffered prefix:
	// while `seekClampEnd > 0`, currentTime is clamped to it (like `seekable`).
	seekClampEnd = MockAudioElement.defaultClamp;
	duration = 0;
	volume = 1;
	muted = false;
	playbackRate = 1;
	paused = true;
	preload = 'metadata';

	get currentTime() {
		return this._currentTime;
	}

	set currentTime(v: number) {
		this._currentTime = this.seekClampEnd > 0 && v > this.seekClampEnd ? this.seekClampEnd : v;
	}

	private listeners: Record<string, Array<() => void>> = {};

	constructor() {
		MockAudioElement.instances.push(this);
	}

	play = vi.fn(async () => {
		this.paused = false;
		this.emit('play');
		this.emit('playing');
	});

	pause = vi.fn(() => {
		this.paused = true;
		this.emit('pause');
	});

	load = vi.fn(() => {
		// a real media element resets the playhead and duration on load() and
		// reports its metadata once parsed
		this.currentTime = 0;
		this.duration = 0;
		this.emit('loadedmetadata');
	});

	addEventListener(event: string, listener: () => void) {
		(this.listeners[event] ??= []).push(listener);
	}

	removeEventListener(event: string, listener: () => void) {
		this.listeners[event] = (this.listeners[event] ?? []).filter((l) => l !== listener);
	}

	emit(event: string) {
		(this.listeners[event] ?? []).forEach((listener) => listener());
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
		position_seconds: 0,
		listened_at: null,
		created_at: now,
		updated_at: now
	};
}

describe('player store queue', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		localStorage.clear();
		vi.clearAllMocks();
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
			JSON.stringify({
				upNext: [episode(7)],
				playStack: [episode(3)],
				currentEpisode: episode(5)
			})
		);
		const player = usePlayerStore();
		expect(player.upNext.map((e) => e.id)).toEqual([7]);
		expect(player.playStack.map((e) => e.id)).toEqual([3]);
		expect(player.currentEpisode?.id).toBe(5);
	});

	it('persists queue changes to localStorage', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);

		const stored = JSON.parse(localStorage.getItem('u2vpodcast.up-next.v1') ?? '{}');
		expect(stored.upNext.map((e: Episode) => e.id)).toEqual([2]);
		expect(stored.currentEpisode?.id).toBe(1);
	});

	it('togglePlay loads the source of a restored episode after a reload', async () => {
		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({
				upNext: [],
				playStack: [],
				currentEpisode: episode(1)
			})
		);
		const player = usePlayerStore();
		// the episode is restored but the shared element has never been loaded
		expect(player.currentEpisode?.id).toBe(1);
		expect(player.stopped).toBe(true);

		await player.togglePlay();
		const el = MockAudioElement.instances[0];
		expect(el.src).toBe('/media/c/yt1.mp3');
		expect(el.load).toHaveBeenCalled();
		expect(el.play).toHaveBeenCalled();
		expect(player.stopped).toBe(false);
	});

	it('seeds the queue with the playlist tail and sets the source to playlist', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		// playing a middle episode schedules only the remaining tail
		await player.play(episodes[1], episodes, { queueSource: 'playlist' });
		expect(player.queueSource).toBe('playlist');
		expect(player.upNext.map((e) => e.id)).toEqual([3]);
	});

	it('defaults the queue source to list for plain plays', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);
		expect(player.queueSource).toBe('list');
	});

	it('removes a completed playlist-sourced episode from the playlist', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes, { queueSource: 'playlist' });
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;
		vi.mocked(api.removeEpisodeFromPlaylist).mockClear();
		el.emit('ended');
		expect(api.removeEpisodeFromPlaylist).toHaveBeenCalledWith(1);
	});

	it('leaves a list-sourced completed episode in the playlist untouched', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;
		vi.mocked(api.removeEpisodeFromPlaylist).mockClear();
		el.emit('ended');
		expect(api.removeEpisodeFromPlaylist).not.toHaveBeenCalled();
	});

	it('long-press skip on a playlist episode removes it and marks it listened', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes, { queueSource: 'playlist' });
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		vi.mocked(api.removeEpisodeFromPlaylist).mockClear();
		await player.skipNext(true);
		expect(player.playStack[0].listen).toBe(true);
		expect(api.removeEpisodeFromPlaylist).toHaveBeenCalledWith(1);
	});
});

describe('player store playback progress', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		MockAudioElement.defaultClamp = 0;
		localStorage.clear();
		vi.clearAllMocks();
		setActivePinia(createPinia());
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('saves the current position on pause', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.currentTime = 42;

		await player.pause();
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 42,
			listened: false
		});
	});

	it('saves position at most once every 10 seconds', async () => {
		vi.useFakeTimers();
		const player = usePlayerStore();
		await player.play(episode(1));
		const el = MockAudioElement.instances[0];

		el.currentTime = 11;
		el.emit('timeupdate');
		expect(api.updateEpisodeProgress).toHaveBeenCalledTimes(1);

		el.currentTime = 21;
		el.emit('timeupdate');
		expect(api.updateEpisodeProgress).toHaveBeenCalledTimes(1);

		vi.advanceTimersByTime(10_000);
		el.emit('timeupdate');
		expect(api.updateEpisodeProgress).toHaveBeenCalledTimes(2);
		expect(api.updateEpisodeProgress).toHaveBeenLastCalledWith('yt1', {
			position_seconds: 21,
			listened: false
		});
	});

	it('resumes from the stored position between 30s and 95% of the duration', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		ep.position_seconds = 120;
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(120);
	});

	it('fetches the authoritative progress from the server before playing', async () => {
		vi.mocked(api.getEpisodeProgress).mockResolvedValueOnce({
			ok: true,
			data: { id: 1, yt_id: 'yt1', position_seconds: 300, listen: false, listened_at: null },
			user: null,
			status: true
		} as never);
		const player = usePlayerStore();
		// the local copy is stale (never played this session)
		const ep = episode(1);
		await player.play(ep);

		expect(api.getEpisodeProgress).toHaveBeenCalledWith('yt1');
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(300);
		expect(player.currentEpisode?.position_seconds).toBe(300);
	});

	it('uses the seeded list progress without a per-play request', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		player.seedProgress([{ ...ep, position_seconds: 300 }]);

		await player.play(ep);
		expect(api.getEpisodeProgress).not.toHaveBeenCalled();
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(300);
	});

	it('retries the resume seek while the browser clamps it', async () => {
		MockAudioElement.defaultClamp = 100;
		try {
			const player = usePlayerStore();
			const ep = episode(1);
			ep.position_seconds = 300;
			await player.play(ep);
			const el = MockAudioElement.instances[0];
			// the browser only allows seeking within its buffered prefix
			expect(el.currentTime).toBe(100);

			// the buffer grows until it covers the target, unclamping seeks
			el.seekClampEnd = 600;
			el.emit('timeupdate');
			expect(el.currentTime).toBe(300);
			expect(player.currentEpisode?.position_seconds).toBe(300);
		} finally {
			MockAudioElement.defaultClamp = 0;
		}
	});

	it('starts from zero when the server progress is not meaningful', async () => {
		vi.mocked(api.getEpisodeProgress).mockResolvedValueOnce({
			ok: true,
			data: { id: 1, yt_id: 'yt1', position_seconds: 20, listen: false, listened_at: null },
			user: null,
			status: true
		} as never);
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(0);
	});

	it('starts from zero when the stored position is not worth resuming', async () => {
		const player = usePlayerStore();
		const nearStart = episode(1);
		nearStart.position_seconds = 20;
		await player.play(nearStart);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(0);
	});

	it('does not resume positions at or beyond 95% of the duration', async () => {
		const player = usePlayerStore();
		const nearEnd = episode(1);
		nearEnd.position_seconds = 580;
		await player.play(nearEnd);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(0);
	});

	it('fromStart clears the stored position and skips resume', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		ep.position_seconds = 300;
		await player.play(ep, undefined, { fromStart: true });
		const el = MockAudioElement.instances[0];

		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
		expect(player.currentEpisode?.position_seconds).toBe(0);

		el.duration = 600;
		el.emit('loadedmetadata');
		expect(el.currentTime).toBe(0);
	});

	it('advance resumes the next queued episode from its saved position', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		b.position_seconds = 180;
		await player.play(a, [a, b]);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		await player.advance();
		expect(player.currentEpisode?.id).toBe(2);
		expect(el.currentTime).toBe(180);
	});

	it('skipNext resumes the next queued episode from its saved position', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		b.position_seconds = 200;
		await player.play(a, [a, b]);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		await player.skipNext(false);
		expect(player.currentEpisode?.id).toBe(2);
		expect(el.currentTime).toBe(200);
	});

	it('playPrevious resumes the history episode from its saved position', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		await player.play(a, [a, b]);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 240;
		el.emit('timeupdate'); // record a's position at 240

		// advancing pushes a onto the history and plays b
		await player.skipNext(false);
		expect(player.currentEpisode?.id).toBe(2);

		await player.playPrevious();
		expect(player.currentEpisode?.id).toBe(1);
		expect(el.currentTime).toBe(240);
	});

	it('marks the episode listened and stores its duration on completion', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;

		el.emit('ended');
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 600,
			listened: true
		});
		expect(player.currentEpisode?.listen).toBe(true);
		expect(player.currentEpisode?.listened_at).not.toBeNull();
		expect(player.currentEpisode?.position_seconds).toBe(600);
	});

	it('resumes a stopped episode from its last saved position', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		// stream to 420s: the throttled save keeps the in-memory episode in sync
		el.currentTime = 420;
		el.emit('timeupdate');
		expect(player.currentEpisode?.position_seconds).toBe(420);

		player.stop();
		expect(el.currentTime).toBe(0);

		await player.togglePlay();
		expect(el.currentTime).toBe(420);
	});

	it('togglePlay resumes a restored episode from the server progress', async () => {
		// a stale copy restored from the persisted queue (position 0 locally)
		const stale = episode(9);
		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({ upNext: [], playStack: [], currentEpisode: stale })
		);
		vi.mocked(api.getEpisodeProgress).mockResolvedValueOnce({
			ok: true,
			data: { id: 9, yt_id: 'yt9', position_seconds: 300, listen: false, listened_at: null },
			user: null,
			status: true
		} as never);

		const player = usePlayerStore();
		await player.togglePlay();
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.emit('loadedmetadata');

		expect(api.getEpisodeProgress).toHaveBeenCalledWith('yt9');
		expect(el.currentTime).toBe(300);
	});

	it('resumes the same episode through play() once the metadata is loaded', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.currentTime = 200;
		el.emit('timeupdate');
		player.stop();

		// replaying the same episode re-uses the loaded element: no metadata
		// event, but the resume applies immediately from the in-memory position
		await player.play(ep);
		expect(el.currentTime).toBe(200);
	});

	it('start over resets the playhead of the already-loaded current episode', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 120;
		el.emit('timeupdate');

		await player.play(ep, undefined, { fromStart: true });
		expect(el.currentTime).toBe(0);
		expect(player.currentEpisode?.position_seconds).toBe(0);
	});

	it('skipNext with markCurrent keeps the duration position despite the flush', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		await player.play(a, [a, b]);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 300;
		el.emit('timeupdate');
		vi.mocked(api.updateEpisodeProgress).mockClear();

		await player.skipNext(true);
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.playStack[0].listen).toBe(true);
		expect(player.playStack[0].position_seconds).toBe(600);
		const aCalls = vi.mocked(api.updateEpisodeProgress).mock.calls.filter(([yt]) => yt === 'yt1');
		expect(aCalls.at(-1)![1]).toEqual({ position_seconds: 600, listened: true });
	});

	it('does not keep saving the position while the audio is stopped', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 50;
		el.emit('timeupdate');
		vi.mocked(api.updateEpisodeProgress).mockClear();

		player.stop();
		// late timeupdate events after a stop must not persist anything
		el.currentTime = 15;
		el.emit('timeupdate');
		el.emit('timeupdate');
		expect(api.updateEpisodeProgress).not.toHaveBeenCalled();
	});

	it('does not resume a stopped episode without a meaningful position', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.currentTime = 15;
		el.emit('timeupdate');
		expect(player.currentEpisode?.position_seconds).toBe(15);

		player.stop();
		await player.togglePlay();
		expect(el.currentTime).toBe(0);
	});

	it('flushes the departing episode position when switching to another episode', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		await player.play(a);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.currentTime = 300;
		el.emit('timeupdate');
		expect(player.currentEpisode?.position_seconds).toBe(300);
		vi.mocked(api.updateEpisodeProgress).mockClear();

		// move the playhead past the last throttled save, then switch sources
		el.currentTime = 342;
		await player.play(b);
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 342,
			listened: false
		});
	});

	it('stops playing keeps the final position: switching after stop does not overwrite it', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		await player.play(a);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.currentTime = 400;
		el.emit('timeupdate');
		player.stop();
		vi.mocked(api.updateEpisodeProgress).mockClear();

		await player.play(b);
		expect(api.updateEpisodeProgress).not.toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
	});

	it('does not overwrite the stored resume position with a premature save', async () => {
		vi.useFakeTimers();
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		b.position_seconds = 300;
		await player.play(a);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.currentTime = 400;
		el.emit('timeupdate');
		await player.pause();
		vi.advanceTimersByTime(15_000);
		vi.mocked(api.updateEpisodeProgress).mockClear();

		// switching to B arms a resume at 300s; its early `timeupdate` fires
		// before `loadedmetadata` arrives and must not clobber the stored value
		await player.play(b);
		el.duration = 0;
		el.currentTime = 0.5;
		el.emit('timeupdate');

		expect(api.updateEpisodeProgress).not.toHaveBeenCalledWith('yt2', {
			position_seconds: 0.5,
			listened: false
		});
	});

	it('persists the resume target when pausing inside the retry window', async () => {
		MockAudioElement.defaultClamp = 100;
		try {
			const player = usePlayerStore();
			const ep = episode(1);
			ep.position_seconds = 300;
			await player.play(ep);
			const el = MockAudioElement.instances[0];
			el.duration = 600;
			// the playhead is still clamped below the target (resume pending)
			el.currentTime = 32;
			el.emit('timeupdate');

			await player.pause();
			expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
				position_seconds: 300,
				listened: false
			});
		} finally {
			MockAudioElement.defaultClamp = 0;
		}
	});

	it('marks a completed episode with the parsed duration when media duration is unavailable', async () => {
		const player = usePlayerStore();
		const ep = episode(1); // duration '00:10:00'
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 0; // media never exposed a usable duration

		el.emit('ended');
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 600,
			listened: true
		});
		expect(player.currentEpisode?.position_seconds).toBe(600);
	});

	it('stop sends the final position once and never a trailing zero', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		el.currentTime = 330;
		el.emit('timeupdate'); // last throttled save
		vi.mocked(api.updateEpisodeProgress).mockClear();

		el.currentTime = 342;
		player.stop();
		// the async `pause` event delivered after the reset must write nothing
		el.currentTime = 0;
		el.emit('pause');

		expect(api.updateEpisodeProgress).toHaveBeenCalledTimes(1);
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 342,
			listened: false
		});
	});

	it('associates progress by episode id across stale copies', async () => {
		const player = usePlayerStore();
		const live = episode(1);
		await player.play(live);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 300;
		el.emit('timeupdate');
		player.stop();

		// a stale copy (never refreshed) still resumes from the recorded value
		const stale = episode(1);
		expect(stale.position_seconds).toBe(0);
		await player.play(stale);
		expect(player.currentEpisode?.position_seconds).toBe(300);
		expect(MockAudioElement.instances[0].currentTime).toBe(300);
	});

	it('episodeWithProgress returns the recorded progress for any copy keyed by id', async () => {
		const player = usePlayerStore();
		const live = episode(1);
		await player.play(live);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 120;
		el.emit('timeupdate');
		player.stop();

		const staleCopy = episode(1);
		const resolved = player.episodeWithProgress(staleCopy);
		expect(resolved.position_seconds).toBe(120);
		expect(resolved.listen).toBe(false);
	});

	it('a second stop press resets the saved position to zero', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 240;
		el.emit('timeupdate'); // recorded at 240

		player.stop(); // first press: stop playback, keep the position
		expect(player.stopped).toBe(true);
		expect(player.currentEpisode?.position_seconds).toBe(240);
		vi.mocked(api.updateEpisodeProgress).mockClear();

		player.stop(); // second press while already stopped: reset to 0
		expect(player.currentEpisode?.position_seconds).toBe(0);
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
	});

	it('a second stop press resets a listened episode to zero keeping the mark', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;
		el.emit('ended'); // completion marks listened (600,true), then stops
		expect(player.stopped).toBe(true);

		vi.mocked(api.updateEpisodeProgress).mockClear();
		player.stop(); // already stopped → reset to 0, mark untouched
		expect(player.currentEpisode?.position_seconds).toBe(0);
		expect(player.currentEpisode?.listen).toBe(true);
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: true
		});
	});

	it('stop does not reset when the stopped episode is already at the start', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		player.stop(); // first press
		expect(player.stopped).toBe(true);

		vi.mocked(api.updateEpisodeProgress).mockClear();
		player.stop(); // already stopped and at 0 → no write
		expect(api.updateEpisodeProgress).not.toHaveBeenCalled();
	});

	it('stop on a non-current stopped episode resets its saved position', async () => {
		const player = usePlayerStore();
		const a = episode(1);
		const b = episode(2);
		await player.play(a, [a, b]);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 240;
		el.emit('timeupdate'); // record a at 240
		await player.skipNext(false); // b becomes the current episode
		el.currentTime = 120;
		await player.pause(); // record b at 120
		vi.mocked(api.updateEpisodeProgress).mockClear();

		// a is stopped but not the current episode: stop resets just a
		player.stop(a);
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.episodeWithProgress(episode(1)).position_seconds).toBe(0);
		const resetCalls = vi
			.mocked(api.updateEpisodeProgress)
			.mock.calls.filter(([yt]) => yt === 'yt1');
		expect(resetCalls.at(-1)![1]).toEqual({ position_seconds: 0, listened: false });
	});

	it('stopping a paused episode resets its saved position to zero', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 240;
		el.emit('timeupdate'); // recorded at 240

		await player.pause(); // paused, not stopped
		player.stop(); // not reproducing → reset to 0
		expect(player.currentEpisode?.position_seconds).toBe(0);
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
	});

	it('a mid-replay stop of a listened episode saves the live position', async () => {
		const player = usePlayerStore();
		const ep = episode(1, true);
		ep.position_seconds = 300;
		await player.play(ep); // listened episode resumes to 300
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 299;
		el.emit('timeupdate'); // resume lands at target
		vi.mocked(api.updateEpisodeProgress).mockClear();

		el.currentTime = 240; // actively re-listened below the saved point
		player.stop(); // halt while reproducing — the live position must persist
		expect(player.stopped).toBe(true);
		expect(player.currentEpisode?.position_seconds).toBe(240);
		expect(player.currentEpisode?.listen).toBe(true);
		expect(
			vi
				.mocked(api.updateEpisodeProgress)
				.mock.calls.some(
					([yt, body]) => yt === 'yt1' && body.position_seconds === 240 && body.listened === true
				)
		).toBe(true);
	});

	it('does not regress the listened mark with a trailing pause after completion', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		vi.mocked(api.updateEpisodeProgress).mockClear();
		el.currentTime = 600;
		el.emit('ended'); // completion -> markListened {600,true}, then stop()
		el.currentTime = 0;
		el.emit('pause'); // trailing async pause after the stop reset

		// every write keeps the mark listened and never uses the zero playhead
		const calls = vi.mocked(api.updateEpisodeProgress).mock.calls;
		expect(calls.length).toBeGreaterThan(0);
		expect(calls.every(([, body]) => body.listened === true)).toBe(true);
		expect(calls.every(([, body]) => body.position_seconds === 600)).toBe(true);
	});
});

describe('player store keyboard seek', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		MockAudioElement.defaultClamp = 0;
		localStorage.clear();
		vi.clearAllMocks();
		vi.spyOn(document, 'hasFocus').mockReturnValue(true);
		setActivePinia(createPinia());
	});

	afterEach(() => {
		vi.restoreAllMocks();
	});

	async function loadedEpisode(): Promise<MockAudioElement> {
		const player = usePlayerStore();
		await player.play(episode(1));
		const el = MockAudioElement.instances[0];
		el.duration = 100;
		return el;
	}

	it('ArrowRight seeks 15s forward clamped to the duration', async () => {
		const el = await loadedEpisode();
		el.currentTime = 50;
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
		expect(el.currentTime).toBe(65);

		el.currentTime = 95;
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
		expect(el.currentTime).toBe(100);
	});

	it('ArrowLeft seeks 15s backward clamped to zero', async () => {
		const el = await loadedEpisode();
		el.currentTime = 50;
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft' }));
		expect(el.currentTime).toBe(35);

		el.currentTime = 5;
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowLeft' }));
		expect(el.currentTime).toBe(0);
	});

	it('does nothing when no episode is loaded', () => {
		usePlayerStore();
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
		expect(MockAudioElement.instances).toHaveLength(0);
	});

	it('does nothing when the document is not focused', async () => {
		vi.mocked(document.hasFocus).mockReturnValue(false);
		const el = await loadedEpisode();
		el.currentTime = 50;
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }));
		expect(el.currentTime).toBe(50);
	});

	it('does not seek when the focus is inside an input', async () => {
		const el = await loadedEpisode();
		el.currentTime = 50;
		const input = document.createElement('input');
		input.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }));
		expect(el.currentTime).toBe(50);
	});

	it('does not seek when the focus is inside a slider', async () => {
		const el = await loadedEpisode();
		el.currentTime = 50;
		const slider = document.createElement('div');
		slider.setAttribute('role', 'slider');
		slider.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true }));
		expect(el.currentTime).toBe(50);
	});
});

describe('player store playback modes', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		MockAudioElement.defaultClamp = 0;
		localStorage.clear();
		vi.clearAllMocks();
		setActivePinia(createPinia());
		// Mulberry32 with a fixed seed: every shuffle is deterministic, so the
		// mode tests can assert exact permutations and per-cycle re-shuffles
		// (playback-modes).
		let state = 0x9e3779b9;
		setRandomSource(() => {
			state += 0x6d2b79f5;
			let t = state;
			t = Math.imul(t ^ (t >>> 15), t | 1);
			t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
			return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
		});
	});

	afterEach(() => {
		setRandomSource(Math.random);
	});

	it('cycleRepeat cycles none → all → one → none', () => {
		const player = usePlayerStore();
		expect(player.repeat).toBe('none');
		expect(player.cycleRepeat()).toBe('all');
		expect(player.cycleRepeat()).toBe('one');
		expect(player.cycleRepeat()).toBe('none');
	});

	it('shuffle permutes the queue without losing or duplicating items', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4), episode(5)];
		await player.play(episodes[0], episodes);
		expect(player.upNext.map((e) => e.id)).toEqual([2, 3, 4, 5]);

		player.toggleShuffle();
		expect(player.shuffle).toBe(true);
		const shuffled = player.upNext.map((e) => e.id);
		expect(shuffled).toHaveLength(4);
		// exactly the queued set: no duplication and nothing left behind
		expect(new Set(shuffled).size).toBe(4);
		expect([...shuffled].sort()).toEqual([2, 3, 4, 5]);
	});

	it('disabling shuffle restores the authored order of the remaining episodes', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4)];
		await player.play(episodes[0], episodes);
		player.toggleShuffle();
		const shuffledFirst = player.upNext.map((e) => e.id);

		// consume one episode in the shuffled order
		await player.advance();
		expect(player.currentEpisode?.id).toBe(shuffledFirst[0]);

		player.toggleShuffle();
		expect(player.shuffle).toBe(false);
		// the remaining episodes are back in their authored order
		expect(player.upNext.map((e) => e.id)).toEqual(
			[2, 3, 4].filter((id) => id !== shuffledFirst[0])
		);
	});

	it('repeat-one replays the finished episode from the start without consuming the queue', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);
		player.cycleRepeat(); // → all
		player.cycleRepeat(); // → one
		expect(player.repeat).toBe('one');

		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;
		el.emit('ended');

		expect(player.currentEpisode?.id).toBe(1);
		expect(player.upNext.map((e) => e.id)).toEqual([2]);
		expect(player.playStack.map((e) => e.id)).toEqual([]);
		expect(el.currentTime).toBe(0);
		expect(player.playing).toBe(true);
	});

	it('repeat-all rebuilds and re-shuffles the queue from the seed after the last item', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4)];
		await player.play(episodes[0], episodes);
		player.toggleShuffle();
		player.cycleRepeat(); // → all
		const firstPass = player.upNext.map((e) => e.id);
		expect(firstPass).toHaveLength(3);

		// drain the initial shuffled queue; the fourth advance hits exhaustion
		for (let i = 0; i < 4; i++) await player.advance();

		// the queue was rebuilt from the seed (re-shuffled) and playback continued
		expect(player.currentEpisode).not.toBeNull();
		const rebuilt = [...player.upNext.map((e) => e.id), player.currentEpisode!.id];
		expect([...rebuilt].sort()).toEqual([2, 3, 4]);
		// a fresh randomization per cycle: the rebuilt order differs
		expect(player.upNext.map((e) => e.id)).not.toEqual(firstPass);
	});

	it('repeat-none stops and clears the queue at the end', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2)];
		await player.play(episodes[0], episodes);
		expect(player.repeat).toBe('none');

		await player.advance(); // consumes episode 2
		await player.advance(); // queue empty → stop and clear
		expect(player.playing).toBe(false);
		expect(player.stopped).toBe(true);
		expect(player.currentTime).toBe(0);
		expect(player.upNext).toEqual([]);
	});

	it('persists shuffle and repeat modes and restores them on a reload', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);
		player.toggleShuffle();
		player.cycleRepeat(); // → all

		const stored = JSON.parse(localStorage.getItem('u2vpodcast.up-next.v1') ?? '{}');
		expect(stored.shuffle).toBe(true);
		expect(stored.repeat).toBe('all');

		// a reload boots a fresh store from the same payload (queue.storage)
		const reloaded = usePlayerStore(createPinia());
		expect(reloaded.shuffle).toBe(true);
		expect(reloaded.repeat).toBe('all');
	});

	it('restores the queue in its shuffled order after a reload', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4)];
		await player.play(episodes[0], episodes);
		player.toggleShuffle();
		const shuffled = player.upNext.map((e) => e.id);

		const reloaded = usePlayerStore(createPinia());
		expect(reloaded.upNext.map((e) => e.id)).toEqual(shuffled);
	});

	it('loads a legacy payload without modes as disabled defaults', () => {
		localStorage.setItem(
			'u2vpodcast.up-next.v1',
			JSON.stringify({
				upNext: [episode(7)],
				playStack: [episode(3)],
				currentEpisode: episode(5)
			})
		);
		const player = usePlayerStore();
		expect(player.shuffle).toBe(false);
		expect(player.repeat).toBe('none');
	});
});
