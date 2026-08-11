<script setup lang="ts">
	import { onBeforeUnmount, ref, watch } from 'vue';
	import {
		PhGauge,
		PhPause,
		PhPlay,
		PhSpeakerHigh,
		PhSpeakerSlash,
		PhStop
	} from '@phosphor-icons/vue';
	import { usePlayerStore } from '@/stores/player';

	const player = usePlayerStore();
	const showSpeed = ref(false);
	const visible = ref(false);
	const speeds = [0.5, 1, 1.25, 1.5, 2];

	let hideTimer: ReturnType<typeof setTimeout> | null = null;

	function clearHideTimer() {
		if (hideTimer) {
			clearTimeout(hideTimer);
			hideTimer = null;
		}
	}

	function armHideTimer() {
		clearHideTimer();
		hideTimer = setTimeout(() => {
			visible.value = false;
			hideTimer = null;
		}, 10000);
	}

	watch(
		() => [player.playing, player.stopped, player.currentEpisode?.id] as const,
		([playing, stopped, episodeId]) => {
			if (episodeId == null) {
				visible.value = false;
				clearHideTimer();
				return;
			}
			if (!stopped) {
				visible.value = true;
				clearHideTimer();
				return;
			}
			if (stopped && !playing) {
				armHideTimer();
			}
		},
		{ immediate: true }
	);

	function onSeek(event: MouseEvent) {
		if (player.duration <= 0) return;
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const ratio = Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
		player.seek(ratio * player.duration);
	}

	function onVolumeInput(event: Event) {
		player.setVolume(Number((event.target as HTMLInputElement).value));
	}

	onBeforeUnmount(clearHideTimer);
</script>

<template>
	<Transition
		enter-active-class="transition-transform duration-300 ease-out"
		enter-from-class="translate-y-full"
		enter-to-class="translate-y-0"
		leave-active-class="transition-transform duration-300 ease-in"
		leave-from-class="translate-y-0"
		leave-to-class="translate-y-full"
	>
		<div
			v-if="visible && player.currentEpisode"
			class="fixed bottom-0 left-0 right-0 z-30 border-t border-outline bg-surface/95 shadow-[0_-4px_20px_var(--glow)] backdrop-blur-xl"
		>
			<div class="mx-auto flex h-20 max-w-[1440px] items-center gap-4 px-4 md:px-8">
				<div class="h-14 w-14 shrink-0 overflow-hidden rounded-lg bg-surface-input">
					<img
						v-if="player.currentEpisode.image"
						:src="player.currentEpisode.image"
						:alt="player.currentEpisode.title"
						class="h-full w-full object-cover"
					/>
				</div>

				<div class="hidden min-w-0 flex-col sm:flex">
					<p class="truncate text-sm font-semibold text-text">
						{{ player.currentEpisode.title }}
					</p>
					<p class="truncate text-xs text-text-muted">
						{{ player.currentLabel }} / {{ player.durationLabel }}
					</p>
				</div>

				<button
					type="button"
					class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
					:aria-label="player.playing ? 'Pause' : 'Play'"
					:disabled="player.loading"
					@click="player.togglePlay()"
				>
					<PhPause v-if="player.playing" class="h-5 w-5 text-white" weight="fill" />
					<PhPlay v-else class="ml-0.5 h-5 w-5 text-white" weight="fill" />
				</button>

				<button
					type="button"
					class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text"
					aria-label="Stop"
					@click="player.stop()"
				>
					<PhStop class="h-4 w-4" weight="fill" />
				</button>

				<div
					class="relative h-5 min-w-0 flex-1 cursor-pointer py-2"
					role="slider"
					aria-label="Seek"
					:aria-valuenow="Math.round(player.progress)"
					:aria-valuemin="0"
					:aria-valuemax="100"
					@click="onSeek"
				>
					<div class="h-1.5 w-full overflow-hidden rounded-full bg-surface-input">
						<div
							class="h-full rounded-full bg-accent-400 transition-[width] duration-150"
							:style="{ width: player.progress + '%' }"
						></div>
					</div>
				</div>

				<span class="shrink-0 text-xs text-text-muted sm:hidden">
					{{ player.currentLabel }}
				</span>

				<div class="relative shrink-0">
					<button
						type="button"
						class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium text-text-muted transition-colors hover:text-text"
						@click="showSpeed = !showSpeed"
					>
						<PhGauge class="h-4 w-4" weight="regular" />
						{{ player.speed }}x
					</button>
					<div
						v-if="showSpeed"
						class="absolute bottom-full right-0 z-10 mb-2 flex flex-col rounded-lg border border-outline bg-surface-card p-1 shadow-card"
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

				<div class="hidden shrink-0 items-center gap-2 sm:flex">
					<button
						type="button"
						class="rounded-md p-1 text-text-muted transition-colors hover:text-text"
						:aria-label="player.muted ? 'Unmute' : 'Mute'"
						@click="player.toggleMute()"
					>
						<PhSpeakerSlash v-if="player.muted" class="h-4 w-4" weight="regular" />
						<PhSpeakerHigh v-else class="h-4 w-4" weight="regular" />
					</button>
					<input
						class="h-1 w-20 cursor-pointer accent-accent-500"
						type="range"
						min="0"
						max="1"
						step="0.05"
						:value="player.volume"
						@input="onVolumeInput"
					/>
				</div>
			</div>
		</div>
	</Transition>
</template>
