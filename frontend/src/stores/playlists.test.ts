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

const failedResult = () => ({ ok: false, data: null, user: null, status: false });

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((done) => {
		resolve = done;
	});
	return { promise, resolve };
}

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

	it('reorder applies the complete order optimistically and persists it', async () => {
		const request = deferred<ReturnType<typeof okResult>>();
		vi.mocked(api.reorderPlaylist).mockReturnValue(request.promise as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		const result = store.reorder([3, 1, 2]);
		await vi.waitFor(() => expect(store.reorderPending).toBe(true));
		expect(store.items.map((e) => e.id)).toEqual([3, 1, 2]);
		expect(api.reorderPlaylist).toHaveBeenCalledWith([3, 1, 2]);
		request.resolve(okResult());
		await result;
		expect(store.items.map((e) => e.id)).toEqual([3, 1, 2]);
		expect(store.reorderPending).toBe(false);
	});

	it('reorder restores the previous order when persistence fails', async () => {
		vi.mocked(api.reorderPlaylist).mockResolvedValue(failedResult() as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		const result = await store.reorder([3, 1, 2]);
		expect(result.ok).toBe(false);
		expect(store.items.map((e) => e.id)).toEqual([1, 2, 3]);
	});

	it('reorder reloads instead of reviving an item removed while pending', async () => {
		const request = deferred<ReturnType<typeof failedResult>>();
		vi.mocked(api.reorderPlaylist).mockReturnValue(request.promise as never);
		vi.mocked(api.removeEpisodeFromPlaylist).mockResolvedValue(okResult() as never);
		vi.mocked(api.getPlaylist).mockResolvedValue(okResult([episode(3), episode(1)]) as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		const reorderResult = store.reorder([3, 1, 2]);
		await vi.waitFor(() => expect(store.reorderPending).toBe(true));
		await store.remove(2);
		request.resolve(failedResult());
		await reorderResult;
		expect(api.getPlaylist).toHaveBeenCalledOnce();
		expect(store.items.map((e) => e.id)).toEqual([3, 1]);
	});

	it('reorder serializes consecutive persistence requests', async () => {
		const first = deferred<ReturnType<typeof okResult>>();
		vi.mocked(api.reorderPlaylist)
			.mockReturnValueOnce(first.promise as never)
			.mockResolvedValueOnce(okResult() as never);
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		const firstResult = store.reorder([3, 1, 2]);
		const secondResult = store.reorder([2, 3, 1]);
		await vi.waitFor(() => expect(api.reorderPlaylist).toHaveBeenCalledTimes(1));
		first.resolve(okResult());
		await firstResult;
		await secondResult;
		expect(api.reorderPlaylist).toHaveBeenNthCalledWith(2, [2, 3, 1]);
		expect(store.items.map((e) => e.id)).toEqual([2, 3, 1]);
	});

	it('reorder rejects an incomplete order without calling the api', async () => {
		const store = usePlaylistStore();
		store.items = [episode(1), episode(2), episode(3)];
		const result = await store.reorder([3, 1]);
		expect(result.ok).toBe(false);
		expect(api.reorderPlaylist).not.toHaveBeenCalled();
		expect(store.items.map((e) => e.id)).toEqual([1, 2, 3]);
	});
});
