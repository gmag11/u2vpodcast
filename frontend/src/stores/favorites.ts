import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import type { Episode } from '@/types';
import { api } from '@/lib/api/client';

export const useFavoritesStore = defineStore('favorites', () => {
	// Per-episode favorite flags keyed by episode id. Populated from loaded
	// episode payloads (`sync`); `favoriteIdSet` drives the card stars and the
	// favorites-only filters without per-card server queries. Mutating the Map
	// is reactive because `ref` wraps the value with `reactive`, whose
	// collection handlers track Map reads/writes.
	const byId = ref<Map<number, boolean>>(new Map());

	// A toggle must reach the API by the episode's public identity (`yt_id`),
	// the same identity the progress/media endpoints use, so `set` takes the
	// whole episode instead of a bare id.
	const favoriteIdSet = computed(() => {
		const set = new Set<number>();
		for (const [id, favorite] of byId.value) {
			if (favorite) set.add(id);
		}
		return set;
	});

	// Merges a loaded episode's flag so every rendered copy of an episode
	// agrees with the server; unknown episodes are simply absent until seen.
	function sync(episode: Episode) {
		byId.value.set(episode.id, episode.favorite);
	}

	async function set(episode: Episode, favorite: boolean) {
		const result = await api.setEpisodeFavorite(episode.yt_id, favorite);
		if (result.ok) {
			byId.value.set(episode.id, favorite);
		}
		return result;
	}

	return { byId, favoriteIdSet, sync, set };
});
