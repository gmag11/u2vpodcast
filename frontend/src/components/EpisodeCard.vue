<script setup lang="ts">
	import { computed, onBeforeUnmount, ref } from 'vue';
	import {
		PhGauge,
		PhLinkSimple,
		PhPause,
		PhPlay,
		PhSpeakerHigh,
		PhSpeakerSlash
	} from '@phosphor-icons/vue';
	import { toHHMMSS } from '@/lib/utils/formatter';
	import type { Episode } from '@/types';

	const props = defineProps<{
		episode: Episode;
	}>();

	const audioRef = ref<HTMLAudioElement | null>(null);
	const playing = ref(false);
	const currentTime = ref(0);
	const duration = ref(0);
	const volume = ref(1);
	const muted = ref(false);
	const speed = ref(1);
	const showSpeed = ref(false);
	const loading = ref(false);

	const speeds = [0.5, 1, 1.25, 1.5, 2];

	const audioUrl = computed(() => {
		const slug = props.episode.channel_slug;
		return `/media/${slug}/${props.episode.yt_id}.mp3`;
	});

	const currentLabel = computed(() => toHHMMSS(currentTime.value) || '0:00');
	const durationLabel = computed(() => {
		if (duration.value > 0) return toHHMMSS(duration.value);
		const raw = Number(props.episode.duration);
		if (!isNaN(raw) && raw > 0) return toHHMMSS(raw);
		return '';
	});

	const progress = computed(() =>
		duration.value > 0 ? (currentTime.value / duration.value) * 100 : 0
	);

	function onTimeUpdate() {
		if (audioRef.value) currentTime.value = audioRef.value.currentTime;
	}

	function onLoadedMetadata() {
		if (audioRef.value) duration.value = audioRef.value.duration;
	}

	function onPlay() {
		playing.value = true;
	}

	function onPause() {
		playing.value = false;
	}

	function onWaiting() {
		loading.value = true;
	}

	function onCanPlay() {
		loading.value = false;
	}

	async function togglePlay() {
		if (!audioRef.value) return;
		if (audioRef.value.paused) {
			await audioRef.value.play();
		} else {
			audioRef.value.pause();
		}
	}

	function seek(event: MouseEvent) {
		if (!audioRef.value || duration.value <= 0) return;
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const ratio = Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
		audioRef.value.currentTime = ratio * duration.value;
	}

	function toggleMute() {
		if (!audioRef.value) return;
		audioRef.value.muted = !audioRef.value.muted;
		muted.value = audioRef.value.muted;
	}

	function onVolumeInput(event: Event) {
		const value = Number((event.target as HTMLInputElement).value);
		volume.value = value;
		if (audioRef.value) audioRef.value.volume = value;
	}

	function setSpeed(value: number) {
		speed.value = value;
		if (audioRef.value) audioRef.value.playbackRate = value;
		showSpeed.value = false;
	}

	function formatDate(value: Date | string) {
		return new Date(value).toLocaleDateString('en-US');
	}

	onBeforeUnmount(() => {
		if (audioRef.value) audioRef.value.pause();
	});
</script>

<template>
	<article
		class="flex flex-col gap-4 rounded-xl border border-outline bg-surface-card p-5 shadow-card"
	>
		<audio
			ref="audioRef"
			:src="audioUrl"
			preload="metadata"
			@timeupdate="onTimeUpdate"
			@loadedmetadata="onLoadedMetadata"
			@play="onPlay"
			@pause="onPause"
			@waiting="onWaiting"
			@canplay="onCanPlay"
		></audio>

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
				:disabled="loading"
				@click="togglePlay"
			>
				<PhPause v-if="playing" class="h-4 w-4 text-white" weight="fill" />
				<PhPlay v-else class="ml-0.5 h-4 w-4 text-white" weight="fill" />
			</button>

			<div
				class="relative h-5 flex-1 cursor-pointer py-2"
				role="slider"
				aria-label="Seek"
				:aria-valuenow="Math.round(progress)"
				:aria-valuemin="0"
				:aria-valuemax="100"
				@click="seek"
			>
				<div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-input">
					<div
						class="h-full rounded-full bg-accent-400 transition-[width] duration-150"
						:style="{ width: progress + '%' }"
					></div>
				</div>
			</div>

			<div class="flex shrink-0 items-center gap-2 text-sm text-text-muted">
				<span>{{ currentLabel }}</span>
				<span v-if="durationLabel">/ {{ durationLabel }}</span>
			</div>
		</div>

		<div class="flex items-center justify-between px-1">
			<div class="relative">
				<button
					type="button"
					class="flex items-center gap-1.5 rounded-md px-2 py-1 text-xs font-medium text-text-muted transition-colors hover:text-text"
					@click="showSpeed = !showSpeed"
				>
					<PhGauge class="h-4 w-4" weight="regular" />
					{{ speed }}x
				</button>
				<div
					v-if="showSpeed"
					class="absolute bottom-full left-0 z-10 mb-2 flex flex-col rounded-lg border border-outline bg-surface-card p-1 shadow-card"
				>
					<button
						v-for="s in speeds"
						:key="s"
						type="button"
						class="rounded-md px-3 py-1.5 text-left text-xs font-medium transition-colors"
						:class="s === speed ? 'bg-accent-600 text-white' : 'text-text-muted hover:text-text'"
						@click="setSpeed(s)"
					>
						{{ s }}x
					</button>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<button
					type="button"
					class="rounded-md p-1 text-text-muted transition-colors hover:text-text"
					:aria-label="muted ? 'Unmute' : 'Mute'"
					@click="toggleMute"
				>
					<PhSpeakerSlash v-if="muted" class="h-4 w-4" weight="regular" />
					<PhSpeakerHigh v-else class="h-4 w-4" weight="regular" />
				</button>
				<input
					class="h-1 w-20 cursor-pointer accent-accent-500"
					type="range"
					min="0"
					max="1"
					step="0.05"
					:value="volume"
					@input="onVolumeInput"
				/>
			</div>
		</div>
	</article>
</template>
