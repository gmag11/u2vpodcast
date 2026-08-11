<script setup lang="ts">
	import { computed, onMounted, ref } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhArrowLeft, PhArrowsClockwise, PhMicrophoneStage } from '@phosphor-icons/vue';
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
	const channels = ref<Channel[]>([]);
	const channelTitle = ref('');
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
		const [episodesResult, channelsResult] = await Promise.all([
			api.getEpisodes(channelId),
			api.getChannels()
		]);
		if (!episodesResult.ok || episodesResult.user == null) {
			auth.setUser(null);
			router.push({ name: 'login', query: { next: route.fullPath } });
			return;
		}
		auth.setUser(episodesResult.user);
		if (episodesResult.data) {
			episodes.value = episodesResult.data as Array<Episode>;
		}
		if (channelsResult.ok && channelsResult.data) {
			channels.value = channelsResult.data as Array<Channel>;
		}
		const channel = channels.value.find((c) => c.id === channelId);
		channelTitle.value = channel?.title ?? 'Episodes';
	}

	async function resolveSlugFallback(): Promise<string> {
		if (channelSlug.value) return channelSlug.value;
		const channel = channels.value.find((c) => c.id === Number(route.params.channelId));
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
		<div class="mb-8 flex w-full max-w-3xl items-center gap-4">
			<button
				type="button"
				aria-label="Back to channels"
				class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-outline text-text-muted transition-colors hover:text-text"
				@click="router.push({ name: 'channels' })"
			>
				<PhArrowLeft class="h-5 w-5" weight="regular" />
			</button>
			<h1 class="truncate font-display text-2xl font-semibold text-text">
				{{ channelTitle }}
			</h1>
		</div>

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
