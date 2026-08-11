<script setup lang="ts">
	import { computed, onMounted, ref } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhArrowsClockwise, PhMicrophoneStage } from '@phosphor-icons/vue';
	import { api } from '@/lib/api/client';
	import { useAuthStore } from '@/stores/auth';
	import { useNotificationStore } from '@/stores/notification';
	import { filterBySearchWords } from '@/lib/utils/list.filter';
	import type { Channel, Episode } from '@/types';
	import AppButton from '@/components/AppButton.vue';
	import AppHeader from '@/components/AppHeader.vue';
	import EpisodeCard from '@/components/EpisodeCard.vue';
	import SearchInput from '@/components/SearchInput.vue';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();
	const notification = useNotificationStore();

	const episodes = ref<Episode[]>([]);
	const searchQuery = ref('');
	const refreshing = ref(false);

	const filteredEpisodes = computed(() =>
		filterBySearchWords(episodes.value, searchQuery.value, (e) =>
			[e.title, e.description, e.yt_id].join(' ')
		)
	);

	const noSearchResults = computed(
		() => searchQuery.value.trim() !== '' && filteredEpisodes.value.length === 0
	);

	const channelSlug = computed(() => {
		if (episodes.value.length > 0) return episodes.value[0].channel_slug;
		return '';
	});

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

	async function resolveSlugFallback(): Promise<string> {
		if (channelSlug.value) return channelSlug.value;
		const channelId = Number(route.params.channelId);
		const result = await api.getChannels();
		if (!result.ok || !result.data) return '';
		const channel = (result.data as Array<Channel>).find((c) => c.id === channelId);
		return channel?.slug ?? '';
	}

	async function refreshChannel() {
		const slug = await resolveSlugFallback();
		if (!slug) {
			notification.show('Unable to identify the channel', 'error');
			return;
		}
		refreshing.value = true;
		try {
			const result = await api.refreshChannel(slug);
			if (result.ok) {
				notification.show('Channel update started', 'success');
			} else {
				notification.show(result.message || 'Failed to start channel update', 'error');
			}
		} catch (err) {
			console.error(err);
			notification.show('Failed to start channel update', 'error');
		} finally {
			refreshing.value = false;
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
			<AppButton variant="secondary" type="button" :disabled="refreshing" @click="refreshChannel">
				<PhArrowsClockwise
					class="h-4 w-4"
					weight="regular"
					:class="refreshing ? 'animate-spin' : ''"
				/>
				Refresh
			</AppButton>
		</template>
	</AppHeader>

	<main class="flex min-h-screen flex-col items-center px-4 pb-28 pt-28">
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
