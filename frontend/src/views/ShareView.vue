<script setup lang="ts">
	import { computed, onMounted, ref, watch } from 'vue';
	import { useRoute } from 'vue-router';
	import { useI18n } from 'vue-i18n';
	import { api } from '@/lib/api/client';
	import type { SharedEpisode } from '@/types';

	// Public, stateless share page. It must never assume an authenticated
	// context: no session, no persistent player, no progress.
	const route = useRoute();
	const { t } = useI18n();

	const episode = ref<SharedEpisode | null>(null);
	const loading = ref(true);
	const unavailable = ref(false);

	const token = computed(() => String(route.params.token ?? ''));

	async function load() {
		loading.value = true;
		unavailable.value = false;
		episode.value = null;
		const result = await api.getSharedEpisode(token.value);
		if (result) episode.value = result;
		else unavailable.value = true;
		loading.value = false;
	}

	onMounted(load);
	watch(token, load);
</script>

<template>
	<div class="flex min-h-screen items-center justify-center p-4">
		<main
			class="w-full max-w-2xl rounded-2xl border border-outline bg-surface-card p-6 shadow-card sm:p-8"
		>
			<p v-if="loading" class="text-center text-sm text-text-muted" data-testid="share-loading">
				{{ t('share.loading') }}
			</p>

			<section
				v-else-if="unavailable"
				class="flex flex-col items-center gap-3 text-center"
				data-testid="share-unavailable"
			>
				<h1 class="text-2xl font-bold text-text">{{ t('share.unavailableTitle') }}</h1>
				<p class="text-sm text-text-muted">{{ t('share.unavailableBody') }}</p>
			</section>

			<article v-else-if="episode" class="flex flex-col gap-5" data-testid="share-episode">
				<div class="flex flex-col gap-4 sm:flex-row">
					<img
						v-if="episode.image"
						:src="episode.image"
						alt=""
						class="h-40 w-full rounded-lg object-cover sm:w-56"
					/>
					<div class="flex min-w-0 flex-col gap-1">
						<p
							v-if="episode.channel_title"
							class="text-xs font-medium uppercase tracking-wide text-accent-500"
							data-testid="share-channel"
						>
							{{ episode.channel_title }}
						</p>
						<h1 class="text-xl font-bold text-text" data-testid="share-title">
							{{ episode.title }}
						</h1>
						<p class="whitespace-pre-line text-sm text-text-muted">
							{{ episode.description }}
						</p>
					</div>
				</div>
				<audio controls :src="episode.audio_url" class="w-full" data-testid="share-audio">
					{{ t('share.audioUnsupported') }}
				</audio>
			</article>
		</main>
	</div>
</template>
