import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { usePlaylistStore } from '@/stores/playlists';
import type { Episode } from '@/types';
import { api } from '@/lib/api/client';

vi.mock('@/lib/api/client', () => ({
	api: {
		getPlaylist: vi.fn(),
		addEpisodeToPlaylist: vi.fn(),
		removeEpisodeFromPlaylist: vi.fn(),
		reorderPlaylist: vi.fn()
	}
}));

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
		position_seconds: 0,
		listened_at: null,
		favorite: false,
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

describe('playlist store', () => {
	beforeEach(() => {
		setActivePinia(createPinia());
		vi.clearAllMocks();
	});

	it('load populates items and the id set', async () => {
		vi.mocked(api.getPlaylist).mockResolvedValue(okResult([episode(1), episode(2)]) as never);
		const store = usePlaylistStore();
		await store.load();
		expect(store.items.map((e) => e.id)).toEqual([1, 2]);
		expect(store.episodeIdSet.has(1)).toBe(true);
		expect(store.episodeIdSet.has(3)).toBe(false);
	});

	it('add sends the episode id and reloads into the tail', async () => {
		vi.mocked(api.addEpisodeToPlaylist).mockResolvedValue(okResult() as never);
		vi.mocked(api.getPlaylist).mockResolvedValue(okResult([episode(1), episode(2)]) as never);
		const store = usePlaylistStore();
		store.items = [episode(1)];
		await store.add(2);
		expect(api.addEpisodeToPlaylist).toHaveBeenCalledWith(2);
		expect(store.items.map((e) => e.id)).toEqual([1, 2]);
		expect(store.episodeIdSet.has(2)).toBe(true);
	});

	it('add skips already-present items without calling the api', async () => {
		const store = usePlaylistStore();
		store.items = [episode(1)];
		const result = await store.add(1);
		expect(result.ok).toBe(true);
		expect(api.addEpisodeToPlaylist).not.toHaveBeenCalled();
		expect(store.items).toHaveLength(1);
	});

	it('remove deletes the episode locally on success', async () => {
		vi.mocked(api.removeEpisodeFromPlaylist).mockResolvedValue(okResult() as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		await store.remove(2);
		expect(api.removeEpisodeFromPlaylist).toHaveBeenCalledWith(2);
		expect(store.items.map((e) => e.id)).toEqual([1, 3]);
		expect(store.episodeIdSet.has(2)).toBe(false);
	});

	it('remove keeps the local items when the removal fails', async () => {
		vi.mocked(api.removeEpisodeFromPlaylist).mockResolvedValue({
			ok: false,
			data: null,
			user: null,
			status: false
		} as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2)];
		await store.remove(2);
		expect(store.items.map((e) => e.id)).toEqual([1, 2]);
	});

	it('reorder rewrites the local order in the submitted sequence', async () => {
		vi.mocked(api.reorderPlaylist).mockResolvedValue(okResult() as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		await store.reorder([3, 1, 2]);
		expect(api.reorderPlaylist).toHaveBeenCalledWith([3, 1, 2]);
		expect(store.items.map((e) => e.id)).toEqual([3, 1, 2]);
	});
});
