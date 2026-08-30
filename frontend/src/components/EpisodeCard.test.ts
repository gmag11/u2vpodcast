import { flushPromises, mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import EpisodeCard from '@/components/EpisodeCard.vue';
import { testI18n } from '@/test/i18n';
import { usePlayerStore } from '@/stores/player';
import { usePlaylistStore } from '@/stores/playlists';
import { useFavoritesStore } from '@/stores/favorites';
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
		setEpisodeFavorite: vi.fn(() =>
			Promise.resolve({ ok: true, data: null, user: null, status: true })
		),
		refreshEpisodeSponsorBlock: vi.fn(() =>
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
		favorite: false,
		sponsorblock_enabled: true,
		created_at: now,
		updated_at: now,
		playback_speed: 1,
		...overrides
	};
}

function mountCard(ep: Episode) {
	return mount(EpisodeCard, {
		props: { episode: ep },
		global: { plugins: [createPinia(), testI18n], stubs: { RouterLink: true } }
	});
}

function mountPlaylistCard(ep: Episode, attachToDocument = false, inPlaylist = true) {
	const pinia = createPinia();
	const playlists = usePlaylistStore(pinia);
	playlists.items = inPlaylist ? [ep] : [];
	return mount(EpisodeCard, {
		...(attachToDocument ? { attachTo: document.body } : {}),
		props: {
			episode: ep,
			list: [ep],
			compact: true,
			presentation: 'playlist',
			queueSource: 'playlist'
		},
		global: { plugins: [pinia, testI18n], stubs: { RouterLink: true } }
	});
}

function mockTitleMeasurements(viewportWidth = 100) {
	const clientWidth = vi
		.spyOn(HTMLElement.prototype, 'clientWidth', 'get')
		.mockImplementation(function (this: HTMLElement) {
			return this.dataset.testid === 'scrolling-text-viewport' ? viewportWidth : 0;
		});
	const scrollWidth = vi
		.spyOn(HTMLElement.prototype, 'scrollWidth', 'get')
		.mockImplementation(function (this: HTMLElement) {
			return this.dataset.testid === 'scrolling-text-text'
				? (this.textContent?.length ?? 0) * 8
				: 0;
		});
	return () => {
		clientWidth.mockRestore();
		scrollWidth.mockRestore();
	};
}

function marqueeMetric(wrapper: ReturnType<typeof mountPlaylistCard>, property: string) {
	const track = wrapper.get('[data-testid="scrolling-text-track"]').element as HTMLElement;
	return Number.parseFloat(track.style.getPropertyValue(property));
}

describe('EpisodeCard mobile playlist presentation', () => {
	beforeEach(() => {
		localStorage.clear();
		vi.clearAllMocks();
		testI18n.global.locale.value = 'en';
	});

	it('renders the dense mobile content only when explicitly selected', () => {
		const ep = episode({
			title: 'A complete episode title that needs horizontal scrolling',
			description: 'This description must stay out of the mobile playlist row',
			channel_title: 'Static channel',
			image: 'https://example.com/cover.jpg',
			favorite: true,
			position_seconds: 120
		});
		const playlistWrapper = mountPlaylistCard(ep);
		const mobile = playlistWrapper.get('[data-testid="playlist-mobile-card"]');

		expect(mobile.get('[data-testid="playlist-image-playback"]').attributes('aria-label')).toBe(
			'Play'
		);
		expect(mobile.findComponent({ name: 'ScrollingText' }).exists()).toBe(true);
		expect(mobile.get('[data-testid="playlist-channel"]').text()).toBe('Static channel');
		expect(mobile.text()).toContain('1:00:00');
		expect(mobile.text()).not.toContain(ep.description);
		expect(mobile.find('[aria-label="Stop"]').exists()).toBe(false);
		expect(mobile.get('[data-testid="playlist-favorite-status"]').attributes('data-icon')).toBe(
			'star'
		);
		expect(mobile.get('[data-testid="playlist-favorite-status"]').attributes('role')).toBe('img');
		expect(mobile.get('[data-testid="playlist-favorite-status"]').attributes('data-active')).toBe(
			'true'
		);
		expect(mobile.get('[data-testid="playlist-membership-status"]').attributes('data-active')).toBe(
			'true'
		);
		expect(mobile.get('[data-testid="playlist-status-column"]').classes()).toContain('flex-col');
		expect(mobile.get('[data-testid="playlist-metadata"]').classes()).toContain('justify-between');

		const standardWrapper = mountCard(ep);
		expect(standardWrapper.find('[data-testid="playlist-mobile-card"]').exists()).toBe(false);
		expect(playlistWrapper.find('.hidden.sm\\:flex').exists()).toBe(true);
	});

	it('formats every duration with at least minutes and seconds', async () => {
		const short = mountPlaylistCard(episode({ duration: '48' }));
		expect(short.get('[data-testid="playlist-metadata"]').text()).toContain('0:48');

		const currentEpisode = episode({ id: 7, duration: '9:47' });
		const current = mountPlaylistCard(currentEpisode);
		const player = usePlayerStore();
		player.currentEpisode = currentEpisode;
		player.duration = 0;
		await flushPromises();
		expect(current.get('[data-testid="playlist-metadata"]').text()).toContain('9:47');
	});

	it('uses the image for shared playlist playback and pause', async () => {
		const ep = episode({ id: 7, yt_id: 'yt7' });
		const wrapper = mountPlaylistCard(ep);
		const player = usePlayerStore();
		const play = vi.spyOn(player, 'play').mockImplementation(async () => undefined);
		const togglePlay = vi.spyOn(player, 'togglePlay').mockImplementation(async () => undefined);

		await wrapper.get('[data-testid="playlist-image-playback"]').trigger('click');
		expect(play).toHaveBeenCalledWith(ep, [ep], { queueSource: 'playlist' });

		player.currentEpisode = ep;
		player.playing = true;
		await flushPromises();
		expect(wrapper.get('[data-testid="playlist-image-playback"]').attributes('aria-label')).toBe(
			'Pause'
		);
		await wrapper.get('[data-testid="playlist-image-playback"]').trigger('click');
		expect(togglePlay).toHaveBeenCalledOnce();
	});

	it('scrolls only the playing title and stops it while paused', async () => {
		const restoreMeasurements = mockTitleMeasurements();
		const ep = episode({ title: 'A title wide enough to overflow the mobile row' });
		const wrapper = mountPlaylistCard(ep);
		const player = usePlayerStore();
		await flushPromises();

		const track = wrapper.get('[data-testid="scrolling-text-track"]');
		expect(track.classes()).not.toContain('scrolling-text-track--active');
		expect(wrapper.find('[data-testid="scrolling-text-copy"]').exists()).toBe(true);

		player.currentEpisode = ep;
		player.playing = true;
		await flushPromises();
		expect(track.classes()).toContain('scrolling-text-track--active');
		expect(wrapper.get('[data-testid="scrolling-text-copy"]').attributes('aria-hidden')).toBe(
			'true'
		);

		player.playing = false;
		await flushPromises();
		expect(track.classes()).not.toContain('scrolling-text-track--active');
		restoreMeasurements();
	});

	it('derives animation duration from travel distance at a fixed speed', async () => {
		const restoreMeasurements = mockTitleMeasurements();
		const shorter = mountPlaylistCard(episode({ title: 'A moderately long title' }));
		const longer = mountPlaylistCard(
			episode({ title: 'A substantially longer title that must travel much farther' })
		);
		await flushPromises();

		const shortDistance = marqueeMetric(shorter, '--scrolling-text-distance');
		const longDistance = marqueeMetric(longer, '--scrolling-text-distance');
		const shortDuration = marqueeMetric(shorter, '--scrolling-text-duration');
		const longDuration = marqueeMetric(longer, '--scrolling-text-duration');

		expect(longDistance).toBeGreaterThan(shortDistance);
		expect(longDuration).toBeGreaterThan(shortDuration);
		expect(shortDistance / shortDuration).toBeCloseTo(longDistance / longDuration);
		restoreMeasurements();
	});

	it('reuses loading, favorite, partial, listened, and untouched state', async () => {
		const partial = mountPlaylistCard(episode({ position_seconds: 1800, favorite: true }));
		expect(partial.find('[data-testid="playlist-favorite-status"]').exists()).toBe(true);
		expect(partial.get('[data-testid="episode-progress"] div').attributes('style')).toContain(
			'width: 50%'
		);

		const listened = mountPlaylistCard(episode({ listen: true, position_seconds: 3600 }));
		expect(listened.get('[data-testid="episode-progress"] div').attributes('style')).toContain(
			'width: 100%'
		);

		const untouched = mountPlaylistCard(episode(), false, false);
		expect(untouched.find('[data-testid="playlist-favorite-status"]').exists()).toBe(true);
		expect(
			untouched.get('[data-testid="playlist-favorite-status"]').attributes('data-active')
		).toBe('false');
		expect(
			untouched.get('[data-testid="playlist-membership-status"]').attributes('data-active')
		).toBe('false');
		expect(untouched.find('[data-testid="episode-progress"]').exists()).toBe(false);

		const player = usePlayerStore();
		player.currentEpisode = untouched.props('episode');
		player.loading = true;
		await flushPromises();
		expect(
			(untouched.get('[data-testid="playlist-image-playback"]').element as HTMLButtonElement)
				.disabled
		).toBe(true);
	});

	it('keeps row state icons informational', async () => {
		const wrapper = mountPlaylistCard(episode({ favorite: true }));
		await wrapper.get('[data-testid="playlist-favorite-status"]').trigger('click');
		expect(api.setEpisodeFavorite).not.toHaveBeenCalled();
		expect(api.removeEpisodeFromPlaylist).not.toHaveBeenCalled();
	});

	it('opens the exact ordered action menu and exposes its destinations', async () => {
		const ep = episode({ id: 7, channel_id: 23, favorite: false, position_seconds: 120 });
		const wrapper = mountPlaylistCard(ep);
		const trigger = wrapper.get('[data-testid="playlist-actions-trigger"]');

		await trigger.trigger('click');
		const menu = wrapper.get('[role="menu"]');
		expect(menu.findAll('[role="menuitem"]').map((item) => item.attributes('aria-label'))).toEqual([
			'Favourite',
			'Remove from playlist',
			'Original link',
			'Refresh SponsorBlock segments',
			'Reset progress',
			'Channel view'
		]);
		expect(menu.get('[data-testid="playlist-original-link"]').attributes('href')).toBe(
			ep.webpage_url
		);
		expect(menu.get('[data-testid="playlist-channel-view"]').attributes('to')).toBe(
			'[object Object]'
		);
		expect(menu.find('[aria-label="Stop"]').exists()).toBe(false);
	});

	it('runs favorite, removal, and reset actions through the shared behavior', async () => {
		const ep = episode({ id: 7, yt_id: 'yt7', position_seconds: 120 });
		const wrapper = mountPlaylistCard(ep);

		await wrapper.get('[data-testid="playlist-actions-trigger"]').trigger('click');
		await wrapper.get('[aria-label="Favourite"]').trigger('click');
		await flushPromises();
		expect(api.setEpisodeFavorite).toHaveBeenCalledWith('yt7', true);
		expect(useNotificationStore().current?.message).toBe('Added to favorites');

		await wrapper.get('[data-testid="playlist-actions-trigger"]').trigger('click');
		await wrapper.get('[aria-label="Reset progress"]').trigger('click');
		await flushPromises();
		expect(api.updateEpisodeProgress).toHaveBeenCalledWith('yt7', {
			position_seconds: 0,
			listened: false
		});
		expect(useNotificationStore().current?.message).toBe('Playback progress reset');

		await wrapper.get('[data-testid="playlist-actions-trigger"]').trigger('click');
		await wrapper.get('[role="menu"]').get('[aria-label="Remove from playlist"]').trigger('click');
		await flushPromises();
		expect(api.removeEpisodeFromPlaylist).toHaveBeenCalledWith(7);
		expect(useNotificationStore().current?.message).toBe('Removed from playlist');
	});

	it('refreshes SponsorBlock data for an old favorite and applies the live snapshot', async () => {
		const ep = episode({ id: 7, yt_id: 'yt7', favorite: true });
		const refreshed = {
			...ep,
			sponsorblock_hash: 'hash-b',
			sponsorblock_segments: [{ start: 10, end: 20, category: 'intro', rejected: false }]
		};
		vi.mocked(api.refreshEpisodeSponsorBlock).mockResolvedValueOnce({
			ok: true,
			data: refreshed,
			user: null,
			status: true
		});
		const wrapper = mountPlaylistCard(ep);
		const player = usePlayerStore();
		player.currentEpisode = ep;

		await wrapper.get('[data-testid="playlist-actions-trigger"]').trigger('click');
		await wrapper.get('[data-testid="playlist-refresh-sponsorblock"]').trigger('click');
		await flushPromises();

		expect(api.refreshEpisodeSponsorBlock).toHaveBeenCalledWith('yt7');
		expect(player.currentEpisode?.sponsorblock_hash).toBe('hash-b');
		expect(player.currentEpisode?.sponsorblock_segments).toEqual([
			{ start: 10, end: 20, category: 'intro', rejected: false }
		]);
		expect(useNotificationStore().current?.message).toBe('SponsorBlock segments refreshed');
	});

	it('provides non-empty Spanish labels for the trigger and menu items', async () => {
		testI18n.global.locale.value = 'es';
		const wrapper = mountPlaylistCard(episode());
		expect(wrapper.get('[data-testid="playlist-actions-trigger"]').attributes('aria-label')).toBe(
			'Acciones del episodio'
		);
		await wrapper.get('[data-testid="playlist-actions-trigger"]').trigger('click');
		expect(
			wrapper
				.get('[role="menu"]')
				.findAll('[role="menuitem"]')
				.map((item) => item.attributes('aria-label'))
		).toEqual([
			'Favorito',
			'Quitar de la playlist',
			'Enlace original',
			'Actualizar segmentos de SponsorBlock',
			'Reiniciar progreso',
			'Ver canal'
		]);
		testI18n.global.locale.value = 'en';
	});

	it('dismisses the menu with Escape and restores focus to its trigger', async () => {
		const wrapper = mountPlaylistCard(episode(), true);
		const trigger = wrapper.get('[data-testid="playlist-actions-trigger"]');
		await trigger.trigger('click');
		expect(trigger.attributes('aria-expanded')).toBe('true');

		document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
		await flushPromises();
		expect(wrapper.find('[role="menu"]').exists()).toBe(false);
		expect(trigger.attributes('aria-expanded')).toBe('false');
		expect(document.activeElement).toBe(trigger.element);
		wrapper.unmount();
	});
});

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

		// A non-current card's stop stays enabled: it is the "rewind this
		// episode" affordance and resets that episode's saved position via the
		// store (fix-stop-reset-scope).
		const stopBtn = wrapper.find('[aria-label="Stop"]');
		expect((stopBtn.element as HTMLButtonElement).disabled).toBe(false);
		await stopBtn.trigger('click');
		expect(spy).toHaveBeenCalledWith(expect.objectContaining({ id: 1 }));
	});

	it('does not render the progress strip without a saved position', () => {
		const wrapper = mountCard(episode({ position_seconds: 0 }));
		expect(wrapper.find('[data-testid="episode-progress"]').exists()).toBe(false);
	});

	it('shows all enabled SponsorBlock categories with category-aware colors', () => {
		const wrapper = mountCard(
			episode({
				position_seconds: 0,
				sponsorblock_segments: [
					{ start: 900, end: 1800, category: 'sponsor', rejected: true },
					{ start: 1200, end: 2100, category: 'intro', rejected: false }
				]
			})
		);
		const markers = wrapper.findAll('[data-testid="episode-sponsorblock-segment"]');
		expect(markers).toHaveLength(2);
		expect(markers[0].classes()).toContain('bg-sponsorblock');
		expect(markers[1].classes()).toContain('bg-sponsorblock-other');
		expect(markers[0].attributes('style')).toContain('left: 25%');
		expect(markers[0].attributes('style')).toContain('width: 25%');
	});

	it('hides markers and the refresh action when SponsorBlock is disabled', async () => {
		const wrapper = mountPlaylistCard(
			episode({
				sponsorblock_enabled: false,
				sponsorblock_segments: [{ start: 900, end: 1800, category: 'sponsor', rejected: true }]
			})
		);
		expect(wrapper.find('[data-testid="episode-sponsorblock-segment"]').exists()).toBe(false);
		await wrapper.get('[data-testid="playlist-actions-trigger"]').trigger('click');
		expect(wrapper.find('[data-testid="playlist-refresh-sponsorblock"]').exists()).toBe(false);
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

describe('EpisodeCard favorite toggle', () => {
	function mountCardWith(ep: Episode, seedFavorites: Episode[]) {
		const pinia = createPinia();
		const favorites = useFavoritesStore(pinia);
		for (const e of seedFavorites) favorites.sync(e);
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

	it('renders a filled star when the episode is favorited', () => {
		const wrapper = mountCardWith(episode({ id: 7, favorite: true }), [
			episode({ id: 7, favorite: true })
		]);
		expect(wrapper.find('[aria-label="Remove from favorites"]').exists()).toBe(true);
		expect(wrapper.find('[aria-label="Add to favorites"]').exists()).toBe(false);
	});

	it('renders a hollow star when the episode is not favorited', () => {
		const wrapper = mountCardWith(episode({ id: 1, favorite: false }), [
			episode({ id: 7, favorite: true })
		]);
		expect(wrapper.find('[aria-label="Add to favorites"]').exists()).toBe(true);
		expect(wrapper.find('[aria-label="Remove from favorites"]').exists()).toBe(false);
	});

	it('marking calls the api, flips the star and notifies', async () => {
		const wrapper = mountCardWith(episode({ id: 1, favorite: false }), []);
		await wrapper.find('[aria-label="Add to favorites"]').trigger('click');
		await flushPromises();
		expect(api.setEpisodeFavorite).toHaveBeenCalledWith('yt1', true);
		expect(wrapper.find('[aria-label="Remove from favorites"]').exists()).toBe(true);
		const notification = useNotificationStore();
		expect(notification.current?.message).toBe('Added to favorites');
	});

	it('unmarking calls the api, flips the star back and notifies', async () => {
		const target = episode({ id: 7, yt_id: 'yt7', favorite: true });
		const wrapper = mountCardWith(target, [target]);
		await wrapper.find('[aria-label="Remove from favorites"]').trigger('click');
		await flushPromises();
		expect(api.setEpisodeFavorite).toHaveBeenCalledWith('yt7', false);
		expect(wrapper.find('[aria-label="Add to favorites"]').exists()).toBe(true);
		const notification = useNotificationStore();
		expect(notification.current?.message).toBe('Removed from favorites');
	});

	it('keeps the star state and surfaces the error when the api fails', async () => {
		vi.mocked(api.setEpisodeFavorite).mockResolvedValue({
			ok: false,
			data: null,
			user: null,
			status: false
		} as never);
		const wrapper = mountCardWith(episode({ id: 1, favorite: false }), []);
		await wrapper.find('[aria-label="Add to favorites"]').trigger('click');
		await flushPromises();
		expect(wrapper.find('[aria-label="Add to favorites"]').exists()).toBe(true);
		const notification = useNotificationStore();
		expect(notification.current?.message).toBe('Could not update favorites');
	});
});
