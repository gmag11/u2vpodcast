import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useFavoritesStore } from '@/stores/favorites';
import type { Episode } from '@/types';
import { api } from '@/lib/api/client';

vi.mock('@/lib/api/client', () => ({
	api: {
		setEpisodeFavorite: vi.fn()
	}
}));

function episode(id: number, favorite: boolean): Episode {
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
		favorite,
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

describe('favorites store', () => {
	beforeEach(() => {
		setActivePinia(createPinia());
		vi.clearAllMocks();
	});

	it('sync merges loaded flags and drives the id set', () => {
		const store = useFavoritesStore();
		store.sync(episode(1, true));
		store.sync(episode(2, false));
		expect(store.favoriteIdSet.has(1)).toBe(true);
		expect(store.favoriteIdSet.has(2)).toBe(false);
	});

	it('set marks an episode via the api and updates local state on success', async () => {
		vi.mocked(api.setEpisodeFavorite).mockResolvedValue(okResult() as never);
		const store = useFavoritesStore();
		store.sync(episode(7, false));

		await store.set(episode(7, false), true);
		expect(api.setEpisodeFavorite).toHaveBeenCalledWith('yt7', true);
		expect(store.favoriteIdSet.has(7)).toBe(true);

		await store.set(episode(7, true), false);
		expect(api.setEpisodeFavorite).toHaveBeenCalledWith('yt7', false);
		expect(store.favoriteIdSet.has(7)).toBe(false);
	});

	it('set keeps local state when the api call fails', async () => {
		vi.mocked(api.setEpisodeFavorite).mockResolvedValue({
			ok: false,
			data: null,
			user: null,
			status: false
		} as never);
		const store = useFavoritesStore();
		store.sync(episode(9, false));

		await store.set(episode(9, false), true);
		expect(store.favoriteIdSet.has(9)).toBe(false);
	});
});
