import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import EpisodeCard from '@/components/EpisodeCard.vue';
import { testI18n } from '@/test/i18n';
import { usePlayerStore } from '@/stores/player';
import { usePlaylistStore } from '@/stores/playlists';
import { useNotificationStore } from '@/stores/notification';
import { api } from '@/lib/api/client';
import type { Episode } from '@/types';

vi.mock('@/lib/api/client', () => ({
	api: {
		updateEpisodeProgress: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		),
		getEpisodeProgress: vi.fn(() =>
			Promise.resolve({ ok: false, data: null, user: null, status: false })
		),
		getPlaylist: vi.fn(() =>
			Promise.resolve({ ok: true, data: [], user: null, status: true })
		),
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

function episode(overrides: Partial<Episode> = {}): Episode {
	const now = new Date();
	return {
		id: 1,
		channel_id: 1,
		channel_slug: 'c',
		channel_title: 'Channel',
		title: 'Episode 1',
		description: 'Description',
		yt_id: 'yt1',
		webpage_url: 'https://www.youtube.com/watch',
		published_at: now,
		duration: '01:00:00',
		image: '',
		listen: false,
		position_seconds: 0,
		listened_at: null,
		created_at: now,
		updated_at: now,
		...overrides
	};
}

function mountCard(ep: Episode) {
	return mount(EpisodeCard, {
		props: { episode: ep },
		global: { plugins: [createPinia(), testI18n], stubs: { RouterLink: true } }
	});
}

describe('EpisodeCard playback indicators', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	it('shows the played mark on completed episodes', () => {
		const wrapper = mountCard(episode({ listen: true, position_seconds: 3600 }));
		expect(wrapper.find('[data-testid="listened-mark"]').exists()).toBe(true);
		expect(wrapper.find('[aria-label="Listened"]').exists()).toBe(true);
	});

	it('shows a resume hint for partially played episodes', () => {
		const wrapper = mountCard(episode({ position_seconds: 100 }));
		expect(wrapper.text()).toContain('Continue at 01:40');
	});

	it('shows neither indicator for untouched episodes', () => {
		const wrapper = mountCard(episode({ position_seconds: 0 }));
		expect(wrapper.find('[aria-label="Listened"]').exists()).toBe(false);
		expect(wrapper.text()).not.toContain('Continue at');
	});

	it('shows a read-only progress strip sized to the saved position', () => {
		const wrapper = mountCard(episode({ position_seconds: 1800 })); // 50% of 01:00:00
		const bar = wrapper.find('[data-testid="episode-progress"]');
		expect(bar.exists()).toBe(true);
		expect(bar.find('div').attributes('style')).toContain('width: 50%');
	});

	it('sizes the progress strip to 100% for completed episodes', () => {
		const wrapper = mountCard(episode({ listen: true, position_seconds: 3600 }));
		const bar = wrapper.find('[data-testid="episode-progress"]');
		expect(bar.exists()).toBe(true);
		expect(bar.find('div').attributes('style')).toContain('width: 100%');
	});

	it('enables stop on a non-current episode and resets its progress', async () => {
		const pinia = createPinia();
		setActivePinia(pinia);
		const player = usePlayerStore();
		const spy = vi.spyOn(player, 'stop').mockImplementation(() => undefined);
		const wrapper = mount(EpisodeCard, {
			props: { episode: episode({ position_seconds: 100 }) },
			global: { plugins: [pinia, testI18n], stubs: { RouterLink: true } }
		});

		const stopBtn = wrapper.find('[aria-label="Stop"]');
		expect((stopBtn.element as HTMLButtonElement).disabled).toBe(false);
		await stopBtn.trigger('click');
		expect(spy).toHaveBeenCalledWith(expect.objectContaining({ id: 1 }));
	});

	it('does not render the progress strip without a saved position', () => {
		const wrapper = mountCard(episode({ position_seconds: 0 }));
		expect(wrapper.find('[data-testid="episode-progress"]').exists()).toBe(false);
	});
});

describe('EpisodeCard playlist toggle', () => {
	function mountCardWith(ep: Episode, seedPlaylist: Episode[]) {
		const pinia = createPinia();
		const playlists = usePlaylistStore(pinia);
		playlists.items = seedPlaylist;
		useNotificationStore(pinia);
		const wrapper = mount(EpisodeCard, {
			props: { episode: ep },
			global: { plugins: [pinia, testI18n], stubs: { RouterLink: true } }
		});
		return wrapper;
	}

	beforeEach(() => {
		localStorage.clear();
		vi.clearAllMocks();
	});

	it('renders remove when the episode is in the playlist id set', () => {
		const wrapper = mountCardWith(episode({ id: 7 }), [episode({ id: 7, title: 'Queued' })]);
		expect(wrapper.find('[aria-label="Remove from playlist"]').exists()).toBe(true);
		expect(wrapper.find('[aria-label="Add to playlist"]').exists()).toBe(false);
	});

	it('renders add when the episode is absent from the playlist', () => {
		const wrapper = mountCardWith(episode({ id: 1 }), [episode({ id: 7 })]);
		expect(wrapper.find('[aria-label="Add to playlist"]').exists()).toBe(true);
		expect(wrapper.find('[aria-label="Remove from playlist"]').exists()).toBe(false);
	});

	it('removing calls the api and notifies', async () => {
		const wrapper = mountCardWith(episode({ id: 7 }), [episode({ id: 7 })]);
		await wrapper.find('[aria-label="Remove from playlist"]').trigger('click');
		await flushPromises();
		expect(api.removeEpisodeFromPlaylist).toHaveBeenCalledWith(7);
		const notification = useNotificationStore();
		expect(notification.current?.message).toBe('Removed from playlist');
	});

	it('adding calls the api and flips the toggle after the reload', async () => {
		vi.mocked(api.getPlaylist).mockResolvedValue({
			ok: true,
			data: [episode({ id: 1 }), episode({ id: 7 })],
			user: null,
			status: true
		} as never);
		const wrapper = mountCardWith(episode({ id: 1 }), [episode({ id: 7 })]);
		await wrapper.find('[aria-label="Add to playlist"]').trigger('click');
		await flushPromises();
		expect(api.addEpisodeToPlaylist).toHaveBeenCalledWith(1);
		expect(wrapper.find('[aria-label="Remove from playlist"]').exists()).toBe(true);
		const notification = useNotificationStore();
		expect(notification.current?.message).toBe('Added to playlist');
	});

	it('unmarks a listened episode, clearing progress and re-adding it', async () => {
		const wrapper = mountCardWith(episode({ id: 1, listen: true, position_seconds: 3600 }), []);
		expect(wrapper.find('[aria-label="Mark as not listened"]').exists()).toBe(true);
		await wrapper.find('[aria-label="Mark as not listened"]').trigger('click');
		await flushPromises();
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt1', {
			position_seconds: 0,
			listened: false
		});
		expect(api.addEpisodeToPlaylist).toHaveBeenCalledWith(1);
		// the played mark clears immediately from the shared per-id progress
		expect(wrapper.find('[data-testid="listened-mark"]').exists()).toBe(false);
		expect(wrapper.find('[aria-label="Mark as not listened"]').exists()).toBe(false);
	});

	it('keeps the cleared mark and surfaces the error when the re-add fails', async () => {
		vi.mocked(api.addEpisodeToPlaylist).mockResolvedValue({
			ok: false,
			data: null,
			user: null,
			status: false
		} as never);
		const wrapper = mountCardWith(episode({ id: 1, listen: true, position_seconds: 3600 }), []);
		await wrapper.find('[aria-label="Mark as not listened"]').trigger('click');
		await flushPromises();
		expect(wrapper.find('[data-testid="listened-mark"]').exists()).toBe(false);
		const notification = useNotificationStore();
		expect(notification.current?.message).toBe('Could not add to playlist');
	});
});