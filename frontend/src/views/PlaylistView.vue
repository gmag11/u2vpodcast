<script setup lang="ts">
	import { computed, nextTick, onMounted, ref, watch } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { useRoute, useRouter } from 'vue-router';
	import { PhArrowLeft, PhDotsSix, PhPlay } from '@phosphor-icons/vue';
	import { VueDraggable, type DraggableEvent } from 'vue-draggable-plus';
	import { useAuthStore } from '@/stores/auth';
	import { useNotificationStore } from '@/stores/notification';
	import { usePlaylistStore } from '@/stores/playlists';
	import { usePlayerStore } from '@/stores/player';
	import type { Episode } from '@/types';
	import AppButton from '@/components/AppButton.vue';
	import AppHeader from '@/components/AppHeader.vue';
	import EpisodeCard from '@/components/EpisodeCard.vue';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();
	const notification = useNotificationStore();
	const playlists = usePlaylistStore();
	const player = usePlayerStore();
	const { t } = useI18n();

	const items = computed(() => playlists.items);
	const sortableItems = ref<Episode[]>([]);
	const dragging = ref(false);
	const committing = ref(false);
	const dragSnapshot = ref<Episode[]>([]);
	const activeEpisodeId = ref<number | null>(null);
	const keyboardSnapshot = ref<Episode[]>([]);
	const liveMessage = ref('');
	const reorderable = computed(() => sortableItems.value.length > 1);
	const reorderBusy = computed(() => committing.value || playlists.reorderPending);

	function ids(episodes: Episode[]) {
		return episodes.map((episode) => episode.id);
	}

	function sameOrder(left: number[], right: number[]) {
		return left.length === right.length && left.every((id, index) => id === right[index]);
	}

	function sameIdSet(left: number[], right: number[]) {
		return left.length === right.length && left.every((id) => right.includes(id));
	}

	function syncFromStore() {
		sortableItems.value = [...playlists.items];
	}

	function announce(key: string, episode: Episode, position: number) {
		liveMessage.value = t(key, {
			title: episode.title,
			position: position + 1,
			total: sortableItems.value.length
		});
	}

	function focusHandle(episodeId: number) {
		nextTick(() => {
			document.querySelector<HTMLButtonElement>(`[data-drag-handle="${episodeId}"]`)?.focus();
		});
	}

	async function commitOrder(previousItems: Episode[], movedEpisode: Episode) {
		const previousIds = ids(previousItems);
		const nextIds = ids(sortableItems.value);
		if (sameOrder(previousIds, nextIds)) {
			syncFromStore();
			return;
		}
		if (!sameIdSet(nextIds, ids(playlists.items))) {
			syncFromStore();
			return;
		}

		player.syncPlaylistOrder(sortableItems.value);
		committing.value = true;
		const result = await playlists.reorder(nextIds);
		syncFromStore();
		player.syncPlaylistOrder(playlists.items);
		committing.value = false;

		if (result.ok) {
			announce('playlist.dropped', movedEpisode, nextIds.indexOf(movedEpisode.id));
		} else {
			liveMessage.value = t('playlist.reorderFailed');
			notification.show(t('playlist.reorderFailed'), 'error');
		}
	}

	function onDragStart(event: DraggableEvent<Episode>) {
		dragging.value = true;
		dragSnapshot.value = [...sortableItems.value];
		const episode = sortableItems.value[event.oldIndex ?? -1];
		activeEpisodeId.value = episode?.id ?? null;
		if (episode) announce('playlist.pickedUp', episode, event.oldIndex ?? 0);
	}

	async function onDragEnd(event: DraggableEvent<Episode>) {
		const previousItems = dragSnapshot.value;
		const movedEpisode = previousItems[event.oldIndex ?? -1];
		dragging.value = false;
		dragSnapshot.value = [];
		activeEpisodeId.value = null;
		if (movedEpisode) await commitOrder(previousItems, movedEpisode);
		else syncFromStore();
	}

	function beginKeyboardReorder(episode: Episode) {
		keyboardSnapshot.value = [...sortableItems.value];
		activeEpisodeId.value = episode.id;
		announce(
			'playlist.pickedUp',
			episode,
			sortableItems.value.findIndex((item) => item.id === episode.id)
		);
	}

	function moveWithKeyboard(episode: Episode, delta: number) {
		const currentIndex = sortableItems.value.findIndex((item) => item.id === episode.id);
		const targetIndex = currentIndex + delta;
		if (targetIndex < 0 || targetIndex >= sortableItems.value.length) return;
		const next = [...sortableItems.value];
		const [moved] = next.splice(currentIndex, 1);
		next.splice(targetIndex, 0, moved);
		sortableItems.value = next;
		announce('playlist.moved', episode, targetIndex);
		focusHandle(episode.id);
	}

	async function finishKeyboardReorder(episode: Episode) {
		const previousItems = keyboardSnapshot.value;
		keyboardSnapshot.value = [];
		activeEpisodeId.value = null;
		await commitOrder(previousItems, episode);
		focusHandle(episode.id);
	}

	function cancelKeyboardReorder(episode: Episode) {
		const previousItems = keyboardSnapshot.value;
		if (sameIdSet(ids(previousItems), ids(playlists.items))) sortableItems.value = previousItems;
		else syncFromStore();
		keyboardSnapshot.value = [];
		activeEpisodeId.value = null;
		announce(
			'playlist.cancelled',
			episode,
			sortableItems.value.findIndex((item) => item.id === episode.id)
		);
		focusHandle(episode.id);
	}

	function onHandleKeydown(event: KeyboardEvent, episode: Episode) {
		if (reorderBusy.value) return;
		const isActive = activeEpisodeId.value === episode.id;
		if ((event.key === ' ' || event.key === 'Enter') && activeEpisodeId.value == null) {
			event.preventDefault();
			beginKeyboardReorder(episode);
		} else if ((event.key === ' ' || event.key === 'Enter') && isActive) {
			event.preventDefault();
			void finishKeyboardReorder(episode);
		} else if (event.key === 'ArrowUp' && isActive) {
			event.preventDefault();
			moveWithKeyboard(episode, -1);
		} else if (event.key === 'ArrowDown' && isActive) {
			event.preventDefault();
			moveWithKeyboard(episode, 1);
		} else if (event.key === 'Escape' && isActive) {
			event.preventDefault();
			cancelKeyboardReorder(episode);
		}
	}

	watch(
		() => playlists.items,
		() => {
			if (!dragging.value && activeEpisodeId.value == null && !reorderBusy.value) syncFromStore();
		},
		{ immediate: true }
	);

	async function load() {
		const result = await playlists.load();
		if (!result.ok || result.user == null) {
			auth.setUser(null);
			router.push({ name: 'login', query: { next: route.fullPath } });
			return;
		}
		auth.setUser(result.user);
		// The playlist payload carries each episode's playback progress; seed the
		// player store so resume works without per-episode requests.
		player.seedProgress(items.value);
	}

	// Play-all seeds the up-next queue with the rest of the playlist in stored
	// order, so auto-advance walks the playlist (playlist-capability).
	function playAll() {
		if (items.value.length > 0) {
			player.play(items.value[0], items.value, { queueSource: 'playlist' });
		}
	}

	onMounted(load);
</script>

<template>
	<AppHeader>
		<template #brand-icon>
			<svg
				class="h-5 w-5"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				viewbox="0 0 24 24"
				xmlns="http://www.w3.org/2000/svg"
			>
				<path
					d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
					stroke-linecap="round"
					stroke-linejoin="round"
				></path>
			</svg>
		</template>
	</AppHeader>

	<main class="flex min-h-screen flex-col items-center px-4 pb-28 pt-28">
		<div class="mb-8 flex w-full max-w-6xl items-center gap-4">
			<button
				type="button"
				aria-label="$t('header.backChannels')"
				class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-outline text-text-muted transition-colors hover:text-text"
				@click="router.push({ name: 'channels' })"
			>
				<PhArrowLeft class="h-5 w-5" weight="regular" />
			</button>
			<h1 class="truncate font-display text-2xl font-semibold text-text">
				{{ $t('playlist.title') }} ({{ items.length }})
			</h1>
			<AppButton v-if="items.length > 0" class="ml-auto" @click="playAll">
				<PhPlay class="h-4 w-4" weight="fill" />
				{{ $t('playlist.playAll') }}
			</AppButton>
		</div>

		<div v-if="items.length === 0" class="mt-10 text-center">
			<p class="font-display text-xl font-semibold text-text">{{ $t('playlist.emptyTitle') }}</p>
			<p class="mt-2 text-sm text-text-muted">
				{{ $t('playlist.emptyBody') }}
			</p>
		</div>

		<VueDraggable
			v-else
			v-model="sortableItems"
			class="playlist-list flex w-full max-w-6xl flex-col gap-3"
			draggable=".playlist-row"
			handle=".playlist-drag-handle"
			:disabled="!reorderable || reorderBusy"
			:animation="180"
			:delay="180"
			:delay-on-touch-only="true"
			:touch-start-threshold="6"
			:fallback-tolerance="4"
			:force-fallback="true"
			:scroll="true"
			:bubble-scroll="true"
			:scroll-sensitivity="80"
			:scroll-speed="14"
			ghost-class="playlist-sortable-ghost"
			chosen-class="playlist-sortable-chosen"
			drag-class="playlist-sortable-drag"
			@start="onDragStart"
			@end="onDragEnd"
		>
			<div
				v-for="episode in sortableItems"
				:key="episode.id"
				class="playlist-row relative flex w-full min-w-0 items-stretch gap-1.5 sm:gap-2"
				:data-episode-id="episode.id"
			>
				<div v-if="reorderable" class="flex shrink-0 items-center justify-center">
					<button
						type="button"
						class="playlist-drag-handle flex h-11 w-11 touch-none items-center justify-center rounded-lg text-text-muted transition-colors hover:bg-surface-high hover:text-text focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-500 active:cursor-grabbing disabled:cursor-not-allowed disabled:opacity-40"
						:class="
							activeEpisodeId === episode.id ? 'cursor-grabbing text-accent-500' : 'cursor-grab'
						"
						:disabled="reorderBusy || (activeEpisodeId != null && activeEpisodeId !== episode.id)"
						:aria-label="t('playlist.dragHandle', { title: episode.title })"
						:aria-pressed="activeEpisodeId === episode.id"
						:aria-describedby="reorderBusy ? 'playlist-reorder-pending' : undefined"
						:data-drag-handle="episode.id"
						@keydown="onHandleKeydown($event, episode)"
					>
						<PhDotsSix class="h-6 w-6" weight="bold" aria-hidden="true" />
					</button>
				</div>
				<EpisodeCard
					class="min-w-0 flex-1"
					:episode="episode"
					:list="sortableItems"
					compact
					queue-source="playlist"
				/>
			</div>
		</VueDraggable>

		<p class="sr-only" role="status" aria-live="assertive" aria-atomic="true">
			{{ liveMessage }}
		</p>
		<p v-if="reorderBusy" id="playlist-reorder-pending" class="sr-only">
			{{ $t('playlist.reorderPending') }}
		</p>
	</main>
</template>

<style scoped>
	.playlist-list :deep(.playlist-sortable-ghost) {
		opacity: 0.35;
	}

	.playlist-list :deep(.playlist-sortable-ghost)::before {
		position: absolute;
		top: -0.45rem;
		left: 0;
		right: 0;
		height: 3px;
		border-radius: 999px;
		background: var(--accent-500);
		content: '';
	}

	.playlist-list :deep(.playlist-sortable-chosen) {
		z-index: 1;
	}

	.playlist-list :deep(.playlist-sortable-drag) {
		opacity: 0.9;
		filter: drop-shadow(0 12px 24px rgb(0 0 0 / 0.28));
	}
</style>
