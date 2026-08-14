<script setup lang="ts">
	import { computed } from 'vue';
	import { PhLinkSimple, PhPause, PhPlay, PhStop } from '@phosphor-icons/vue';
	import { usePlayerStore } from '@/stores/player';
	import type { Episode } from '@/types';

	const props = withDefaults(
		defineProps<{
			episode: Episode;
			compact?: boolean;
		}>(),
		{
			compact: false
		}
	);

	const player = usePlayerStore();

	const isCurrent = computed(() => player.isCurrent(props.episode));
	const isPlaying = computed(() => isCurrent.value && player.playing);
	const durationLabel = computed(() =>
		isCurrent.value ? player.durationLabel : props.episode.duration
	);

	function formatDate(value: Date | string) {
		return new Date(value).toLocaleDateString('en-US');
	}
</script>

<template>
	<article
		class="flex flex-col gap-4 rounded-xl border border-outline bg-surface-card shadow-card"
		:class="[isCurrent ? 'border-accent-500/60' : '', compact ? 'p-4' : 'p-5']"
	>
		<div class="flex flex-col gap-5 sm:flex-row sm:items-start">
			<div class="flex items-start gap-3 sm:flex-col sm:gap-3">
				<div
					class="shrink-0 overflow-hidden rounded-lg bg-surface-input"
					:class="compact ? 'h-20 w-28' : 'h-28 w-48'"
				>
					<img
						v-if="props.episode.image"
						:src="props.episode.image"
						:alt="props.episode.title"
						class="h-full w-full object-cover"
					/>
				</div>

				<div class="flex shrink-0 items-center gap-1.5 sm:hidden">
					<button
						type="button"
						class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
						:aria-label="isPlaying ? 'Pause' : 'Play'"
						:disabled="isCurrent && player.loading"
						@click="isCurrent ? player.togglePlay() : player.play(props.episode)"
					>
						<PhPause v-if="isPlaying" class="h-4 w-4 text-white" weight="fill" />
						<PhPlay v-else class="ml-0.5 h-4 w-4 text-white" weight="fill" />
					</button>

					<button
						type="button"
						class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text"
						aria-label="Stop"
						:disabled="!isCurrent"
						@click="player.stop()"
					>
						<PhStop class="h-4 w-4" weight="fill" />
					</button>
				</div>

				<div class="hidden gap-2 sm:flex">
					<button
						type="button"
						class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
						:aria-label="isPlaying ? 'Pause' : 'Play'"
						:disabled="isCurrent && player.loading"
						@click="isCurrent ? player.togglePlay() : player.play(props.episode)"
					>
						<PhPause v-if="isPlaying" class="h-4 w-4 text-white" weight="fill" />
						<PhPlay v-else class="ml-0.5 h-4 w-4 text-white" weight="fill" />
					</button>

					<button
						type="button"
						class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text"
						aria-label="Stop"
						:disabled="!isCurrent"
						@click="player.stop()"
					>
						<PhStop class="h-4 w-4" weight="fill" />
					</button>
				</div>
			</div>

			<div class="flex flex-col gap-1.5">
				<RouterLink
					v-if="compact && props.episode.channel_title"
					:to="{ name: 'episodes', params: { channelId: String(props.episode.channel_id) } }"
					class="w-max text-xs font-medium uppercase tracking-wide text-accent-500 hover:underline"
				>
					{{ props.episode.channel_title }}
				</RouterLink>
				<h2
					class="text-base font-bold uppercase leading-tight tracking-wide text-text line-clamp-2"
				>
					{{ props.episode.title }}
				</h2>
				<time class="text-sm text-text-muted">
					{{ formatDate(props.episode.published_at) }}
				</time>
				<span class="text-sm text-text-muted">{{ durationLabel }}</span>
				<p class="mt-1 line-clamp-2 text-sm text-text-muted">
					{{ props.episode.description }}
				</p>
				<a
					class="mt-1 inline-flex w-max items-center gap-1.5 text-sm text-accent-500 hover:underline"
					:href="props.episode.webpage_url"
					target="_blank"
					rel="noopener noreferrer"
				>
					<PhLinkSimple class="h-4 w-4" weight="regular" />
					YouTube
				</a>
			</div>
		</div>
	</article>
</template>
