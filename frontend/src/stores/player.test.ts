import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import {
	chapterTimelineMarkers,
	sponsorBlockSkipTarget,
	sponsorBlockTimelineMarkers,
	usePlayerStore,
	setRandomSource
} from '@/stores/player';
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
		),
		setChannelPlaybackSpeed: vi.fn(() =>
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
	// A real media element keeps the rate a load() should reset back to.
	defaultPlaybackRate = 1;
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
		// a real media element resets the playhead, duration and playback
		// rate (to defaultPlaybackRate) on load() and reports its metadata
		// once parsed
		this.currentTime = 0;
		this.duration = 0;
		this.playbackRate = this.defaultPlaybackRate;
		this.emit('loadedmetadata');
	});

	removeAttribute(name: string) {
		if (name === 'src') this.src = '';
	}

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

type MockMediaActionHandler = (details: MediaSessionActionDetails) => void;

class MockMediaMetadata {
	static rejectArtwork = false;

	title = '';
	artist = '';
	artwork: readonly MediaImage[] = [];

	constructor(init: MediaMetadataInit = {}) {
		if (MockMediaMetadata.rejectArtwork && init.artwork?.length)
			throw new Error('artwork rejected');
		this.title = init.title ?? '';
		this.artist = init.artist ?? '';
		this.artwork = init.artwork ?? [];
	}
}

class MockMediaSession {
	handlers = new Map<MediaSessionAction, MockMediaActionHandler | null>();
	registrationAttempts = new Map<MediaSessionAction, number>();
	rejectedActions = new Set<MediaSessionAction>();
	metadata: MockMediaMetadata | null = null;
	playbackState: MediaSessionPlaybackState = 'none';
	positionStates: MediaPositionState[] = [];

	setActionHandler(action: MediaSessionAction, handler: MockMediaActionHandler | null) {
		this.registrationAttempts.set(action, (this.registrationAttempts.get(action) ?? 0) + 1);
		if (this.rejectedActions.has(action)) throw new Error(`${action} unsupported`);
		this.handlers.set(action, handler);
	}

	setPositionState(state?: MediaPositionState) {
		if (state) this.positionStates.push({ ...state });
	}

	invoke(action: MediaSessionAction, details: Partial<MediaSessionActionDetails> = {}) {
		this.handlers.get(action)?.({ action, ...details } as MediaSessionActionDetails);
	}
}

function installMediaSession(session: MockMediaSession) {
	Object.defineProperty(navigator, 'mediaSession', {
		configurable: true,
		value: session
	});
	vi.stubGlobal('MediaMetadata', MockMediaMetadata as unknown as typeof MediaMetadata);
}

function removeMediaSession() {
	Reflect.deleteProperty(navigator, 'mediaSession');
	Reflect.deleteProperty(globalThis, 'MediaMetadata');
}

function episode(id: number, listen = false): Episode {
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
		listen,
		position_seconds: 0,
		listened_at: null,
		favorite: false,
		chapters: [],
		sponsorblock_enabled: true,
		sponsorblock_segments: [],
		sponsorblock_hash: null,
		created_at: now,
		updated_at: now
	};
}

describe('SponsorBlock playback', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		localStorage.clear();
		vi.clearAllMocks();
		setActivePinia(createPinia());
	});

	it('skips only the complete union of rejected overlapping intervals', () => {
		const segments = [
			{ start: 120, end: 150, category: 'sponsor', rejected: true },
			{ start: 145, end: 170, category: 'intro', rejected: true },
			{ start: 125, end: 180, category: 'selfpromo', rejected: false }
		];
		expect(sponsorBlockSkipTarget(119.9, segments)).toBe(119.9);
		expect(sponsorBlockSkipTarget(120, segments)).toBe(170);
		expect(sponsorBlockSkipTarget(160, segments)).toBe(170);
		expect(sponsorBlockSkipTarget(170, segments)).toBe(170);
		expect(sponsorBlockSkipTarget(175, segments)).toBe(175);
		expect(sponsorBlockSkipTarget(125, [segments[2]])).toBe(125);
		expect(sponsorBlockSkipTarget(125, [])).toBe(125);
	});

	it('maps clamped segments onto the original timeline', () => {
		expect(
			sponsorBlockTimelineMarkers(200, [
				{ start: -10, end: 20, category: 'sponsor', rejected: true },
				{ start: 60, end: 120, category: 'intro', rejected: false },
				{ start: 190, end: 220, category: 'outro', rejected: true },
				{ start: 150, end: 140, category: 'sponsor', rejected: true }
			])
		).toEqual([
			{ left: 0, width: 10, category: 'sponsor' },
			{ left: 30, width: 30, category: 'intro' },
			{ left: 95, width: 5, category: 'outro' }
		]);
		expect(
			sponsorBlockTimelineMarkers(0, [{ start: 1, end: 2, category: 'sponsor', rejected: true }])
		).toEqual([]);
	});

	it('maps valid chapter starts onto the original timeline', () => {
		expect(chapterTimelineMarkers(600, undefined)).toEqual([]);
		expect(chapterTimelineMarkers(600, [])).toEqual([]);
		expect(
			chapterTimelineMarkers(600, [
				{ start: 0, end: 60, title: 'Introduction' },
				{ start: 150, end: 300, title: 'Main topic' },
				{ start: 700, end: 800, title: 'Beyond duration' }
			])
		).toEqual([
			{ left: 0, title: 'Introduction', startSeconds: 0 },
			{ left: 25, title: 'Main topic', startSeconds: 150 }
		]);
		expect(chapterTimelineMarkers(0, [{ start: 0, end: 10, title: 'Intro' }])).toEqual([]);
	});

	it('skips on timeupdate and explicit seek using the original timeline', async () => {
		const player = usePlayerStore();
		const item = episode(1);
		item.sponsorblock_segments = [{ start: 120, end: 150, category: 'sponsor', rejected: true }];
		item.sponsorblock_hash = 'hash-a';
		await player.play(item);
		const audio = MockAudioElement.instances[0];
		audio.currentTime = 125;
		audio.emit('timeupdate');
		expect(audio.currentTime).toBe(150);
		expect(player.currentTime).toBe(150);

		player.seek(130);
		expect(audio.currentTime).toBe(150);
		expect(player.currentTime).toBe(150);
	});

	it('uses rejected intervals for resume and relative seeks but bypasses them when disabled', async () => {
		const player = usePlayerStore();
		const item = episode(1);
		item.position_seconds = 125;
		item.sponsorblock_segments = [
			{ start: 120, end: 150, category: 'sponsor', rejected: true },
			{ start: 100, end: 180, category: 'intro', rejected: false }
		];
		await player.play(item);
		const audio = MockAudioElement.instances[0];
		expect(audio.currentTime).toBe(150);

		audio.duration = 600;
		audio.currentTime = 110;
		player.seekRelative(15);
		expect(audio.currentTime).toBe(150);

		player.applySponsorBlockSnapshot({ ...item, sponsorblock_enabled: false });
		audio.currentTime = 125;
		audio.emit('timeupdate');
		expect(audio.currentTime).toBe(125);
	});

	it('applies visible, rejection, and disabled snapshots without reloading the source', async () => {
		const player = usePlayerStore();
		const item = episode(1);
		item.sponsorblock_hash = 'hash-a';
		item.sponsorblock_segments = [{ start: 10, end: 20, category: 'intro', rejected: false }];
		await player.play(item);
		const audio = MockAudioElement.instances[0];
		const source = audio.src;
		const loadCalls = audio.load.mock.calls.length;

		player.applySponsorBlockSnapshot({
			...item,
			sponsorblock_hash: 'hash-b',
			sponsorblock_segments: [{ start: 10, end: 25, category: 'intro', rejected: false }]
		});
		expect(player.currentEpisode?.sponsorblock_segments?.[0].end).toBe(25);

		player.applySponsorBlockSnapshot({
			...item,
			sponsorblock_hash: 'hash-b',
			sponsorblock_segments: [{ start: 10, end: 25, category: 'intro', rejected: true }]
		});
		expect(player.currentEpisode?.sponsorblock_segments?.[0].rejected).toBe(true);

		player.applySponsorBlockSnapshot({ ...item, sponsorblock_enabled: false });
		expect(player.currentEpisode?.sponsorblock_enabled).toBe(false);
		expect(player.currentEpisode?.sponsorblock_segments).toEqual([]);
		expect(player.currentEpisode?.sponsorblock_hash).toBeNull();
		expect(audio.src).toBe(source);
		expect(audio.load).toHaveBeenCalledTimes(loadCalls);
	});
});

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

	it('replaces an existing queue when playing from a new list', async () => {
		const player = usePlayerStore();
		const firstList = [episode(1), episode(2), episode(3)];
		const nextList = [episode(9), episode(10), episode(11)];
		await player.play(firstList[0], firstList);

		await player.play(nextList[0], nextList);

		expect(player.upNext.map((e) => e.id)).toEqual([10, 11]);
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

	it('synchronizes playlist up next and advances in the reordered tail', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4)];
		await player.play(episodes[1], episodes, { queueSource: 'playlist' });

		player.syncPlaylistOrder([episodes[1], episodes[3], episodes[0], episodes[2]]);

		expect(player.upNext.map((item) => item.id)).toEqual([4, 1, 3]);
		await player.advance();
		expect(player.currentEpisode?.id).toBe(4);
	});

	it('does not replace a queue seeded from another list', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3)];
		await player.play(episodes[0], episodes);

		player.syncPlaylistOrder([episodes[0], episodes[2], episodes[1]]);

		expect(player.upNext.map((item) => item.id)).toEqual([2, 3]);
	});

	it('restores the reordered playlist tail when shuffle is disabled', async () => {
		const player = usePlayerStore();
		const episodes = [episode(1), episode(2), episode(3), episode(4)];
		await player.play(episodes[0], episodes, { queueSource: 'playlist' });
		player.toggleShuffle();

		player.syncPlaylistOrder([episodes[0], episodes[3], episodes[2], episodes[1]]);
		player.toggleShuffle();

		expect(player.upNext.map((item) => item.id)).toEqual([4, 3, 2]);
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

	it('stop on a paused episode keeps the saved position', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;

		// stream to 420s, then pause so the episode is NOT reproducing
		el.currentTime = 420;
		el.emit('timeupdate');
		await player.togglePlay();
		expect(player.currentEpisode?.position_seconds).toBe(420);

		vi.mocked(api.updateEpisodeProgress).mockClear();
		player.stop();
		// stop converges to the stopped state but never writes a 0 position
		expect(player.currentEpisode?.position_seconds).toBe(420);
		expect(api.updateEpisodeProgress).not.toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
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

	it('a second stop press keeps the saved position', async () => {
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

		player.stop(); // second press while already stopped: still keeps it
		expect(player.currentEpisode?.position_seconds).toBe(240);
		expect(api.updateEpisodeProgress).not.toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
	});

	it('a second stop press keeps the position and the mark on a listened episode', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;
		el.emit('ended'); // completion marks listened (600,true), then stops
		expect(player.stopped).toBe(true);

		vi.mocked(api.updateEpisodeProgress).mockClear();
		player.stop(); // persistent-bar stop on the stopped episode → nothing changes
		expect(player.currentEpisode?.position_seconds).toBe(600);
		expect(player.currentEpisode?.listen).toBe(true);
		expect(api.updateEpisodeProgress).not.toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: true
		});
	});

	it('card stop on a paused current episode resets its saved position', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 240;
		el.emit('timeupdate'); // recorded at 240

		await player.pause(); // paused, not stopped
		player.stop(ep); // card stop while not reproducing → reset to 0
		expect(player.currentEpisode?.position_seconds).toBe(0);
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
	});

	it('card stop on a listened non-reproducing episode resets to zero keeping the mark', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 600;
		el.emit('ended'); // completion marks listened (600,true), then stops
		expect(player.stopped).toBe(true);

		vi.mocked(api.updateEpisodeProgress).mockClear();
		player.stop(ep); // card stop on the stopped current episode → reset, mark kept
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

	it('card stop on a non-current stopped episode resets its saved position', async () => {
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

		// a is stopped but not the current episode: the card's stop resets
		// just a (fix-stop-reset-scope); the current playback is untouched
		player.stop(a);
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.episodeWithProgress(episode(1)).position_seconds).toBe(0);
		const resetCalls = vi
			.mocked(api.updateEpisodeProgress)
			.mock.calls.filter(([yt]) => yt === 'yt1');
		expect(resetCalls.at(-1)![1]).toEqual({ position_seconds: 0, listened: false });
	});

	it('stopping a paused episode keeps its saved position', async () => {
		const player = usePlayerStore();
		const ep = episode(1);
		await player.play(ep);
		const el = MockAudioElement.instances[0];
		el.duration = 600;
		el.currentTime = 240;
		el.emit('timeupdate'); // recorded at 240

		await player.pause(); // paused, not stopped
		player.stop(); // not reproducing → converge to stopped, keep position
		expect(player.currentEpisode?.position_seconds).toBe(240);
		expect(api.updateEpisodeProgress).not.toHaveBeenCalledWith('yt1', {
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

	it('cycles the mobile combined mode normal -> repeat -> shuffle -> normal', () => {
		const player = usePlayerStore();
		expect(player.mobilePlaybackMode).toBe('normal');

		expect(player.cycleMobilePlaybackMode()).toBe('repeat');
		expect(player.shuffle).toBe(false);
		expect(player.repeat).toBe('all');
		expect(player.mobilePlaybackMode).toBe('repeat');

		expect(player.cycleMobilePlaybackMode()).toBe('shuffle');
		expect(player.shuffle).toBe(true);
		expect(player.repeat).toBe('none');
		expect(player.mobilePlaybackMode).toBe('shuffle');

		expect(player.cycleMobilePlaybackMode()).toBe('normal');
		expect(player.shuffle).toBe(false);
		expect(player.repeat).toBe('none');
		expect(player.mobilePlaybackMode).toBe('normal');
	});

	it('maps an unreachable combination to its closest mobile mode without changing it', () => {
		const player = usePlayerStore();
		player.cycleRepeat(); // -> all
		player.cycleRepeat(); // -> one
		expect(player.repeat).toBe('one');
		// repeat-one has no exact mobile representation; closest is 'repeat'
		expect(player.mobilePlaybackMode).toBe('repeat');
		expect(player.repeat).toBe('one');

		player.toggleShuffle();
		expect(player.shuffle).toBe(true);
		// shuffle combined with a repeat mode also has no exact representation;
		// closest is 'shuffle', and the underlying state stays untouched
		expect(player.mobilePlaybackMode).toBe('shuffle');
		expect(player.shuffle).toBe(true);
		expect(player.repeat).toBe('one');

		// interacting with the control now advances from that closest state
		expect(player.cycleMobilePlaybackMode()).toBe('normal');
		expect(player.shuffle).toBe(false);
		expect(player.repeat).toBe('none');
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

describe('player store system media controls', () => {
	let session: MockMediaSession;

	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		MockMediaMetadata.rejectArtwork = false;
		localStorage.clear();
		vi.clearAllMocks();
		setActivePinia(createPinia());
		session = new MockMediaSession();
		installMediaSession(session);
	});

	afterEach(() => {
		removeMediaSession();
		vi.unstubAllGlobals();
	});

	it('registers every action once and publishes episode metadata', async () => {
		const player = usePlayerStore();
		const item = episode(1);
		item.image = 'https://example.test/cover.jpg';

		await player.play(item);
		await player.play(item);

		expect([...session.handlers.keys()]).toEqual([
			'play',
			'pause',
			'nexttrack',
			'previoustrack',
			'seekforward',
			'seekbackward',
			'seekto'
		]);
		for (const attempts of session.registrationAttempts.values()) expect(attempts).toBe(1);
		expect(session.metadata).toMatchObject({
			title: 'Episode 1',
			artist: 'Channel',
			artwork: [{ src: 'https://example.test/cover.jpg' }]
		});
	});

	it('falls back safely when the API or individual features are unavailable', async () => {
		removeMediaSession();
		const withoutApi = usePlayerStore();
		await withoutApi.play(episode(1));
		expect(withoutApi.playing).toBe(true);

		setActivePinia(createPinia());
		session = new MockMediaSession();
		session.rejectedActions.add('seekto');
		Object.defineProperty(session, 'setPositionState', { configurable: true, value: undefined });
		installMediaSession(session);
		const partialApi = usePlayerStore();
		await partialApi.play(episode(2));
		const audio = MockAudioElement.instances.at(-1)!;
		audio.duration = 600;
		audio.emit('loadedmetadata');

		expect(partialApi.playing).toBe(true);
		expect(session.handlers.get('play')).toBeTypeOf('function');
		expect(session.handlers.get('nexttrack')).toBeTypeOf('function');
		expect(session.handlers.has('seekto')).toBe(false);
	});

	it('falls back to text metadata when artwork is rejected', async () => {
		MockMediaMetadata.rejectArtwork = true;
		const item = episode(1);
		item.image = 'invalid artwork';

		await usePlayerStore().play(item);

		expect(session.metadata).toMatchObject({ title: 'Episode 1', artist: 'Channel', artwork: [] });
	});

	it('routes system and native play-pause changes through shared state', async () => {
		const player = usePlayerStore();
		await player.play(episode(1));
		const audio = MockAudioElement.instances[0];
		vi.mocked(api.updateEpisodeProgress).mockClear();

		session.invoke('pause');
		expect(player.playing).toBe(false);
		expect(player.stopped).toBe(false);
		expect(session.playbackState).toBe('paused');
		expect(api.updateEpisodeProgress).toHaveBeenCalled();

		session.invoke('play');
		await vi.waitFor(() => expect(player.playing).toBe(true));
		expect(player.stopped).toBe(false);
		expect(session.playbackState).toBe('playing');

		player.stopped = true;
		audio.paused = false;
		audio.emit('play');
		expect(player.playing).toBe(true);
		expect(player.stopped).toBe(false);
	});

	it('uses queue and history semantics for system next and previous', async () => {
		const player = usePlayerStore();
		const items = [episode(1), episode(2)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];

		session.invoke('nexttrack');
		await vi.waitFor(() => expect(player.currentEpisode?.id).toBe(2));
		expect(items[0].listen).toBe(false);
		expect(player.upNext).toEqual([]);

		audio.duration = 600;
		audio.currentTime = 10;
		audio.emit('timeupdate');
		session.invoke('previoustrack');
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.currentTime).toBe(0);

		session.invoke('previoustrack');
		await vi.waitFor(() => expect(player.currentEpisode?.id).toBe(1));
		session.invoke('nexttrack');
		expect(player.currentEpisode?.id).toBe(1);
	});

	it('applies bounded original-timeline system seeks', async () => {
		const player = usePlayerStore();
		const item = episode(1);
		item.sponsorblock_segments = [{ start: 120, end: 150, category: 'sponsor', rejected: true }];
		await player.play(item);
		const audio = MockAudioElement.instances[0];
		audio.duration = 600;
		audio.emit('loadedmetadata');

		audio.currentTime = 100;
		audio.emit('timeupdate');
		session.invoke('seekforward', { seekOffset: 20 });
		expect(player.currentTime).toBe(150);

		audio.currentTime = 100;
		audio.emit('timeupdate');
		session.invoke('seekbackward');
		expect(player.currentTime).toBe(85);

		session.invoke('seekto', { seekTime: 125 });
		expect(player.currentTime).toBe(150);
		session.invoke('seekto', { seekTime: 900 });
		expect(player.currentTime).toBe(600);

		audio.duration = 0;
		audio.currentTime = 90;
		session.invoke('seekforward', { seekOffset: 30 });
		expect(audio.currentTime).toBe(90);
	});

	it('updates metadata, playback state, and validated position state', async () => {
		const player = usePlayerStore();
		const items = [episode(1), episode(2)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];
		expect(session.playbackState).toBe('playing');

		audio.duration = 600;
		audio.currentTime = 700;
		audio.emit('loadedmetadata');
		audio.emit('timeupdate');
		expect(session.positionStates.at(-1)).toEqual({
			duration: 600,
			position: 600,
			playbackRate: 1
		});

		player.setSpeed(1.5);
		expect(session.positionStates.at(-1)?.playbackRate).toBe(1.5);
		const updateCount = session.positionStates.length;
		audio.duration = Number.NaN;
		audio.emit('timeupdate');
		expect(session.positionStates).toHaveLength(updateCount);

		audio.duration = 600;
		session.invoke('nexttrack');
		await vi.waitFor(() => expect(player.currentEpisode?.id).toBe(2));
		expect(session.metadata?.title).toBe('Episode 2');
		session.invoke('pause');
		expect(session.playbackState).toBe('paused');
		player.stop();
		expect(session.playbackState).toBe('none');
	});

	it('tears down protected native media and can establish a fresh session', async () => {
		const player = usePlayerStore();
		const items = [episode(1), episode(2)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];
		const stalePlay = session.handlers.get('play')!;

		player.teardownNativeMedia();

		expect(player.currentEpisode?.id).toBe(1);
		expect(player.upNext.map((item) => item.id)).toEqual([2]);
		expect(player.stopped).toBe(true);
		expect(player.playing).toBe(false);
		expect(audio.src).toBe('');
		expect(session.metadata).toBeNull();
		expect(session.playbackState).toBe('none');
		for (const handler of session.handlers.values()) expect(handler).toBeNull();

		stalePlay({ action: 'play' });
		await Promise.resolve();
		expect(player.playing).toBe(false);
		expect(audio.src).toBe('');

		await player.togglePlay();
		expect(player.playing).toBe(true);
		expect(session.handlers.get('play')).toBeTypeOf('function');
		expect(session.metadata?.title).toBe('Episode 1');
	});
});

describe('player store per-channel playback speed', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		MockAudioElement.instances.length = 0;
		localStorage.clear();
		vi.clearAllMocks();
		setActivePinia(createPinia());
	});

	function channelEpisode(id: number, slug: string, speed: number): Episode {
		return {
			...episode(id),
			channel_slug: slug,
			channel_title: `Channel ${slug}`,
			playback_speed: speed
		};
	}

	it('starts playback at the channel saved speed from the episode payload', async () => {
		const player = usePlayerStore();
		const items = [channelEpisode(1, 'a', 1.35), channelEpisode(2, 'a', 1.35)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];
		expect(player.speed).toBe(1.35);
		expect(audio.playbackRate).toBe(1.35);
	});

	it('starts at 1x when the channel has no saved speed', async () => {
		const player = usePlayerStore();
		const items = [episode(1), episode(2)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];
		expect(player.speed).toBe(1);
		expect(audio.playbackRate).toBe(1);
	});

	it('auto-advance into a different channel applies the new channel speed', async () => {
		const player = usePlayerStore();
		const items = [channelEpisode(1, 'a', 2), channelEpisode(2, 'b', 1.35)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];
		expect(player.speed).toBe(2);

		// the first episode ends; the queue advances to channel b
		audio.emit('ended');
		await vi.waitFor(() => expect(player.currentEpisode?.id).toBe(2));
		expect(player.speed).toBe(1.35);
		expect(audio.playbackRate).toBe(1.35);
	});

	it('manual skip into a different channel applies its speed and never carries over the old rate', async () => {
		const player = usePlayerStore();
		const items = [channelEpisode(1, 'a', 2), channelEpisode(2, 'b', 1.35)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];

		await player.skipNext();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.speed).toBe(1.35);
		expect(audio.playbackRate).toBe(1.35);

		// the previous control returns to channel a: its 2x comes back and the
		// 1.35x from channel b is not carried over
		await player.playPrevious();
		expect(player.currentEpisode?.id).toBe(1);
		expect(player.speed).toBe(2);
		expect(audio.playbackRate).toBe(2);
	});

	it('same-channel skip keeps the channel speed', async () => {
		const player = usePlayerStore();
		const items = [channelEpisode(1, 'a', 1.35), channelEpisode(2, 'a', 1.35)];
		await player.play(items[0], items);
		await player.skipNext();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.speed).toBe(1.35);
	});

	it('setSpeed rounds, applies, and saves the value per channel', async () => {
		const player = usePlayerStore();
		const items = [channelEpisode(1, 'a', 1)];
		await player.play(items[0], items);
		const audio = MockAudioElement.instances[0];

		player.setSpeed(1.7000000000000002);
		expect(player.speed).toBe(1.7);
		expect(audio.playbackRate).toBe(1.7);
		await vi.waitFor(() => expect(api.setChannelPlaybackSpeed).toHaveBeenCalledWith('a', 1.7));

		// a different channel keeps its own independent value
		const itemsB = [channelEpisode(2, 'b', 2)];
		await player.play(itemsB[0], itemsB);
		player.setSpeed(1.35);
		expect(player.speed).toBe(1.35);
		await vi.waitFor(() => expect(api.setChannelPlaybackSpeed).toHaveBeenCalledWith('b', 1.35));
	});

	it('applies the rate to the element when a second episode starts', async () => {
		const player = usePlayerStore();
		const first = channelEpisode(1, 'a', 2);
		const second = channelEpisode(2, 'b', 1.35);
		await player.play(first, [first]);
		const audio = MockAudioElement.instances[0];
		expect(audio.playbackRate).toBe(2);

		// starting a second episode retargets the source (load() resets the
		// element's rate): the store state updates AND the element must really
		// play at the new channel's speed
		await player.play(second, [second]);
		expect(player.speed).toBe(1.35);
		expect(audio.playbackRate).toBe(1.35);
	});

	it('clamps the speed to the supported range', async () => {
		const player = usePlayerStore();
		await player.play(episode(1), [episode(1)]);
		player.setSpeed(10);
		expect(player.speed).toBe(3);
		player.setSpeed(0.1);
		expect(player.speed).toBe(0.5);
	});

	it('persists channel speeds with the queue and restores them on reload', async () => {
		const player = usePlayerStore();
		const items = [episode(1), episode(2)];
		await player.play(items[0], items);
		player.setSpeed(1.7);

		const stored = JSON.parse(localStorage.getItem('u2vpodcast.up-next.v1') ?? '{}');
		expect(stored.channelSpeedBySlug).toEqual({ c: 1.7 });

		// a reload boots a fresh store over the same payload; resuming starts
		// at the persisted channel speed (the restored episode's own payload
		// value must not override the saved map)
		const reloaded = usePlayerStore(createPinia());
		await reloaded.togglePlay();
		const audio = MockAudioElement.instances[0];
		expect(reloaded.speed).toBe(1.7);
		expect(audio.playbackRate).toBe(1.7);
	});
});
