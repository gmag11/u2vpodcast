import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import PersistentPlayerExpanded from '@/components/PersistentPlayerExpanded.vue';
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
		sponsorblock_enabled: true,
		created_at: now,
		updated_at: now
	};
}

describe('PersistentPlayerExpanded', () => {
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

	async function mountExpanded(open = true) {
		wrapper = mount(PersistentPlayerExpanded, {
			props: { open },
			global: { plugins: [testI18n] }
		});
		await flushPromises();
		return wrapper;
	}

	function startPlayback(player: ReturnType<typeof usePlayerStore>) {
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;
		player.duration = 600;
		player.currentTime = 60;
	}

	it('does not render when closed, renders when open', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded(false);
		expect(w.find('[data-testid="player-expanded"]').exists()).toBe(false);

		await w.setProps({ open: true });
		expect(w.find('[data-testid="player-expanded"]').exists()).toBe(true);
	});

	it('emits close when the chevron is pressed', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();
		await w.get('button[aria-label="Collapse now-playing view"]').trigger('click');
		expect(w.emitted('close')).toHaveLength(1);
	});

	it('renders an interactive scrubber that seeks on click', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const seekSpy = vi.spyOn(player, 'seek');
		const w = await mountExpanded();

		const track = w.get('[data-testid="player-progress-expanded"]');
		expect(track.attributes('role')).toBe('slider');
		vi.spyOn(track.element, 'getBoundingClientRect').mockReturnValue({
			left: 0,
			width: 100
		} as DOMRect);
		await track.trigger('click', { clientX: 50 });
		expect(seekSpy).toHaveBeenCalledWith(300);
	});

	it('shows elapsed and remaining time labels', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();
		expect(w.text()).toContain('1:00');
		expect(w.text()).toContain('-9:00');
	});

	it('uses the continuous scrolling title behavior while playing', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();

		const title = w.get('[data-testid="scrolling-text-viewport"]');
		expect(title.attributes('aria-label')).toBe('Episode 1');
		expect(w.getComponent({ name: 'ScrollingText' }).props('active')).toBe(true);

		player.playing = false;
		await flushPromises();
		expect(w.getComponent({ name: 'ScrollingText' }).props('active')).toBe(false);
	});

	it('exposes speed, transport, mode toggle and queue controls', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2)];
		const w = await mountExpanded();

		expect(w.find('button[aria-label="Previous"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Seek back 10 seconds"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Pause"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Seek forward 10 seconds"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Next"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Playback speed"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Up next queue"]').exists()).toBe(true);
		expect(w.find('button[aria-label="Normal order"]').exists()).toBe(true);
	});

	it('cycles the combined mode control', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();

		const toggle = w.get('button[aria-label="Normal order"]');
		await toggle.trigger('click');
		expect(player.mobilePlaybackMode).toBe('repeat');
	});

	it('seeks ±10 seconds from the transport controls', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const seekRelativeSpy = vi.spyOn(player, 'seekRelative');
		const w = await mountExpanded();

		await w.get('button[aria-label="Seek back 10 seconds"]').trigger('click');
		expect(seekRelativeSpy).toHaveBeenCalledWith(-10);
		await w.get('button[aria-label="Seek forward 10 seconds"]').trigger('click');
		expect(seekRelativeSpy).toHaveBeenCalledWith(10);
	});

	it('long press on next skips and marks listened', async () => {
		vi.useFakeTimers();
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2)];
		const w = await mountExpanded();

		const nextBtn = w.get('button[aria-label="Next"]');
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

		vi.useRealTimers();
	});

	it('opens the queue panel and lists upcoming episodes', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.upNext = [episode(2), episode(3)];
		const w = await mountExpanded();

		await w.get('button[aria-label="Up next queue"]').trigger('click');
		expect(w.text()).toContain('Episode 2');
		expect(w.text()).toContain('Episode 3');

		const remove = w.get('button[aria-label="Remove from queue"]');
		await remove.trigger('click');
		expect(player.upNext.map((e) => e.id)).toEqual([3]);
	});

	it('renders no volume or mute control', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();

		expect(w.find('button[aria-label="Mute"]').exists()).toBe(false);
		expect(w.find('button[aria-label="Unmute"]').exists()).toBe(false);
		expect(w.find('input[type="range"]').exists()).toBe(false);
	});
});
