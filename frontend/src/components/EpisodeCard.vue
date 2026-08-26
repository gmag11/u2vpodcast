<script setup lang="ts">
	import { computed } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { PhLinkSimple, PhPause, PhPlay, PhStop } from '@phosphor-icons/vue';
	import { usePlayerStore, RESUME_POSITION_S, parseDurationSeconds } from '@/stores/player';
	import type { Episode } from '@/types';
	import { toHHMMSS } from '@/lib/utils/formatter';

	const props = withDefaults(
		defineProps<{
			episode: Episode;
			compact?: boolean;
			list?: Episode[];
		}>(),
		{
			compact: false,
			list: undefined
		}
	);

	const player = usePlayerStore();
	const { d } = useI18n();

	const isCurrent = computed(() => player.isCurrent(props.episode));
	const isPlaying = computed(() => isCurrent.value && player.playing);
	const durationLabel = computed(() =>
		isCurrent.value ? player.durationLabel : props.episode.duration
	);

	// Progress indicators reflect the shared player's per-id progress, so a
	// card updates live without a reload no matter which copy of the episode
	// is rendered (playback-progress).
	const liveEpisode = computed(() =>
		player.episodeWithProgress(
			isCurrent.value && player.currentEpisode ? player.currentEpisode : props.episode
		)
	);
	const hasPlayedMark = computed(() => liveEpisode.value.listen);
	const resumeSeconds = computed(() =>
		!liveEpisode.value.listen && liveEpisode.value.position_seconds > RESUME_POSITION_S
			? liveEpisode.value.position_seconds
			: 0
	);
	const resumeLabel = computed(() =>
		resumeSeconds.value > 0 ? toHHMMSS(resumeSeconds.value) : ''
	);
	const canStartOver = computed(() => isCurrent.value && resumeSeconds.value > 0);

	// Fraction (0-100) of the saved playback position against the published
	// duration, used by the read-only progress strip. The current episode's
	// strip tracks the live playhead instead, so it evolves during playback
	// (playback-progress).
	const savedProgress = computed(() => {
		const total = parseDurationSeconds(props.episode.duration);
		if (!total || total <= 0) return 0;
		const pos = liveEpisode.value.position_seconds;
		return Math.min(Math.max((pos / total) * 100, 0), 100);
	});
	const progressRatio = computed(() =>
		isCurrent.value && !player.stopped ? player.progress : savedProgress.value
	);

	function formatDate(value: Date | string) {
		return d(new Date(value), 'short');
	}
</script>

<template>
	<article
		class="relative flex flex-col gap-4 overflow-hidden rounded-xl border border-outline bg-surface-card shadow-card"
		:class="[isCurrent ? 'border-accent-500/60' : '', compact ? 'p-4' : 'p-5']"
	>
		<!-- Played mark: the card's top-right corner is tinted green -->
		<span
			v-if="hasPlayedMark"
			class="absolute right-0 top-0"
			data-testid="listened-mark"
			role="img"
			:aria-label="$t('card.listened')"
		>
			<svg class="h-7 w-7 text-success" viewBox="0 0 24 24" aria-hidden="true">
				<path d="M0 0 L24 0 L24 24 Z" fill="currentColor" />
			</svg>
		</span>
		<div class="flex flex-1 flex-col gap-5 sm:flex-row sm:items-start">
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

				<div class="flex shrink-0 flex-col items-start gap-1.5 sm:hidden">
					<div class="flex items-center gap-1.5">
						<button
							type="button"
							class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
							:aria-label="isPlaying ? $t('player.pause') : $t('player.play')"
							:disabled="isCurrent && player.loading"
							@click="isCurrent ? player.togglePlay() : player.play(props.episode, props.list)"
						>
							<PhPause v-if="isPlaying" class="h-4 w-4 text-white" weight="fill" />
							<PhPlay v-else class="ml-0.5 h-4 w-4 text-white" weight="fill" />
						</button>

						<button
							type="button"
							class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text"
							:aria-label="$t('player.stop')"
							:disabled="isCurrent && player.loading"
							@click="player.stop(props.episode)"
						>
							<PhStop class="h-4 w-4" weight="fill" />
						</button>
					</div>
					<span class="text-sm text-text-muted">{{ durationLabel }}</span>
				</div>

				<div class="hidden w-full items-center justify-between gap-2 sm:flex">
					<button
						type="button"
						class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
						:aria-label="isPlaying ? $t('player.pause') : $t('player.play')"
						:disabled="isCurrent && player.loading"
						@click="isCurrent ? player.togglePlay() : player.play(props.episode, props.list)"
					>
						<PhPause v-if="isPlaying" class="h-4 w-4 text-white" weight="fill" />
						<PhPlay v-else class="ml-0.5 h-4 w-4 text-white" weight="fill" />
					</button>

					<button
						type="button"
						class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text"
						:aria-label="$t('player.stop')"
						:disabled="isCurrent && player.loading"
						@click="player.stop(props.episode)"
					>
						<PhStop class="h-4 w-4" weight="fill" />
					</button>

					<span class="ml-auto text-sm text-text-muted">{{ durationLabel }}</span>
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
				<p class="mt-1 line-clamp-2 text-sm text-text-muted">
					{{ props.episode.description }}
				</p>
				<div
					v-if="resumeSeconds > 0"
					class="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1"
				>
					<span class="text-xs text-text-muted">
						{{ $t('card.continueAt', { time: resumeLabel }) }}
					</span>
					<button
						v-if="canStartOver"
						type="button"
						class="inline-flex items-center text-xs text-accent-500 transition-colors hover:underline"
						@click="player.play(props.episode, props.list, { fromStart: true })"
					>
						{{ $t('card.startOver') }}
					</button>
				</div>
				<div class="mt-1 flex w-full items-center justify-between gap-2">
					<a
						class="inline-flex w-max items-center gap-1.5 text-sm text-accent-500 hover:underline"
						:href="props.episode.webpage_url"
						target="_blank"
						rel="noopener noreferrer"
					>
						<PhLinkSimple class="h-4 w-4" weight="regular" />
						{{ $t('common.youtube') }}
					</a>
					<time class="shrink-0 text-sm text-text-muted">
						{{ formatDate(props.episode.published_at) }}
					</time>
				</div>
			</div>
		</div>

		<!-- Read-only progress strip: reflects the saved playback point (or the
		     live playhead for the current episode). Never interactive. -->
		<div
			v-if="progressRatio > 0"
			class="absolute inset-x-0 bottom-0 h-1 bg-surface-input"
			aria-hidden="true"
			data-testid="episode-progress"
		>
			<div class="h-full bg-success" :style="{ width: `${progressRatio}%` }"></div>
		</div>
	</article>
</template>
