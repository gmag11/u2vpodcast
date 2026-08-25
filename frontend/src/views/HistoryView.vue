<script setup lang="ts">
	import { computed, onMounted, ref } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhArrowLeft, PhRss } from '@phosphor-icons/vue';
	import { api, baseEndpoint } from '@/lib/api/client';
	import { useAuthStore } from '@/stores/auth';
	import { usePlayerStore } from '@/stores/player';
	import { filterBySearchWords } from '@/lib/utils/list.filter';
	import type { Episode } from '@/types';
	import AppHeader from '@/components/AppHeader.vue';
	import EpisodeCard from '@/components/EpisodeCard.vue';
	import SearchInput from '@/components/SearchInput.vue';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();
	const player = usePlayerStore();

	const episodes = ref<Episode[]>([]);
	const searchQuery = ref('');

	const filteredEpisodes = computed(() =>
		filterBySearchWords(episodes.value, searchQuery.value, (e) =>
			[e.title, e.description, e.yt_id].join(' ')
		)
	);

	const noSearchResults = computed(
		() => searchQuery.value.trim() !== '' && filteredEpisodes.value.length === 0
	);

	async function load() {
		const result = await api.getAllEpisodes();
		if (!result.ok || result.user == null) {
			auth.setUser(null);
			router.push({ name: 'login', query: { next: route.fullPath } });
			return;
		}
		auth.setUser(result.user);
		if (result.data) {
			episodes.value = result.data as Array<Episode>;
			// The list payload carries each episode's playback progress; seed the
			// player store so resume works without per-episode requests.
			player.seedProgress(episodes.value);
		}
	}

	onMounted(load);
</script>

<template>
	<AppHeader>
		<template #brand-icon>
			<svg
				class="h-5 w-5"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				viewbox="0 0 24 24"
				xmlns="http://www.w3.org/2000/svg"
			>
				<path
					d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
					stroke-linecap="round"
					stroke-linejoin="round"
				></path>
			</svg>
		</template>
	</AppHeader>

	<main class="flex min-h-screen flex-col items-center px-4 pb-28 pt-28">
		<div class="mb-8 flex w-full max-w-6xl items-center gap-4">
			<button
				type="button"
				aria-label="$t('header.backChannels')"
				class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-outline text-text-muted transition-colors hover:text-text"
				@click="router.push({ name: 'channels' })"
			>
				<PhArrowLeft class="h-5 w-5" weight="regular" />
			</button>
			<h1 class="truncate font-display text-2xl font-semibold text-text">
				{{ $t('history.title') }} ({{ episodes.length }})
			</h1>
			<a
				:href="`${baseEndpoint}/feed.xml`"
				target="_blank"
				rel="noopener noreferrer"
				:aria-label="$t('history.rssTooltip')"
				class="group relative ml-auto flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-outline text-accent-500 transition-colors hover:text-accent-400"
			>
				<PhRss class="h-5 w-5" weight="regular" />
				<span
					class="pointer-events-none absolute bottom-full right-0 z-20 mb-2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					{{ $t('history.rssTooltip') }}
				</span>
			</a>
		</div>

		<div class="mb-10 w-full max-w-6xl">
			<SearchInput v-model="searchQuery" :placeholder="$t('history.searchPlaceholder')" />
		</div>

		<p v-if="noSearchResults" class="mt-4 text-text-muted">{{ $t('common.noResults') }}</p>

		<div v-else-if="filteredEpisodes.length === 0" class="mt-10 text-center">
			<p class="font-display text-xl font-semibold text-text">{{ $t('history.emptyTitle') }}</p>
			<p class="mt-2 text-sm text-text-muted">
				{{ $t('history.emptyBody') }}
			</p>
		</div>

		<div v-else class="flex w-full max-w-6xl flex-col gap-5">
			<EpisodeCard
				v-for="episode in filteredEpisodes"
				:key="episode.id"
				:episode="episode"
				:list="filteredEpisodes"
				compact
			/>
		</div>
	</main>
</template>
