<script setup lang="ts">
	import { computed, ref } from 'vue';
	import {
		PhGauge,
		PhLinkSimple,
		PhPause,
		PhPlay,
		PhSpeakerHigh,
		PhSpeakerSlash,
		PhStop
	} from '@phosphor-icons/vue';
	import { usePlayerStore } from '@/stores/player';
	import type { Episode } from '@/types';

	const props = defineProps<{
		episode: Episode;
	}>();

	const player = usePlayerStore();
	const showSpeed = ref(false);
	const speeds = [0.5, 1, 1.25, 1.5, 2];

	const isCurrent = computed(() => player.isCurrent(props.episode));
	const isPlaying = computed(() => isCurrent.value && player.playing);

	function onSeek(event: MouseEvent) {
		if (!isCurrent.value || player.duration <= 0) return;
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const ratio = Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
		player.seek(ratio * player.duration);
	}

	function onVolumeInput(event: Event) {
		player.setVolume(Number((event.target as HTMLInputElement).value));
	}

	function formatDate(value: Date | string) {
		return new Date(value).toLocaleDateString('en-US');
	}
</script>

<template>
	<article
		class="flex flex-col gap-4 rounded-xl border border-outline bg-surface-card p-5 shadow-card"
		:class="isCurrent ? 'border-accent-500/60' : ''"
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

			<div
				class="relative h-5 flex-1 cursor-pointer py-2"
				role="slider"
				aria-label="Seek"
				:aria-valuenow="Math.round(isCurrent ? player.progress : 0)"
				:aria-valuemin="0"
				:aria-valuemax="100"
				@click="onSeek"
			>
				<div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-input">
					<div
						class="h-full rounded-full bg-accent-400 transition-[width] duration-150"
						:style="{ width: (isCurrent ? player.progress : 0) + '%' }"
					></div>
				</div>
			</div>

			<div class="flex shrink-0 items-center gap-2 text-sm text-text-muted">
				<span v-if="isCurrent">{{ player.currentLabel }} / {{ player.durationLabel }}</span>
				<span v-else>{{ props.episode.duration }}</span>
			</div>
		</div>

		<div class="flex items-center justify-between px-1">
			<div class="relative">
				<button
					type="button"
					class="flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-text-muted transition-colors hover:text-text"
					:disabled="!isCurrent"
					@click="showSpeed = !showSpeed"
				>
					<PhGauge class="h-4 w-4" weight="regular" />
					{{ isCurrent ? player.speed : 1 }}x
				</button>
				<div
					v-if="showSpeed && isCurrent"
					class="absolute bottom-full left-0 z-10 mb-2 flex flex-col rounded-lg border border-outline bg-surface-card p-1 shadow-card"
				>
					<button
						v-for="s in speeds"
						:key="s"
						type="button"
						class="rounded-md px-3 py-1.5 text-left text-xs font-medium transition-colors"
						:class="
							s === player.speed ? 'bg-accent-600 text-white' : 'text-text-muted hover:text-text'
						"
						@click="
							player.setSpeed(s);
							showSpeed = false;
						"
					>
						{{ s }}x
					</button>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<button
					type="button"
					class="rounded-md p-1 text-text-muted transition-colors hover:text-text"
					:aria-label="player.muted ? 'Unmute' : 'Mute'"
					:disabled="!isCurrent"
					@click="player.toggleMute()"
				>
					<PhSpeakerSlash v-if="player.muted" class="h-4 w-4" weight="regular" />
					<PhSpeakerHigh v-else class="h-4 w-4" weight="regular" />
				</button>
				<input
					class="h-1 w-20 cursor-pointer accent-accent-500 disabled:opacity-40"
					type="range"
					min="0"
					max="1"
					step="0.05"
					:value="player.volume"
					:disabled="!isCurrent"
					@input="onVolumeInput"
				/>
			</div>
		</div>
	</article>
</template>
