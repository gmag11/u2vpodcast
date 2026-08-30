import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import PersistentPlayer from '@/components/PersistentPlayer.vue';
import { usePlayerStore } from '@/stores/player';
import { testI18n } from '@/test/i18n';
import type { Episode } from '@/types';

vi.mock('@/lib/api/client', () => ({
	api: {
		updateEpisodeProgress: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		),
		getEpisodeProgress: vi.fn(() =>
			Promise.resolve({ ok: false, data: null, user: null, status: false })
		),
		setChannelPlaybackSpeed: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		)
	}
}));

class MockAudioElement {
	src = '';
	currentTime = 0;
	duration = 0;
	volume = 1;
	muted = false;
	playbackRate = 1;
	// A real media element keeps the rate a load() should reset back to.
	defaultPlaybackRate = 1;
	paused = true;
	preload = 'metadata';

	private listeners: Record<string, Array<() => void>> = {};

	play = vi.fn(async () => {
		this.paused = false;
	});
	pause = vi.fn(() => {
		this.paused = true;
	});
	load = vi.fn(() => {
		this.playbackRate = this.defaultPlaybackRate;
	});

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
		sponsorblock_enabled: true,
		created_at: now,
		updated_at: now
	};
}

describe('PersistentPlayer controls', () => {
	let wrapper: ReturnType<typeof mount> | null = null;

	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		localStorage.clear();
		setActivePinia(createPinia());
	});

	afterEach(() => {
		wrapper?.unmount();
		wrapper = null;
		vi.useRealTimers();
	});

	async function mountBar() {
		wrapper = mount(PersistentPlayer, {
			global: { plugins: [testI18n] }
		});
		await flushPromises();
		return wrapper.get('[data-testid="player-wide"]');
	}

	function startPlayback(player: ReturnType<typeof usePlayerStore>) {
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;
	}

	it('renders the next button to the right of the stop button', async () => {
		const player = usePlayerStore();
		startPlayback(player);

		const bar = await mountBar();
		const stopBtn = bar.get('button[aria-label="Stop"]');
		const nextBtn = bar.get('button[aria-label="Next"]');

		const stopPos = stopBtn.element.compareDocumentPosition(nextBtn.element);
		expect(stopPos & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
	});

	it('enables the next button with a queue and skips on click', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2)];

		const nextBtn = (await mountBar()).get('button[aria-label="Next"]');
		expect((nextBtn.element as HTMLButtonElement).disabled).toBe(false);

		await nextBtn.trigger('click');
		await flushPromises();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.upNext).toEqual([]);
	});

	it('disables the next button when the queue is empty', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [];

		const nextBtn = (await mountBar()).get('button[aria-label="Next"]');
		expect((nextBtn.element as HTMLButtonElement).disabled).toBe(true);
	});

	it('keeps every SponsorBlock category visible with the expected colors while paused', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			sponsorblock_segments: [
				{ start: 60, end: 120, category: 'sponsor', rejected: true },
				{ start: 90, end: 150, category: 'intro', rejected: false }
			]
		};
		const bar = await mountBar();

		player.playing = false;
		await flushPromises();

		const markers = bar.findAll('[data-testid="player-sponsorblock-segment"]');
		expect(markers).toHaveLength(2);
		expect(markers[0].classes()).toContain('bg-sponsorblock');
		expect(markers[1].classes()).toContain('bg-sponsorblock-other');
		expect(markers[0].attributes('style')).toContain('left: 10%');
		expect(markers[0].attributes('style')).toContain('width: 10%');
	});

	it('renders no SponsorBlock markers when disabled', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			sponsorblock_enabled: false,
			sponsorblock_segments: [{ start: 60, end: 120, category: 'sponsor', rejected: true }]
		};
		const bar = await mountBar();
		expect(bar.find('[data-testid="player-sponsorblock-segment"]').exists()).toBe(false);
	});

	it('previous restarts the current episode beyond 3 seconds', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentTime = 12;

		const prevBtn = (await mountBar()).get('button[aria-label="Previous"]');
		await prevBtn.trigger('click');
		await flushPromises();
		expect(player.currentEpisode?.id).toBe(1);
		expect(player.currentTime).toBe(0);
	});

	it('previous navigates back within 3 seconds', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2)];
		player.playStack = [episode(0)];
		player.currentTime = 1;

		const prevBtn = (await mountBar()).get('button[aria-label="Previous"]');
		await prevBtn.trigger('click');
		await flushPromises();
		expect(player.currentEpisode?.id).toBe(0);
		expect(player.playStack).toEqual([]);
	});

	it('long press on next skips and marks listened', async () => {
		vi.useFakeTimers();
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2)];

		const nextBtn = (await mountBar()).get('button[aria-label="Next"]');
		await nextBtn.trigger('pointerdown');
		await vi.advanceTimersByTimeAsync(600);
		await flushPromises();

		expect(player.currentEpisode?.id).toBe(2);
		expect(player.playStack[0].listen).toBe(true);

		// the release after a long press must not skip twice
		await nextBtn.trigger('pointerup');
		await nextBtn.trigger('click');
		await flushPromises();
		expect(player.currentEpisode?.id).toBe(2);
	});

	it('opens the queue popover and removes an item', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2), episode(3)];

		const bar = await mountBar();
		const toggle = bar.get('button[aria-label="Up next queue"]');
		await toggle.trigger('click');

		expect(bar.text()).toContain('Episode 2');
		expect(bar.text()).toContain('Episode 3');

		const remove = bar.get('button[aria-label="Remove from queue"]');
		await remove.trigger('click');
		expect(player.upNext.map((e) => e.id)).toEqual([3]);
	});

	it('shows an empty state in the queue popover and clears all', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2), episode(3)];

		const bar = await mountBar();
		const toggle = bar.get('button[aria-label="Up next queue"]');
		await toggle.trigger('click');

		const clear = bar.findAll('button').find((b) => b.text().includes('Clear'));
		expect(clear).toBeDefined();
		await clear!.trigger('click');

		expect(player.upNext).toEqual([]);
		await toggle.trigger('click');
		expect(bar.text()).toContain('No episodes queued');
	});

	it('keeps the bar visible after stop while the queue is non-empty, then hides when it empties', async () => {
		vi.useFakeTimers();
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2), episode(3)];
		await mountBar();
		expect(wrapper!.find('.fixed.bottom-0').exists()).toBe(true);

		// stop playback with items still queued -> stays visible
		player.playing = false;
		player.stopped = true;
		await flushPromises();
		expect(wrapper!.find('.fixed.bottom-0').exists()).toBe(true);

		// empty the queue -> the 10s hide timer arms and the bar leaves
		player.clearQueue();
		await vi.advanceTimersByTimeAsync(10050);
		await flushPromises();
		expect(wrapper!.find('.fixed.bottom-0').exists()).toBe(false);
	});

	it('shows the bar in queue-only mode after a reload with no current episode', async () => {
		const player = usePlayerStore();
		player.currentEpisode = null;
		player.upNext = [episode(2)];
		player.stopped = true;

		const bar = await mountBar();
		expect(wrapper!.find('.fixed.bottom-0').exists()).toBe(true);
		expect(bar.text()).toContain('Queue ready');

		const playBtn = bar.get('button[aria-label="Play"]');
		expect((playBtn.element as HTMLButtonElement).disabled).toBe(true);

		// queue stays reachable: the popover opens without a current episode
		await bar.get('button[aria-label="Up next queue"]').trigger('click');
		expect(bar.text()).toContain('Episode 2');
	});

	it('renders only play or pause as a compact control', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		await mountBar();

		const compact = wrapper!.get('[data-testid="player-compact"]');
		expect(compact.findAll('button')).toHaveLength(1);
		expect(compact.get('button').attributes('aria-label')).toBe('Pause');
		expect(compact.find('[role="slider"]').exists()).toBe(false);
	});

	it('shows channel and elapsed-only compact clock including hour rollover', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = { ...player.currentEpisode!, channel_title: 'VisualPolitik' };
		player.duration = 7200;
		player.currentTime = 669;
		await mountBar();

		const compact = wrapper!.get('[data-testid="player-compact"]');
		expect(compact.text()).toContain('VisualPolitik • 11:09');
		expect(compact.text()).not.toContain(player.durationLabel);

		player.currentTime = 3600;
		await flushPromises();
		expect(compact.text()).toContain('VisualPolitik • 1:00:00');
	});

	it('toggles playback from the compact control', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const togglePlay = vi.spyOn(player, 'togglePlay').mockImplementation(async () => {
			player.playing = !player.playing;
		});
		await mountBar();

		await wrapper!.get('[data-testid="player-compact"] button').trigger('click');
		expect(togglePlay).toHaveBeenCalledOnce();
		expect(player.playing).toBe(false);
	});

	it('renders matching SponsorBlock markers on the compact read-only track', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			sponsorblock_segments: [
				{ start: 60, end: 120, category: 'sponsor', rejected: true },
				{ start: 180, end: 240, category: 'intro', rejected: false }
			]
		};
		player.currentTime = 90;
		await mountBar();

		const compact = wrapper!.get('[data-testid="player-compact"]');
		const wide = wrapper!.get('[data-testid="player-wide"]');
		const compactMarkers = compact.findAll('[data-testid="player-sponsorblock-segment"]');
		const wideMarkers = wide.findAll('[data-testid="player-sponsorblock-segment"]');
		expect(compactMarkers).toHaveLength(2);
		expect(compactMarkers[0].classes()).toContain('bg-sponsorblock');
		expect(compactMarkers[1].classes()).toContain('bg-sponsorblock-other');
		expect(compactMarkers.map((marker) => marker.attributes('style'))).toEqual(
			wideMarkers.map((marker) => marker.attributes('style'))
		);

		const track = compact.get('[data-testid="player-progress-compact"]');
		expect(track.attributes('aria-hidden')).toBe('true');
		expect(track.attributes('role')).toBeUndefined();
		expect(track.attributes('tabindex')).toBeUndefined();
		await track.trigger('click');
		expect(player.currentTime).toBe(90);
	});
});

describe('PersistentPlayer speed control', () => {
	let wrapper: ReturnType<typeof mount> | null = null;

	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		vi.stubGlobal('Audio', AudioClass);
		localStorage.clear();
		setActivePinia(createPinia());
	});

	afterEach(() => {
		wrapper?.unmount();
		wrapper = null;
	});

	async function mountBar() {
		wrapper = mount(PersistentPlayer, {
			global: { plugins: [testI18n] }
		});
		await flushPromises();
		return wrapper.get('[data-testid="player-wide"]');
	}

	function startPlayback(player: ReturnType<typeof usePlayerStore>) {
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;
	}

	function openPanel(bar: Awaited<ReturnType<typeof mountBar>>) {
		return bar.get('button[aria-label="Playback speed"]').trigger('click');
	}

	it('adjusts the speed in half-tenth steps with the stepper', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const bar = await mountBar();
		await openPanel(bar);

		const value = () => bar.get('[data-testid="speed-value"]').text();
		expect(value()).toBe('1x');

		await bar.get('button[aria-label="Increase speed"]').trigger('click');
		expect(value()).toBe('1.05x');
		await bar.get('button[aria-label="Increase speed"]').trigger('click');
		expect(value()).toBe('1.1x');
		await bar.get('button[aria-label="Decrease speed"]').trigger('click');
		expect(value()).toBe('1.05x');
		expect(player.speed).toBe(1.05);
	});

	it('keeps the panel open while stepping', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const bar = await mountBar();
		await openPanel(bar);
		await bar.get('button[aria-label="Increase speed"]').trigger('click');
		expect(bar.find('[data-testid="speed-panel"]').exists()).toBe(true);
	});

	it('presets are still selectable from the panel', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const bar = await mountBar();
		await openPanel(bar);

		const preset = bar
			.findAll('[data-testid="speed-panel"] button')
			.find((b) => b.text() === '1.5x');
		expect(preset).toBeTruthy();
		await preset!.trigger('click');
		expect(player.speed).toBe(1.5);
		// selecting a preset closes the panel
		expect(bar.find('[data-testid="speed-panel"]').exists()).toBe(false);
	});

	it('disables the steppers at the range bounds', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const bar = await mountBar();
		player.speed = 0.5;
		await openPanel(bar);
		expect(
			(bar.get('button[aria-label="Decrease speed"]').element as HTMLButtonElement).disabled
		).toBe(true);
		expect(
			(bar.get('button[aria-label="Increase speed"]').element as HTMLButtonElement).disabled
		).toBe(false);

		// crossing the max bound while the panel stays open disables the
		// increase stepper reactively
		player.speed = 3;
		await flushPromises();
		expect(
			(bar.get('button[aria-label="Increase speed"]').element as HTMLButtonElement).disabled
		).toBe(true);
		expect(
			(bar.get('button[aria-label="Decrease speed"]').element as HTMLButtonElement).disabled
		).toBe(false);
	});

	it('formats the displayed label trimming trailing zeros', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.speed = 1.7;
		const bar = await mountBar();
		expect(bar.get('button[aria-label="Playback speed"]').text()).toContain('1.7x');

		await openPanel(bar);
		expect(bar.get('[data-testid="speed-value"]').text()).toBe('1.7x');
	});
});
