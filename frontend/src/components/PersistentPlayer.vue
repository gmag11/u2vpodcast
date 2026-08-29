<script setup lang="ts">
	import { computed, onBeforeUnmount, ref, watch } from 'vue';
	import {
		PhGauge,
		PhList,
		PhPause,
		PhPlay,
		PhRepeat,
		PhRepeatOnce,
		PhShuffle,
		PhSkipBack,
		PhSkipForward,
		PhSpeakerHigh,
		PhSpeakerSlash,
		PhStop,
		PhX
	} from '@phosphor-icons/vue';
	import {
		parseDurationSeconds,
		sponsorBlockTimelineMarkers,
		usePlayerStore
	} from '@/stores/player';

	const player = usePlayerStore();
	const showSpeed = ref(false);
	const queueOpen = ref(false);
	const visible = ref(false);
	const speeds = [0.5, 1, 1.25, 1.5, 2];
	const sponsorBlockMarkers = computed(() => {
		const episode = player.currentEpisode;
		if (!episode) return [];
		const duration = player.duration || parseDurationSeconds(episode.duration) || 0;
		return sponsorBlockTimelineMarkers(
			duration,
			episode.sponsorblock_enabled === true ? episode.sponsorblock_segments : []
		);
	});

	let hideTimer: ReturnType<typeof setTimeout> | null = null;
	let nextTimer: ReturnType<typeof setTimeout> | null = null;
	let nextClickSuppress = false;

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
		() =>
			[player.playing, player.stopped, player.currentEpisode?.id, player.upNext.length] as const,
		([playing, stopped, episodeId, queueLength]) => {
			if (episodeId == null && queueLength === 0) {
				visible.value = false;
				clearHideTimer();
				return;
			}
			if (!stopped) {
				visible.value = true;
				clearHideTimer();
				return;
			}
			if (episodeId == null) {
				// Queue-only mode (e.g. right after a reload that restored the
				// queue but no current episode): the bar exists so the queue
				// stays reachable until the user plays something.
				visible.value = true;
				clearHideTimer();
				return;
			}
			if (stopped && queueLength > 0) {
				// A non-empty queue must stay reachable: keep the bar visible so
				// the user can inspect and manage up-next without playing.
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

	// Next control: short press skips, long press (> 500ms) skips and marks
	// the finished episode listened. Keyboard activation (Enter/Space) resolves
	// to a native click and keeps the short behavior.
	function nextPointerDown() {
		if (player.upNext.length === 0) return;
		nextClickSuppress = false;
		nextTimer = setTimeout(() => {
			nextTimer = null;
			nextClickSuppress = true;
			player.skipNext(true);
		}, 500);
	}

	function nextPointerUp() {
		if (nextTimer) {
			clearTimeout(nextTimer);
			nextTimer = null;
			nextClickSuppress = true;
			player.skipNext();
		}
	}

	function nextPointerLeave() {
		if (nextTimer) {
			clearTimeout(nextTimer);
			nextTimer = null;
		}
	}

	function onNextClick() {
		if (nextClickSuppress) {
			nextClickSuppress = false;
			return;
		}
		player.skipNext();
	}

	function onDocumentPointerDown(event: Event) {
		const target = event.target as HTMLElement;
		if (queueOpen.value && !target.closest('[data-queue-panel]')) queueOpen.value = false;
		if (showSpeed.value && !target.closest('[data-speed-panel]')) showSpeed.value = false;
	}

	function onDocumentKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			queueOpen.value = false;
			showSpeed.value = false;
		}
	}

	function onSeek(event: MouseEvent) {
		if (player.duration <= 0) return;
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const ratio = Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
		player.seek(ratio * player.duration);
	}

	function onVolumeInput(event: Event) {
		player.setVolume(Number((event.target as HTMLInputElement).value));
	}

	onBeforeUnmount(() => {
		clearHideTimer();
		if (nextTimer) {
			clearTimeout(nextTimer);
			nextTimer = null;
		}
		document.removeEventListener('pointerdown', onDocumentPointerDown);
		document.removeEventListener('keydown', onDocumentKeydown);
	});

	document.addEventListener('pointerdown', onDocumentPointerDown);
	document.addEventListener('keydown', onDocumentKeydown);
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
			v-if="visible && (player.currentEpisode != null || player.upNext.length > 0)"
			class="fixed bottom-0 left-0 right-0 z-30 border-t border-outline bg-surface/95 shadow-[0_-4px_20px_var(--glow)] backdrop-blur-xl"
		>
			<div class="mx-auto flex h-20 max-w-[1440px] items-center gap-2 px-4 md:gap-4 md:px-8">
				<div class="h-14 w-14 shrink-0 overflow-hidden rounded-lg bg-surface-input">
					<img
						v-if="player.currentEpisode?.image"
						:src="player.currentEpisode.image"
						:alt="player.currentEpisode.title"
						class="h-full w-full object-cover"
					/>
				</div>

				<div class="hidden min-w-0 flex-col sm:flex">
					<p v-if="player.currentEpisode" class="max-w-60 truncate text-sm font-semibold text-text">
						{{ player.currentEpisode.title }}
					</p>
					<p v-else class="max-w-60 truncate text-sm font-semibold text-text">
						{{ $t('player.queueReady') }}
					</p>
					<p v-if="player.currentEpisode" class="max-w-60 truncate text-xs text-text-muted">
						{{ player.currentLabel }} / {{ player.durationLabel }}
					</p>
				</div>

				<button
					type="button"
					class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:text-text-muted"
					:aria-label="$t('player.previous')"
					:disabled="player.currentEpisode == null"
					@click="player.playPrevious()"
				>
					<PhSkipBack class="h-4 w-4" weight="fill" />
				</button>

				<button
					type="button"
					class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
					:aria-label="player.playing ? $t('player.pause') : $t('player.play')"
					:disabled="player.loading || player.currentEpisode == null"
					@click="player.togglePlay()"
				>
					<PhPause v-if="player.playing" class="h-5 w-5 text-white" weight="fill" />
					<PhPlay v-else class="ml-0.5 h-5 w-5 text-white" weight="fill" />
				</button>

				<button
					type="button"
					class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:text-text-muted"
					:aria-label="$t('player.stop')"
					@click="player.stop()"
				>
					<PhStop class="h-4 w-4" weight="fill" />
				</button>

				<button
					type="button"
					class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:text-text-muted"
					:aria-label="$t('player.next')"
					:disabled="player.upNext.length === 0"
					@pointerdown="nextPointerDown"
					@pointerup="nextPointerUp"
					@pointerleave="nextPointerLeave"
					@click="onNextClick"
				>
					<PhSkipForward class="h-4 w-4" weight="fill" />
				</button>

				<div
					class="relative h-5 min-w-0 flex-1 cursor-pointer py-2"
					role="slider"
					:aria-label="$t('player.seek')"
					:aria-valuenow="Math.round(player.progress)"
					:aria-valuemin="0"
					:aria-valuemax="100"
					@click="onSeek"
				>
					<div class="relative h-1.5 w-full overflow-hidden rounded-full bg-surface-input">
						<div
							class="absolute inset-y-0 left-0 rounded-full bg-accent-400 transition-[width] duration-150"
							:style="{ width: player.progress + '%' }"
						></div>
						<div
							v-for="(marker, index) in sponsorBlockMarkers"
							:key="index"
							class="absolute inset-y-0 z-10"
							:class="marker.category === 'sponsor' ? 'bg-sponsorblock' : 'bg-sponsorblock-other'"
							:data-category="marker.category"
							data-testid="player-sponsorblock-segment"
							:style="{ left: `${marker.left}%`, width: `${marker.width}%` }"
						></div>
					</div>
				</div>

				<span class="shrink-0 text-xs text-text-muted sm:hidden">
					{{ player.currentLabel }}
				</span>

				<div class="relative shrink-0" data-speed-panel>
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

				<div class="flex shrink-0 items-center gap-1">
					<button
						type="button"
						class="flex h-9 w-9 items-center justify-center rounded-md transition-colors"
						:class="player.shuffle ? 'bg-accent-600 text-white' : 'text-text-muted hover:text-text'"
						:aria-label="$t('player.shuffle')"
						:aria-pressed="player.shuffle"
						:title="$t('player.shuffle')"
						@click="player.toggleShuffle()"
					>
						<PhShuffle class="h-5 w-5" weight="regular" />
					</button>
					<button
						type="button"
						class="flex h-9 w-9 items-center justify-center rounded-md transition-colors"
						:class="
							player.repeat !== 'none'
								? 'bg-accent-600 text-white'
								: 'text-text-muted hover:text-text'
						"
						:aria-label="
							player.repeat === 'none'
								? $t('player.repeatOff')
								: player.repeat === 'all'
									? $t('player.repeatAll')
									: $t('player.repeatOne')
						"
						:title="
							player.repeat === 'none'
								? $t('player.repeatOff')
								: player.repeat === 'all'
									? $t('player.repeatAll')
									: $t('player.repeatOne')
						"
						@click="player.cycleRepeat()"
					>
						<PhRepeat v-if="player.repeat !== 'one'" class="h-5 w-5" weight="regular" />
						<PhRepeatOnce v-else class="h-5 w-5" weight="regular" />
					</button>
				</div>

				<div class="hidden shrink-0 items-center gap-2 sm:flex">
					<button
						type="button"
						class="rounded-md p-1 text-text-muted transition-colors hover:text-text"
						:aria-label="player.muted ? $t('player.unmute') : $t('player.mute')"
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

				<div class="relative shrink-0" data-queue-panel>
					<button
						type="button"
						class="relative flex h-9 w-9 items-center justify-center rounded-md text-text-muted transition-colors hover:text-text"
						:aria-label="$t('player.queue')"
						:aria-expanded="queueOpen"
						@click="queueOpen = !queueOpen"
					>
						<PhList class="h-5 w-5" weight="regular" />
						<span
							v-if="player.upNext.length > 0"
							class="absolute -right-0.5 -top-0.5 flex h-4 min-w-4 items-center justify-center rounded-full bg-accent-600 px-1 text-[10px] font-semibold text-white"
						>
							{{ player.upNext.length }}
						</span>
					</button>

					<div
						v-if="queueOpen"
						class="absolute bottom-full right-0 z-10 mb-2 w-80 max-h-80 overflow-y-auto rounded-lg border border-outline bg-surface-card p-2 shadow-card"
					>
						<p class="px-2 py-1 text-xs font-semibold uppercase tracking-wide text-text-muted">
							{{ $t('player.upNext') }} ({{ player.upNext.length }})
						</p>
						<p v-if="player.upNext.length === 0" class="px-2 py-3 text-sm text-text-muted">
							{{ $t('player.emptyQueue') }}
						</p>
						<ul v-else class="flex flex-col gap-1">
							<li
								v-for="(ep, index) in player.upNext"
								:key="ep.id"
								class="flex items-center gap-2 rounded-md px-2 py-1.5 hover:bg-surface-input"
							>
								<span class="w-4 shrink-0 text-center text-xs text-text-muted">{{
									index + 1
								}}</span>
								<img
									v-if="ep.image"
									:src="ep.image"
									alt=""
									class="h-8 w-12 shrink-0 rounded object-cover"
								/>
								<div class="min-w-0 flex-1">
									<p class="truncate text-xs font-medium text-text">{{ ep.title }}</p>
									<p class="truncate text-[11px] text-text-muted">{{ ep.channel_title }}</p>
								</div>
								<button
									type="button"
									class="shrink-0 rounded p-1 text-text-muted transition-colors hover:text-text"
									:aria-label="$t('player.removeFromQueue')"
									@click="player.removeFromQueue(ep.id)"
								>
									<PhX class="h-4 w-4" weight="bold" />
								</button>
							</li>
						</ul>
						<button
							v-if="player.upNext.length > 0"
							type="button"
							class="mt-1 w-full rounded-md px-2 py-1.5 text-left text-xs font-medium text-text-muted transition-colors hover:bg-surface-input hover:text-text"
							@click="
								player.clearQueue();
								queueOpen = false;
							"
						>
							{{ $t('player.clearQueue') }}
						</button>
					</div>
				</div>
			</div>
		</div>
	</Transition>
</template>
