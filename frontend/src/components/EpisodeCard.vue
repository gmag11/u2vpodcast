<script setup lang="ts">
	import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
	import { useI18n } from 'vue-i18n';
	import {
		PhArrowCounterClockwise,
		PhLinkSimple,
		PhListPlus,
		PhDotsThreeVertical,
		PhMicrophoneStage,
		PhPause,
		PhPlay,
		PhPlaylist,
		PhStar,
		PhStop
	} from '@phosphor-icons/vue';
	import { usePlayerStore, RESUME_POSITION_S, parseDurationSeconds } from '@/stores/player';
	import { usePlaylistStore } from '@/stores/playlists';
	import { useFavoritesStore } from '@/stores/favorites';
	import { useNotificationStore } from '@/stores/notification';
	import { api } from '@/lib/api/client';
	import type { Episode } from '@/types';
	import { toHHMMSS } from '@/lib/utils/formatter';

	const props = withDefaults(
		defineProps<{
			episode: Episode;
			compact?: boolean;
			presentation?: 'standard' | 'playlist';
			list?: Episode[];
			queueSource?: 'playlist' | 'list';
		}>(),
		{
			compact: false,
			presentation: 'standard',
			list: undefined,
			queueSource: 'list'
		}
	);

	const player = usePlayerStore();
	const playlists = usePlaylistStore();
	const favorites = useFavoritesStore();
	const notification = useNotificationStore();
	const { d, t } = useI18n();

	// Every time the card receives an episode, merge its stored flag into the
	// shared favorites store so the star agrees with the server and with any
	// other copy of the same episode rendered elsewhere (episode-favorites).
	watch(
		() => props.episode,
		(episode) => favorites.sync(episode),
		{ immediate: true }
	);

	const isCurrent = computed(() => player.isCurrent(props.episode));
	const isPlaying = computed(() => isCurrent.value && player.playing);
	const isPlaylistPresentation = computed(() => props.presentation === 'playlist');
	const menuOpen = ref(false);
	const menuTrigger = ref<HTMLButtonElement | null>(null);
	const menuElement = ref<HTMLElement | null>(null);
	const titleViewport = ref<HTMLElement | null>(null);
	const titleText = ref<HTMLElement | null>(null);
	const titleScrollDistance = ref(0);
	const TITLE_SCROLL_GAP_PX = 32;
	const TITLE_SCROLL_SPEED_PX_PER_SECOND = 32;
	const menuId = computed(() => `episode-actions-${props.episode.id}`);

	function formatDurationLabel(seconds: number) {
		const totalSeconds = Math.max(0, Math.floor(seconds));
		const hours = Math.floor(totalSeconds / 3600);
		const minutes = Math.floor((totalSeconds % 3600) / 60);
		const remainingSeconds = totalSeconds % 60;
		const minuteSeconds = `${minutes}:${String(remainingSeconds).padStart(2, '0')}`;
		return hours > 0
			? `${hours}:${String(minutes).padStart(2, '0')}:${String(remainingSeconds).padStart(2, '0')}`
			: minuteSeconds;
	}

	const durationLabel = computed(() => {
		const storedDuration = parseDurationSeconds(props.episode.duration) ?? 0;
		const durationSeconds =
			isCurrent.value && player.duration > 0 ? player.duration : storedDuration;
		return formatDurationLabel(durationSeconds);
	});
	const inPlaylist = computed(() => playlists.episodeIdSet.has(props.episode.id));
	const isFavorite = computed(() => favorites.favoriteIdSet.has(props.episode.id));

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
	const titleScrollActive = computed(() => isPlaying.value && titleScrollDistance.value > 0);
	const titleScrollStyle = computed(() => ({
		'--playlist-title-distance': `${titleScrollDistance.value}px`,
		'--playlist-title-duration': `${titleScrollDistance.value / TITLE_SCROLL_SPEED_PX_PER_SECOND}s`
	}));
	let titleResizeObserver: ResizeObserver | undefined;

	function formatDate(value: Date | string) {
		return d(new Date(value), 'short');
	}

	function togglePlayback() {
		if (isCurrent.value) player.togglePlay();
		else player.play(props.episode, props.list, { queueSource: props.queueSource });
	}

	async function openMenu() {
		menuOpen.value = true;
		await nextTick();
		menuElement.value?.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
	}

	function closeMenu(restoreFocus = true) {
		menuOpen.value = false;
		if (restoreFocus) nextTick(() => menuTrigger.value?.focus());
	}

	function toggleMenu() {
		if (menuOpen.value) closeMenu();
		else void openMenu();
	}

	function onDocumentPointerDown(event: PointerEvent) {
		if (!menuOpen.value) return;
		const target = event.target as Node;
		if (menuElement.value?.contains(target) || menuTrigger.value?.contains(target)) return;
		closeMenu(false);
	}

	function onDocumentKeydown(event: KeyboardEvent) {
		if (event.key !== 'Escape' || !menuOpen.value) return;
		event.preventDefault();
		closeMenu();
	}

	async function measureTitleScroll() {
		await nextTick();
		const viewportWidth = titleViewport.value?.clientWidth ?? 0;
		const textWidth = titleText.value?.scrollWidth ?? 0;
		titleScrollDistance.value =
			viewportWidth > 0 && textWidth > viewportWidth ? textWidth + TITLE_SCROLL_GAP_PX : 0;
	}

	watch(
		() => props.episode.title,
		() => void measureTitleScroll()
	);

	onMounted(() => {
		document.addEventListener('pointerdown', onDocumentPointerDown);
		document.addEventListener('keydown', onDocumentKeydown);
		void measureTitleScroll();
		if (typeof ResizeObserver !== 'undefined') {
			titleResizeObserver = new ResizeObserver(() => void measureTitleScroll());
			if (titleViewport.value) titleResizeObserver.observe(titleViewport.value);
			if (titleText.value) titleResizeObserver.observe(titleText.value);
		}
	});

	onBeforeUnmount(() => {
		document.removeEventListener('pointerdown', onDocumentPointerDown);
		document.removeEventListener('keydown', onDocumentKeydown);
		titleResizeObserver?.disconnect();
	});

	// Playlist toggle: add when absent, remove when present, notifying on both
	// outcomes (playlist-capability). The id set drives the button state, so the
	// card never needs a playlist refetch.
	async function togglePlaylist() {
		const id = props.episode.id;
		if (inPlaylist.value) {
			const result = await playlists.remove(id);
			notification.show(
				result.ok ? t('playlist.removed') : t('playlist.removeFailed'),
				result.ok ? 'success' : 'error'
			);
		} else {
			const result = await playlists.add(id);
			notification.show(
				result.ok ? t('playlist.added') : t('playlist.addFailed'),
				result.ok ? 'success' : 'error'
			);
		}
	}

	// Favorite toggle: hollow star when not favorite, filled when favorite,
	// notifying on both outcomes. The id set from the shared store drives the
	// button state, so every copy of the episode flips together
	// (episode-favorites).
	async function toggleFavorite() {
		const favorite = !isFavorite.value;
		const result = await favorites.set(props.episode, favorite);
		notification.show(
			result.ok
				? favorite
					? t('favorites.added')
					: t('favorites.removed')
				: t('favorites.failed'),
			result.ok ? 'success' : 'error'
		);
	}

	async function resetProgress() {
		const recorded = liveEpisode.value;
		try {
			const result = await api.updateEpisodeProgress(props.episode.yt_id, {
				position_seconds: 0,
				listened: recorded.listen
			});
			if (!result.ok) {
				notification.show(t('playlist.resetProgressFailed'), 'error');
				return;
			}
			if (isCurrent.value) {
				player.seek(0);
				player.currentTime = 0;
			}
			player.applyProgress(props.episode, {
				position_seconds: 0,
				listen: recorded.listen,
				listened_at: recorded.listened_at ?? null
			});
			notification.show(t('playlist.progressReset'), 'success');
		} catch (err) {
			console.error(err);
			notification.show(t('playlist.resetProgressFailed'), 'error');
		}
	}

	async function runMenuAction(action: () => void | Promise<void>) {
		await action();
		closeMenu();
	}

	// "Mark as not listened": clears the listened state (position reset to 0),
	// swaps the card back from the played mark immediately, and re-appends the
	// episode at the end of the playlist. If the playlist add fails, the cleared
	// mark is kept and the error surfaced (playlist-capability).
	const unmarking = ref(false);
	async function unmark() {
		if (unmarking.value) return;
		unmarking.value = true;
		try {
			const progress = await api.updateEpisodeProgress(props.episode.yt_id, {
				position_seconds: 0,
				listened: false
			});
			if (!progress.ok) {
				notification.show(t('playlist.unmarkFailed'), 'error');
				return;
			}
			player.applyProgress(props.episode, {
				position_seconds: 0,
				listen: false,
				listened_at: null
			});
			const result = await playlists.add(props.episode.id);
			notification.show(
				result.ok ? t('playlist.added') : t('playlist.addFailed'),
				result.ok ? 'success' : 'error'
			);
		} catch (err) {
			console.error(err);
			notification.show(t('playlist.unmarkFailed'), 'error');
		} finally {
			unmarking.value = false;
		}
	}
</script>

<template>
	<article
		class="relative flex flex-col gap-4 rounded-xl border border-outline bg-surface-card shadow-card"
		:class="[
			isCurrent ? 'border-accent-500/60' : '',
			isPlaylistPresentation
				? 'overflow-visible p-2 sm:overflow-hidden sm:p-4'
				: compact
					? 'overflow-hidden p-4'
					: 'overflow-hidden p-5'
		]"
	>
		<!-- Played mark: the card's top-right corner is tinted green -->
		<span
			v-if="hasPlayedMark"
			class="absolute right-0 top-0"
			:class="isPlaylistPresentation ? 'hidden sm:block' : ''"
			data-testid="listened-mark"
			role="img"
			:aria-label="$t('card.listened')"
		>
			<svg class="h-7 w-7 text-success" viewBox="0 0 24 24" aria-hidden="true">
				<path d="M0 0 L24 0 L24 24 Z" fill="currentColor" />
			</svg>
		</span>

		<div
			v-if="isPlaylistPresentation"
			class="flex min-w-0 items-center gap-2 sm:hidden"
			data-testid="playlist-mobile-card"
		>
			<button
				type="button"
				class="group relative h-16 w-16 shrink-0 overflow-hidden rounded-md bg-surface-input focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-500"
				data-testid="playlist-image-playback"
				:aria-label="isPlaying ? $t('player.pause') : $t('player.play')"
				:disabled="isCurrent && player.loading"
				@click="togglePlayback"
			>
				<img
					v-if="props.episode.image"
					:src="props.episode.image"
					alt=""
					class="h-full w-full object-cover"
				/>
				<span
					class="absolute inset-0 flex items-center justify-center bg-black/25 text-white transition-colors group-hover:bg-black/40"
				>
					<PhPause v-if="isPlaying" class="h-6 w-6" weight="fill" />
					<PhPlay v-else class="ml-0.5 h-6 w-6" weight="fill" />
				</span>
			</button>

			<div class="min-w-0 flex-1 overflow-hidden">
				<div
					ref="titleViewport"
					class="playlist-title-viewport overflow-hidden"
					data-testid="playlist-title-viewport"
					:aria-label="props.episode.title"
				>
					<h2
						class="playlist-title-scroll inline-flex w-max min-w-full whitespace-nowrap text-sm font-bold text-text"
						:class="{ 'playlist-title-scroll--active': titleScrollActive }"
						:style="titleScrollStyle"
						data-testid="playlist-title-scroll"
					>
						<span ref="titleText" class="shrink-0" data-testid="playlist-title-text">
							{{ props.episode.title }}
						</span>
						<span
							v-if="titleScrollDistance > 0"
							class="shrink-0"
							data-testid="playlist-title-copy"
							aria-hidden="true"
						>
							{{ props.episode.title }}
						</span>
					</h2>
				</div>
				<p class="truncate text-xs font-normal text-text-muted" data-testid="playlist-channel">
					{{ props.episode.channel_title }}
				</p>
				<div
					class="mt-1 flex items-center justify-between gap-2 text-xs text-text-muted"
					data-testid="playlist-metadata"
				>
					<span>{{ durationLabel }}</span>
					<time>{{ formatDate(props.episode.published_at) }}</time>
				</div>
			</div>

			<div
				class="relative flex h-16 w-8 shrink-0 flex-col items-center justify-between"
				data-testid="playlist-status-column"
			>
				<button
					ref="menuTrigger"
					type="button"
					class="flex h-8 w-8 items-center justify-center rounded-md text-text-muted hover:bg-surface-high hover:text-text focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-500"
					data-testid="playlist-actions-trigger"
					:aria-label="$t('playlist.actions')"
					aria-haspopup="menu"
					:aria-expanded="menuOpen"
					:aria-controls="menuOpen ? menuId : undefined"
					@click="toggleMenu"
				>
					<PhDotsThreeVertical class="h-4 w-4" weight="bold" />
				</button>
				<div
					v-if="menuOpen"
					:id="menuId"
					ref="menuElement"
					role="menu"
					class="absolute right-0 top-full z-50 mt-1 w-52 overflow-hidden rounded-md border border-outline bg-surface-card py-1 text-sm text-text shadow-card"
				>
					<button
						type="button"
						role="menuitem"
						class="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-surface-high focus:bg-surface-high focus:outline-none"
						:aria-label="$t('playlist.favorite')"
						@click="runMenuAction(toggleFavorite)"
					>
						<PhStar class="h-4 w-4" :weight="isFavorite ? 'fill' : 'regular'" />
						<span>{{ $t('playlist.favorite') }}</span>
					</button>
					<button
						type="button"
						role="menuitem"
						class="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-surface-high focus:bg-surface-high focus:outline-none"
						:aria-label="$t('playlist.remove')"
						@click="runMenuAction(togglePlaylist)"
					>
						<PhPlaylist class="h-4 w-4" weight="fill" />
						<span>{{ $t('playlist.remove') }}</span>
					</button>
					<a
						role="menuitem"
						class="flex w-full items-center gap-2 px-3 py-2 hover:bg-surface-high focus:bg-surface-high focus:outline-none"
						data-testid="playlist-original-link"
						:aria-label="$t('playlist.originalLink')"
						:href="props.episode.webpage_url"
						target="_blank"
						rel="noopener noreferrer"
						@click="closeMenu(false)"
					>
						<PhLinkSimple class="h-4 w-4" />
						<span>{{ $t('playlist.originalLink') }}</span>
					</a>
					<button
						type="button"
						role="menuitem"
						class="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-surface-high focus:bg-surface-high focus:outline-none"
						:aria-label="$t('playlist.resetProgress')"
						@click="runMenuAction(resetProgress)"
					>
						<PhArrowCounterClockwise class="h-4 w-4" />
						<span>{{ $t('playlist.resetProgress') }}</span>
					</button>
					<RouterLink
						role="menuitem"
						class="flex w-full items-center gap-2 px-3 py-2 hover:bg-surface-high focus:bg-surface-high focus:outline-none"
						data-testid="playlist-channel-view"
						:aria-label="$t('playlist.channelView')"
						:to="{ name: 'episodes', params: { channelId: String(props.episode.channel_id) } }"
						@click="closeMenu(false)"
					>
						<PhMicrophoneStage class="h-4 w-4" />
						<span>{{ $t('playlist.channelView') }}</span>
					</RouterLink>
				</div>
				<div class="flex w-full items-center justify-between">
					<span
						role="img"
						data-icon="star"
						data-testid="playlist-favorite-status"
						:data-active="isFavorite"
						:class="isFavorite ? 'text-accent-500' : 'text-text-muted'"
						:aria-label="isFavorite ? $t('favorites.remove') : $t('favorites.add')"
					>
						<PhStar class="h-3.5 w-3.5" :weight="isFavorite ? 'fill' : 'regular'" />
					</span>
					<span
						role="img"
						data-icon="playlist"
						data-testid="playlist-membership-status"
						:data-active="inPlaylist"
						:class="inPlaylist ? 'text-accent-500' : 'text-text-muted'"
						:aria-label="inPlaylist ? $t('playlist.remove') : $t('playlist.add')"
					>
						<PhPlaylist class="h-3.5 w-3.5" :weight="inPlaylist ? 'fill' : 'regular'" />
					</span>
				</div>
			</div>
		</div>

		<div
			class="flex-1 flex-col gap-5 sm:flex-row sm:items-start"
			:class="isPlaylistPresentation ? 'hidden sm:flex' : 'flex'"
		>
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
							@click="togglePlayback"
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
						@click="togglePlayback"
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
				<div v-if="resumeSeconds > 0" class="mt-1.5 flex flex-wrap items-center gap-x-3 gap-y-1">
					<span class="text-xs text-text-muted">
						{{ $t('card.continueAt', { time: resumeLabel }) }}
					</span>
					<button
						v-if="canStartOver"
						type="button"
						class="inline-flex items-center text-xs text-accent-500 transition-colors hover:underline"
						@click="
							player.play(props.episode, props.list, {
								fromStart: true,
								queueSource: props.queueSource
							})
						"
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
					<div class="flex shrink-0 items-center gap-1.5">
						<button
							v-if="hasPlayedMark"
							type="button"
							class="flex h-8 w-8 items-center justify-center rounded-md border border-outline text-text-muted transition-colors hover:text-text disabled:opacity-50"
							:disabled="unmarking"
							:aria-label="$t('playlist.unmark')"
							:title="$t('playlist.unmark')"
							@click="unmark"
						>
							<PhArrowCounterClockwise class="h-4 w-4" weight="regular" />
						</button>
						<button
							type="button"
							class="flex h-8 w-8 items-center justify-center rounded-md border border-outline text-accent-500 transition-colors hover:text-accent-400"
							:aria-label="isFavorite ? $t('favorites.remove') : $t('favorites.add')"
							:title="isFavorite ? $t('favorites.remove') : $t('favorites.add')"
							@click="toggleFavorite"
						>
							<PhStar class="h-4 w-4" :weight="isFavorite ? 'fill' : 'regular'" />
						</button>
						<button
							type="button"
							class="flex h-8 w-8 items-center justify-center rounded-md border border-outline text-accent-500 transition-colors hover:text-accent-400"
							:aria-label="inPlaylist ? $t('playlist.remove') : $t('playlist.add')"
							:title="inPlaylist ? $t('playlist.remove') : $t('playlist.add')"
							@click="togglePlaylist"
						>
							<PhPlaylist v-if="inPlaylist" class="h-4 w-4" weight="fill" />
							<PhListPlus v-else class="h-4 w-4" weight="regular" />
						</button>
						<time class="shrink-0 text-sm text-text-muted">
							{{ formatDate(props.episode.published_at) }}
						</time>
					</div>
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

<style scoped>
	.playlist-title-viewport {
		container-type: inline-size;
	}

	.playlist-title-scroll {
		gap: 32px;
	}

	.playlist-title-scroll--active {
		animation: playlist-title-scroll var(--playlist-title-duration) linear infinite;
		will-change: transform;
	}

	@keyframes playlist-title-scroll {
		from {
			transform: translateX(0);
		}
		to {
			transform: translateX(calc(-1 * var(--playlist-title-distance)));
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.playlist-title-scroll--active {
			animation: none;
		}
	}
</style>
