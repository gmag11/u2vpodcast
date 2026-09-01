import { flushPromises, mount, type VueWrapper } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { createMemoryHistory, createRouter } from 'vue-router';
import { defineComponent, nextTick } from 'vue';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { VueDraggable } from 'vue-draggable-plus';
import PlaylistView from '@/views/PlaylistView.vue';
import { api } from '@/lib/api/client';
import { useNotificationStore } from '@/stores/notification';
import { usePlayerStore } from '@/stores/player';
import { usePlaylistStore } from '@/stores/playlists';
import { testI18n } from '@/test/i18n';
import type { Episode } from '@/types';

const EpisodeCardStub = defineComponent({
	name: 'EpisodeCard',
	props: {
		episode: { type: Object, required: true },
		list: { type: Array, required: true },
		presentation: { type: String, required: true },
		queueSource: { type: String, required: true }
	},
	template:
		'<article class="episode-card"><button class="card-action">{{ episode.title }}</button></article>'
});

function episode(id: number): Episode {
	const now = new Date();
	return {
		id,
		channel_id: 1,
		channel_slug: 'channel',
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
		created_at: now,
		updated_at: now
	};
}

const okResult = (data: unknown = null) => ({
	ok: true,
	data,
	user: { id: 1, name: 'admin', role: 'Admin', active: true },
	status: true
});

const failedResult = () => ({ ok: false, data: null, user: null, status: false });

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((done) => {
		resolve = done;
	});
	return { promise, resolve };
}

async function mountView(episodes = [episode(1), episode(2), episode(3)]) {
	const pinia = createPinia();
	setActivePinia(pinia);
	const router = createRouter({
		history: createMemoryHistory(),
		routes: [
			{ path: '/', name: 'channels', component: { template: '<div />' } },
			{ path: '/login', name: 'login', component: { template: '<div />' } },
			{ path: '/playlist', name: 'playlist', component: PlaylistView }
		]
	});
	vi.spyOn(api, 'getPlaylist').mockResolvedValue(okResult(episodes) as never);
	vi.spyOn(api, 'reorderPlaylist').mockResolvedValue(okResult() as never);
	await router.push('/playlist');
	await router.isReady();
	const wrapper = mount(PlaylistView, {
		attachTo: document.body,
		global: {
			plugins: [router, pinia, testI18n],
			stubs: {
				AppHeader: { template: '<header><slot name="brand-icon" /></header>' },
				EpisodeCard: EpisodeCardStub
			}
		}
	});
	await flushPromises();
	return { wrapper, playlists: usePlaylistStore(), player: usePlayerStore() };
}

function rowIds(wrapper: VueWrapper) {
	return wrapper.findAll('.playlist-row').map((row) => Number(row.attributes('data-episode-id')));
}

async function emitDrop(wrapper: VueWrapper, order: Episode[], oldIndex = 0) {
	const draggable = wrapper.findComponent(VueDraggable);
	draggable.vm.$emit('start', { oldIndex });
	draggable.vm.$emit('update:modelValue', order);
	await nextTick();
	draggable.vm.$emit('end', { oldIndex });
	await flushPromises();
}

describe('PlaylistView inline reordering', () => {
	beforeEach(() => {
		vi.restoreAllMocks();
		document.body.innerHTML = '';
	});

	it('replaces caret controls with one six-dot handle per row', async () => {
		const { wrapper } = await mountView();
		const handles = wrapper.findAll('.playlist-drag-handle');
		expect(handles).toHaveLength(3);
		expect(handles[0].attributes('aria-label')).toBe('Reorder Episode 1');
		expect(handles[0].classes()).toContain('h-11');
		expect(handles[0].classes()).toContain('w-11');
		expect(handles[0].find('.playlist-drag-icon').classes()).toContain('h-4');
		expect(wrapper.find('[aria-label="Move up"]').exists()).toBe(false);
		expect(wrapper.find('[aria-label="Move down"]').exists()).toBe(false);
		expect(wrapper.findAll('.episode-card')).toHaveLength(3);
		wrapper.unmount();
	});

	it('opts cards into the playlist presentation without making card actions draggable', async () => {
		const { wrapper } = await mountView();
		const cards = wrapper.findAllComponents(EpisodeCardStub);
		expect(cards).toHaveLength(3);
		expect(cards.every((card) => card.props('presentation') === 'playlist')).toBe(true);

		await wrapper.get('.card-action').trigger('pointerdown');
		await wrapper.get('.card-action').trigger('click');
		expect(api.reorderPlaylist).not.toHaveBeenCalled();
		expect(wrapper.find('.playlist-drag-handle').attributes('aria-pressed')).toBe('false');
		wrapper.unmount();
	});

	it('removes only the mobile list inset while preserving header and desktop spacing', async () => {
		const { wrapper } = await mountView();
		const main = wrapper.get('[data-testid="playlist-main"]');
		expect(main.classes()).toContain('px-0');
		expect(main.classes()).toContain('sm:px-4');
		expect(wrapper.get('[data-testid="playlist-header"]').classes()).toContain('px-4');
		expect(wrapper.get('.playlist-list').classes()).not.toContain('px-4');
		wrapper.unmount();
	});

	it('hides reorder controls when fewer than two episodes exist', async () => {
		const { wrapper } = await mountView([episode(1)]);
		expect(wrapper.find('.playlist-drag-handle').exists()).toBe(false);
		wrapper.unmount();
	});

	it('configures handle-only touch sorting and viewport auto-scroll', async () => {
		const { wrapper } = await mountView();
		const draggable = wrapper.findComponent(VueDraggable);
		expect(draggable.props()).toMatchObject({
			handle: '.playlist-drag-handle',
			draggable: '.playlist-row',
			delay: 180,
			delayOnTouchOnly: true,
			touchStartThreshold: 6,
			fallbackTolerance: 4,
			forceFallback: true,
			scroll: true,
			bubbleScroll: true,
			scrollSensitivity: 80,
			scrollSpeed: 14
		});
		expect(wrapper.find('.episode-card').classes()).not.toContain('touch-none');
		expect(wrapper.find('.playlist-drag-handle').classes()).toContain('touch-none');
		wrapper.unmount();
	});

	it('persists the complete order after dropping a row', async () => {
		const { wrapper } = await mountView();
		await emitDrop(wrapper, [episode(2), episode(3), episode(1)]);
		expect(api.reorderPlaylist).toHaveBeenCalledWith([2, 3, 1]);
		expect(rowIds(wrapper)).toEqual([2, 3, 1]);
		expect(wrapper.find('[role="status"]').text()).toContain(
			'Dropped Episode 1 at position 3 of 3'
		);
		wrapper.unmount();
	});

	it('does not persist a drop that leaves the order unchanged', async () => {
		const episodes = [episode(1), episode(2), episode(3)];
		const { wrapper } = await mountView(episodes);
		await emitDrop(wrapper, episodes);
		expect(api.reorderPlaylist).not.toHaveBeenCalled();
		wrapper.unmount();
	});

	it('updates playlist-sourced up next before reorder persistence settles', async () => {
		const episodes = [episode(1), episode(2), episode(3)];
		const request = deferred<ReturnType<typeof okResult>>();
		const { wrapper, player } = await mountView(episodes);
		vi.mocked(api.reorderPlaylist).mockReturnValue(request.promise as never);
		player.currentEpisode = episodes[0];
		player.queueSource = 'playlist';
		player.upNext = [episodes[1], episodes[2]];

		const draggable = wrapper.findComponent(VueDraggable);
		draggable.vm.$emit('start', { oldIndex: 1 });
		draggable.vm.$emit('update:modelValue', [episodes[0], episodes[2], episodes[1]]);
		await nextTick();
		draggable.vm.$emit('end', { oldIndex: 1 });

		await vi.waitFor(() => expect(api.reorderPlaylist).toHaveBeenCalledWith([1, 3, 2]));
		expect(player.upNext.map((item) => item.id)).toEqual([3, 2]);

		request.resolve(okResult());
		await flushPromises();
		wrapper.unmount();
	});

	it('supports keyboard pickup, movement, drop, and announcements', async () => {
		const { wrapper } = await mountView();
		const firstHandle = wrapper.find('[data-drag-handle="1"]');
		await firstHandle.trigger('keydown', { key: ' ' });
		expect(wrapper.find('[role="status"]').text()).toContain('Picked up Episode 1');
		await firstHandle.trigger('keydown', { key: 'ArrowDown' });
		expect(rowIds(wrapper)).toEqual([2, 1, 3]);
		expect(wrapper.find('[role="status"]').text()).toContain('Moved Episode 1 to position 2 of 3');
		await wrapper.find('[data-drag-handle="1"]').trigger('keydown', { key: 'Enter' });
		await flushPromises();
		expect(api.reorderPlaylist).toHaveBeenCalledWith([2, 1, 3]);
		expect(document.activeElement?.getAttribute('data-drag-handle')).toBe('1');
		wrapper.unmount();
	});

	it('restores keyboard order when the operation is cancelled', async () => {
		const { wrapper } = await mountView();
		await wrapper.find('[data-drag-handle="1"]').trigger('keydown', { key: 'Enter' });
		await wrapper.find('[data-drag-handle="1"]').trigger('keydown', { key: 'ArrowDown' });
		await wrapper.find('[data-drag-handle="1"]').trigger('keydown', { key: 'Escape' });
		expect(rowIds(wrapper)).toEqual([1, 2, 3]);
		expect(api.reorderPlaylist).not.toHaveBeenCalled();
		expect(wrapper.find('[role="status"]').text()).toContain('Reordering cancelled');
		wrapper.unmount();
	});

	it('restores the persisted order and notifies when saving fails', async () => {
		vi.spyOn(api, 'reorderPlaylist').mockResolvedValue(failedResult() as never);
		const { wrapper, player } = await mountView();
		vi.mocked(api.reorderPlaylist).mockResolvedValue(failedResult() as never);
		player.currentEpisode = episode(1);
		player.queueSource = 'playlist';
		player.upNext = [episode(2), episode(3)];
		await emitDrop(wrapper, [episode(3), episode(1), episode(2)], 2);
		expect(rowIds(wrapper)).toEqual([1, 2, 3]);
		expect(player.upNext.map((item) => item.id)).toEqual([2, 3]);
		expect(useNotificationStore().current).toEqual({
			message: 'Could not save the playlist order',
			type: 'error'
		});
		wrapper.unmount();
	});

	it('rejects a stale drop after an external playlist removal', async () => {
		const { wrapper, playlists } = await mountView();
		const draggable = wrapper.findComponent(VueDraggable);
		draggable.vm.$emit('start', { oldIndex: 0 });
		draggable.vm.$emit('update:modelValue', [episode(2), episode(3), episode(1)]);
		await nextTick();
		playlists.items = playlists.items.filter((item) => item.id !== 2);
		draggable.vm.$emit('end', { oldIndex: 0 });
		await flushPromises();
		expect(api.reorderPlaylist).not.toHaveBeenCalled();
		expect(rowIds(wrapper)).toEqual([1, 3]);
		wrapper.unmount();
	});

	it('play all seeds playback from the current reordered playlist', async () => {
		const { wrapper, player } = await mountView();
		const play = vi.spyOn(player, 'play').mockResolvedValue();
		await emitDrop(wrapper, [episode(2), episode(3), episode(1)]);
		const playAllButton = wrapper.get('[data-testid="playlist-play-all"]');
		expect(playAllButton.attributes('aria-label')).toBe('Play all');
		expect(playAllButton.get('span').classes()).toEqual(
			expect.arrayContaining(['hidden', 'sm:inline'])
		);
		await playAllButton.trigger('click');
		expect(play).toHaveBeenCalledOnce();
		const [firstEpisode, queue, options] = play.mock.calls[0];
		expect(firstEpisode.id).toBe(2);
		expect(queue?.map((item) => item.id)).toEqual([2, 3, 1]);
		expect(options).toEqual({ queueSource: 'playlist' });
		wrapper.unmount();
	});
});
