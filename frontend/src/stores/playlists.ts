import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { Episode } from '@/types';
import { api, type ApiResult } from '@/lib/api/client';

const DEFENSIVE_OK: ApiResult<unknown> = { ok: true, data: null, user: null, status: true };

export const usePlaylistStore = defineStore('playlists', () => {
	const items = ref<Episode[]>([]);
	const loaded = ref(false);

	// The id set drives the card toggles: an episode is either pending (in the
	// playlist) or not, without each card querying the server.
	const episodeIdSet = computed(() => new Set(items.value.map((episode) => episode.id)));

	async function load() {
		let result: ApiResult<Array<Episode>>;
		try {
			result = await api.getPlaylist();
		} catch {
			// A failed fetch (offline, session dropped) must never leave the
			// store half-initialised nor reject through App-level callers on
			// boot; the cards simply stay in their "not in playlist" state
			// until a later successful load.
			return { ok: false, data: null, user: null, status: false };
		}
		if (result.ok && result.data) {
			items.value = result.data as Array<Episode>;
			loaded.value = true;
		}
		return result;
	}

	async function add(episodeId: number) {
		// Defensive: never ask the server to duplicate an already-present item
		// (the single-playlist invariant, enforced server-side too).
		if (episodeIdSet.value.has(episodeId)) return { ...DEFENSIVE_OK };
		const result = await api.addEpisodeToPlaylist(episodeId);
		if (result.ok) {
			// Reload so the store holds the exact server order (the append lands
			// at the playlist's end) plus the joined channel fields.
			await load();
		}
		return result;
	}

	async function remove(episodeId: number) {
		const result = await api.removeEpisodeFromPlaylist(episodeId);
		if (result.ok) {
			items.value = items.value.filter((episode) => episode.id !== episodeId);
		}
		return result;
	}

	async function reorder(episodeIds: number[]) {
		const result = await api.reorderPlaylist(episodeIds);
		if (result.ok) {
			const byId = new Map(items.value.map((episode) => [episode.id, episode]));
			items.value = episodeIds
				.map((id) => byId.get(id))
				.filter((episode) => episode != null) as Episode[];
		}
		return result;
	}

	return { items, loaded, episodeIdSet, load, add, remove, reorder };
});
