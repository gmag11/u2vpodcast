import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import PersistentPlayer from '@/components/PersistentPlayer.vue';
import { usePlayerStore } from '@/stores/player';
import { testI18n } from '@/test/i18n';
import type { Episode } from '@/types';

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

describe('PersistentPlayer next control', () => {
	beforeEach(() => {
		vi.stubGlobal('HTMLAudioElement', AudioClass);
		setActivePinia(createPinia());
	});

	async function mountBar() {
		const wrapper = mount(PersistentPlayer, {
			global: { plugins: [testI18n] }
		});
		await flushPromises();
		return wrapper;
	}

	it('renders the next button to the right of the stop button', async () => {
		const player = usePlayerStore();
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;

		const wrapper = await mountBar();
		const bar = wrapper.get('.fixed.bottom-0');
		const stopBtn = bar.get('button[aria-label="Stop"]');
		const nextBtn = bar.get('button[aria-label="Next"]');

		const stopPos = stopBtn.element.compareDocumentPosition(nextBtn.element);
		// NEXT is documented order: 4 = FOLLOWING
		expect(stopPos & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
	});

	it('enables the next button when the queue holds an episode and advances on click', async () => {
		const player = usePlayerStore();
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;
		player.upNext = [episode(2)];

		const wrapper = await mountBar();
		const nextBtn = wrapper.get('button[aria-label="Next"]');
		expect((nextBtn.element as HTMLButtonElement).disabled).toBe(false);

		await nextBtn.trigger('click');
		await flushPromises();
		expect(player.currentEpisode?.id).toBe(2);
		expect(player.upNext).toEqual([]);
	});

	it('disables the next button when the queue is empty', async () => {
		const player = usePlayerStore();
		player.currentEpisode = episode(1);
		player.playing = true;
		player.stopped = false;
		player.upNext = [];

		const wrapper = await mountBar();
		const nextBtn = wrapper.get('button[aria-label="Next"]');
		expect((nextBtn.element as HTMLButtonElement).disabled).toBe(true);
	});
});