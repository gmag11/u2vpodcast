<script setup lang="ts">
	import { ref } from 'vue';
	import { PhLinkSimple, PhPlay, PhSpeakerHigh } from '@phosphor-icons/vue';
	import { toHHMMSS } from '@/lib/utils/formatter';
	import type { Episode } from '@/types';

	const props = defineProps<{
		episode: Episode;
	}>();

	const playing = ref(false);

	function formatDate(value: Date | string) {
		return new Date(value).toLocaleDateString('en-US');
	}

	function formatDuration(raw: string | number | null) {
		if (raw == null) return '';
		const seconds = typeof raw === 'number' ? raw : Number(raw);
		if (isNaN(seconds) || seconds <= 0) return '1s';
		return toHHMMSS(seconds) || '1s';
	}

	function togglePlay() {
		playing.value = !playing.value;
	}
</script>

<template>
	<article
		class="flex flex-col gap-4 rounded-xl border border-outline bg-surface-card p-5 shadow-card"
	>
		<div class="flex flex-col gap-5 sm:flex-row sm:items-start">
			<div class="h-28 w-full shrink-0 overflow-hidden rounded-lg bg-surface-input sm:w-48">
				<img
					v-if="props.episode.image"
					:src="props.episode.image"
					:alt="props.episode.title"
					class="h-full w-full object-cover"
				/>
			</div>
			<div class="flex flex-col gap-1.5">
				<h2
					class="text-base font-bold uppercase leading-tight tracking-wide text-text line-clamp-2"
				>
					{{ props.episode.title }}
				</h2>
				<time class="text-sm text-text-muted">
					{{ formatDate(props.episode.published_at) }}
				</time>
				<p class="mt-1 line-clamp-2 text-sm text-text-muted">{{ props.episode.description }}</p>
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

		<div class="mt-2 flex items-center gap-4 px-1">
			<button
				type="button"
				class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
				:aria-label="playing ? 'Pause' : 'Play'"
				@click="togglePlay"
			>
				<PhPlay v-if="!playing" class="ml-0.5 h-4 w-4 text-white" weight="fill" />
				<span v-else class="ml-0.5 h-3 w-3 rounded-[2px] bg-white"></span>
			</button>
			<div class="relative h-1.5 flex-1 overflow-hidden rounded-full bg-surface-input">
				<div
					class="absolute left-0 top-0 h-full rounded-full bg-accent-400 transition-all duration-300"
					:class="playing ? 'w-1/4' : 'w-1/12'"
				></div>
			</div>
			<div class="flex shrink-0 items-center gap-2 text-sm text-text-muted">
				<PhSpeakerHigh class="h-4 w-4" weight="regular" />
				<span>{{ formatDuration(props.episode.duration) }}</span>
			</div>
		</div>
	</article>
</template>
