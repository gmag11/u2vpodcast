<script setup lang="ts">
	import { computed, onMounted, ref } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhMicrophoneStage } from '@phosphor-icons/vue';
	import { api } from '@/lib/api/client';
	import { useAuthStore } from '@/stores/auth';
	import { filterBySearchWords } from '@/lib/utils/list.filter';
	import type { Episode } from '@/types';
	import AppHeader from '@/components/AppHeader.vue';
	import EpisodeCard from '@/components/EpisodeCard.vue';
	import SearchInput from '@/components/SearchInput.vue';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();

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
		const channelId = Number(route.params.channelId);
		const result = await api.getEpisodes(channelId);
		if (!result.ok || result.user == null) {
			auth.setUser(null);
			router.push({ name: 'login', query: { next: route.fullPath } });
			return;
		}
		auth.setUser(result.user);
		if (result.data) {
			episodes.value = result.data as Array<Episode>;
		}
	}

	onMounted(load);
</script>

<template>
	<AppHeader>
		<template #brand-icon>
			<PhMicrophoneStage class="h-5 w-5" weight="fill" />
		</template>
		<template #actions>
			<div class="hidden md:block"></div>
		</template>
	</AppHeader>

	<main class="flex min-h-screen flex-col items-center px-4 pb-20 pt-28">
		<div class="mb-10 w-full max-w-3xl">
			<SearchInput v-model="searchQuery" placeholder="Search episodes…" />
		</div>

		<p v-if="noSearchResults" class="mt-4 text-text-muted">No results match your search.</p>

		<div v-else-if="filteredEpisodes.length === 0" class="mt-10 text-center">
			<p class="font-display text-xl font-semibold text-text">No episodes yet</p>
			<p class="mt-2 text-sm text-text-muted">
				The channel is being processed and episodes will appear here as they are downloaded.
			</p>
		</div>

		<div v-else class="flex w-full max-w-3xl flex-col gap-5">
			<EpisodeCard v-for="episode in filteredEpisodes" :key="episode.id" :episode="episode" />
		</div>
	</main>
</template>
