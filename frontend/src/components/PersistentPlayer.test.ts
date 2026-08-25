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
		position_seconds: 0,
		listened_at: null,
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
		return wrapper;
	}

	function startPlayback(player: ReturnType<typeof usePlayerStore>) {
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;
	}

	it('renders the next button to the right of the stop button', async () => {
		const player = usePlayerStore();
		startPlayback(player);

		const bar = (await mountBar()).get('.fixed.bottom-0');
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
		const bar = await mountBar();
		expect(bar.find('.fixed.bottom-0').exists()).toBe(true);

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
		expect(bar.find('.fixed.bottom-0').exists()).toBe(true);
		expect(bar.text()).toContain('Queue ready');

		const playBtn = bar.get('button[aria-label="Play"]');
		expect((playBtn.element as HTMLButtonElement).disabled).toBe(true);

		// queue stays reachable: the popover opens without a current episode
		await bar.get('button[aria-label="Up next queue"]').trigger('click');
		expect(bar.text()).toContain('Episode 2');
	});
});