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
		chapters: [],
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

	it('renders chapter markers on the scrubber and seeks on activation', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			chapters: [
				{ start: 0, end: 150, title: 'Introduction' },
				{ start: 150, end: 300, title: 'Main topic' }
			]
		};
		const seekSpy = vi.spyOn(player, 'seek');
		const w = await mountExpanded();

		const markers = w.findAll('[data-testid="player-chapter-marker"]');
		expect(markers).toHaveLength(2);
		expect(markers[0].get('[aria-hidden="true"]').classes()).toContain('bg-chapter-marker');
		expect(markers[0].attributes('style')).toContain('left: 0%');
		expect(markers[1].attributes('style')).toContain('left: 25%');
		expect(markers[1].attributes('title')).toBeUndefined();
		const tooltip = markers[1].get('[role="tooltip"]');
		expect(tooltip.text()).toBe('Main topic');
		expect(tooltip.classes()).toContain('group-hover:opacity-100');
		expect(tooltip.classes()).toContain('group-focus-visible:opacity-100');
		expect(markers[1].attributes('aria-describedby')).toBe(tooltip.attributes('id'));
		expect(markers[1].element.tagName).toBe('BUTTON');
		expect(markers[1].attributes('tabindex')).toBeUndefined();

		await markers[1].trigger('click');
		expect(seekSpy).toHaveBeenCalledOnce();
		expect(seekSpy).toHaveBeenCalledWith(150);
	});

	it('renders no chapter markers without stored chapters', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();

		expect(w.find('[data-testid="player-chapter-marker"]').exists()).toBe(false);
	});

	it('shows the current chapter without reserving space when there is none', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			chapters: [{ start: 10, end: 150, title: 'Introduction' }]
		};
		const w = await mountExpanded();

		expect(w.get('[data-testid="player-current-chapter"]').text()).toBe('Introduction');
		player.currentEpisode = { ...player.currentEpisode!, chapters: [] };
		await flushPromises();
		expect(w.find('[data-testid="player-current-chapter"]').exists()).toBe(false);
	});

	it('renders a chapter list only when the episode has chapters', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		const w = await mountExpanded();
		expect(w.find('[data-testid="player-chapters"]').exists()).toBe(false);

		player.currentEpisode = {
			...player.currentEpisode!,
			chapters: [
				{ start: 0, end: 150, title: 'Introduction' },
				{ start: 150, end: 3750, title: 'Main topic' }
			]
		};
		await flushPromises();

		expect(w.get('[data-testid="player-chapters"]').text()).toContain('Chapters');
		const rows = w.findAll('[data-testid="player-chapter-row"]');
		expect(rows).toHaveLength(2);
		expect(rows[0].text()).toContain('Introduction');
		expect(rows[0].text()).toContain('0:00');
		expect(rows[1].text()).toContain('Main topic');
		expect(rows[1].text()).toContain('2:30');
	});

	it('seeks to a chapter start when its row is activated', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			chapters: [
				{ start: 0, end: 150, title: 'Introduction' },
				{ start: 150, end: 300, title: 'Main topic' }
			]
		};
		const seekSpy = vi.spyOn(player, 'seek');
		const w = await mountExpanded();

		await w.findAll('[data-testid="player-chapter-row"]')[1].trigger('click');
		expect(seekSpy).toHaveBeenCalledOnce();
		expect(seekSpy).toHaveBeenCalledWith(150);
	});

	it('updates the highlighted chapter as playback crosses a boundary', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			chapters: [
				{ start: 0, end: 150, title: 'Introduction' },
				{ start: 150, end: 300, title: 'Main topic' }
			]
		};
		const w = await mountExpanded();
		const rows = w.findAll('[data-testid="player-chapter-row"]');

		expect(rows[0].attributes('aria-current')).toBe('true');
		expect(rows[1].attributes('aria-current')).toBeUndefined();
		player.currentTime = 150;
		await flushPromises();
		expect(rows[0].attributes('aria-current')).toBeUndefined();
		expect(rows[1].attributes('aria-current')).toBe('true');
	});

	it('keeps a long chapter list internally scrollable', async () => {
		const player = usePlayerStore();
		startPlayback(player);
		player.currentEpisode = {
			...player.currentEpisode!,
			chapters: Array.from({ length: 20 }, (_, index) => ({
				start: index * 30,
				end: (index + 1) * 30,
				title: `Chapter ${index + 1}`
			}))
		};
		const w = await mountExpanded();
		const list = w.get('[data-testid="player-chapter-list"]');

		expect(w.findAll('[data-testid="player-chapter-row"]')).toHaveLength(20);
		expect(list.classes()).toContain('max-h-64');
		expect(list.classes()).toContain('overflow-y-auto');
		expect(w.find('button[aria-label="Pause"]').exists()).toBe(true);
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
