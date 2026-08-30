<script setup lang="ts">
	import { computed, onBeforeUnmount, ref } from 'vue';
	import {
		PhCaretDown,
		PhGauge,
		PhList,
		PhMinus,
		PhPause,
		PhPlay,
		PhPlus,
		PhRepeat,
		PhShuffle,
		PhSkipBack,
		PhSkipForward,
		PhArrowCounterClockwise,
		PhArrowClockwise,
		PhX
	} from '@phosphor-icons/vue';
	import {
		parseDurationSeconds,
		sponsorBlockTimelineMarkers,
		SPEED_MAX,
		SPEED_MIN,
		SPEED_STEP,
		usePlayerStore
	} from '@/stores/player';
	import ScrollingText from '@/components/ScrollingText.vue';

	defineProps<{ open: boolean }>();
	const emit = defineEmits<{ close: [] }>();

	const player = usePlayerStore();
	const showSpeed = ref(false);
	const queueOpen = ref(false);
	const speeds = [0.5, 1, 1.25, 1.5, 2];
	let nextTimer: ReturnType<typeof setTimeout> | null = null;
	let nextClickSuppress = false;

	const sponsorBlockMarkers = computed(() => {
		const episode = player.currentEpisode;
		if (!episode) return [];
		const duration = player.duration || parseDurationSeconds(episode.duration) || 0;
		return sponsorBlockTimelineMarkers(
			duration,
			episode.sponsorblock_enabled === true ? episode.sponsorblock_segments : []
		);
	});

	function speedLabel(value: number) {
		return String(Math.round(value * 100) / 100);
	}

	function onSeek(event: MouseEvent) {
		if (player.duration <= 0) return;
		const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const ratio = Math.min(Math.max((event.clientX - rect.left) / rect.width, 0), 1);
		player.seek(ratio * player.duration);
	}

	// Next control: short press skips, long press (> 500ms) skips and marks
	// the finished episode listened — same dual behavior as the wide player
	// (per the up-next-queue capability). Keyboard activation (Enter/Space)
	// resolves to a native click and keeps the short behavior.
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

	onBeforeUnmount(() => {
		if (nextTimer) {
			clearTimeout(nextTimer);
			nextTimer = null;
		}
	});

	function close() {
		emit('close');
	}
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
			v-if="open"
			class="fixed inset-x-0 bottom-0 z-40 flex max-h-[92vh] flex-col overflow-y-auto rounded-t-2xl border-t border-outline bg-surface shadow-[0_-4px_30px_var(--glow)] backdrop-blur-xl"
			data-testid="player-expanded"
		>
			<div class="flex items-center justify-between px-4 pb-2 pt-4">
				<button
					type="button"
					class="flex h-9 w-9 items-center justify-center rounded-full text-text-muted transition-colors hover:text-text"
					:aria-label="$t('player.collapse')"
					@click="close"
				>
					<PhCaretDown class="h-6 w-6" weight="bold" />
				</button>
			</div>

			<div class="flex flex-col items-center gap-4 px-6 pb-6">
				<div
					class="aspect-square w-full max-w-xs overflow-hidden rounded-xl bg-surface-input shadow-card"
				>
					<img
						v-if="player.currentEpisode?.image"
						:src="player.currentEpisode.image"
						:alt="player.currentEpisode.title"
						class="h-full w-full object-cover"
					/>
				</div>

				<div class="w-full min-w-0 text-center">
					<ScrollingText
						class="text-base font-semibold text-text"
						:text="player.currentEpisode?.title ?? $t('player.queueReady')"
						:active="player.playing"
					/>
					<p v-if="player.currentEpisode" class="truncate text-sm text-text-muted">
						{{ player.currentEpisode.channel_title }}
					</p>
				</div>

				<div class="flex w-full items-center justify-center gap-3">
					<div class="relative" data-speed-panel>
						<button
							type="button"
							class="flex items-center gap-1 rounded-md px-2 py-1 text-xs font-medium text-text-muted transition-colors hover:text-text"
							:aria-label="$t('player.speed')"
							:aria-expanded="showSpeed"
							@click="showSpeed = !showSpeed"
						>
							<PhGauge class="h-4 w-4" weight="regular" />
							{{ speedLabel(player.speed) }}x
						</button>
						<div
							v-if="showSpeed"
							class="absolute bottom-full left-1/2 z-10 mb-2 flex -translate-x-1/2 flex-col gap-1 rounded-lg border border-outline bg-surface-card p-1 shadow-card"
							data-testid="speed-panel"
						>
							<div class="flex items-center justify-between gap-1 px-1">
								<button
									type="button"
									class="flex h-7 w-7 items-center justify-center rounded-md text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
									:disabled="player.speed <= SPEED_MIN"
									:aria-label="$t('player.speedDecrease')"
									@click="player.setSpeed(player.speed - SPEED_STEP)"
								>
									<PhMinus class="h-4 w-4" weight="bold" />
								</button>
								<span
									class="min-w-[3rem] text-center text-xs font-semibold text-text"
									data-testid="speed-value"
									>{{ speedLabel(player.speed) }}x</span
								>
								<button
									type="button"
									class="flex h-7 w-7 items-center justify-center rounded-md text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
									:disabled="player.speed >= SPEED_MAX"
									:aria-label="$t('player.speedIncrease')"
									@click="player.setSpeed(player.speed + SPEED_STEP)"
								>
									<PhPlus class="h-4 w-4" weight="bold" />
								</button>
							</div>
							<button
								v-for="s in speeds"
								:key="s"
								type="button"
								class="rounded-md px-3 py-1.5 text-left text-xs font-medium transition-colors"
								:class="
									s === player.speed
										? 'bg-accent-600 text-white'
										: 'text-text-muted hover:text-text'
								"
								@click="
									player.setSpeed(s);
									showSpeed = false;
								"
							>
								{{ speedLabel(s) }}x
							</button>
						</div>
					</div>

					<button
						type="button"
						class="flex h-9 w-9 items-center justify-center rounded-md transition-colors"
						:class="
							player.mobilePlaybackMode !== 'normal'
								? 'bg-accent-600 text-white'
								: 'text-text-muted hover:text-text'
						"
						:aria-label="
							player.mobilePlaybackMode === 'normal'
								? $t('player.mobileModeNormal')
								: player.mobilePlaybackMode === 'repeat'
									? $t('player.repeatAll')
									: $t('player.shuffle')
						"
						@click="player.cycleMobilePlaybackMode()"
					>
						<PhShuffle
							v-if="player.mobilePlaybackMode === 'shuffle'"
							class="h-5 w-5"
							weight="regular"
						/>
						<PhRepeat v-else class="h-5 w-5" weight="regular" />
					</button>

					<div class="relative" data-queue-panel>
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
							class="absolute bottom-full right-0 z-10 mb-2 max-h-80 w-72 overflow-y-auto rounded-lg border border-outline bg-surface-card p-2 shadow-card"
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

				<div class="w-full">
					<div
						class="relative h-5 w-full cursor-pointer py-2"
						role="slider"
						:aria-label="$t('player.seek')"
						:aria-valuenow="Math.round(player.progress)"
						:aria-valuemin="0"
						:aria-valuemax="100"
						data-testid="player-progress-expanded"
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
					<div class="flex items-center justify-between text-xs text-text-muted">
						<span>{{ player.currentLabel }}</span>
						<span>-{{ player.remainingLabel }}</span>
					</div>
				</div>

				<div class="flex w-full items-center justify-center gap-4">
					<button
						type="button"
						class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:text-text-muted"
						:aria-label="$t('player.previous')"
						:disabled="player.currentEpisode == null"
						@click="player.playPrevious()"
					>
						<PhSkipBack class="h-5 w-5" weight="fill" />
					</button>

					<button
						type="button"
						class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
						:aria-label="$t('player.seekBack10')"
						:disabled="player.currentEpisode == null"
						@click="player.seekRelative(-10)"
					>
						<PhArrowCounterClockwise class="h-6 w-6" weight="regular" />
					</button>

					<button
						type="button"
						class="flex h-16 w-16 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary-400 to-primary-600 text-white shadow-lg transition-transform hover:scale-105"
						:aria-label="player.playing ? $t('player.pause') : $t('player.play')"
						:disabled="player.loading || player.currentEpisode == null"
						@click="player.togglePlay()"
					>
						<PhPause v-if="player.playing" class="h-7 w-7 text-white" weight="fill" />
						<PhPlay v-else class="ml-0.5 h-7 w-7 text-white" weight="fill" />
					</button>

					<button
						type="button"
						class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
						:aria-label="$t('player.seekForward10')"
						:disabled="player.currentEpisode == null"
						@click="player.seekRelative(10)"
					>
						<PhArrowClockwise class="h-6 w-6" weight="regular" />
					</button>

					<button
						type="button"
						class="flex h-11 w-11 shrink-0 items-center justify-center rounded-full border border-outline text-text-muted transition-colors hover:text-text disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:text-text-muted"
						:aria-label="$t('player.next')"
						:disabled="player.upNext.length === 0"
						@pointerdown="nextPointerDown"
						@pointerup="nextPointerUp"
						@pointerleave="nextPointerLeave"
						@click="onNextClick"
					>
						<PhSkipForward class="h-5 w-5" weight="fill" />
					</button>
				</div>
			</div>
		</div>
	</Transition>
</template>
