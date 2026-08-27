<script setup lang="ts">
	import { computed, onMounted } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhArrowLeft, PhCaretDown, PhCaretUp, PhPlay } from '@phosphor-icons/vue';
	import { useAuthStore } from '@/stores/auth';
	import { usePlaylistStore } from '@/stores/playlists';
	import { usePlayerStore } from '@/stores/player';
	import AppButton from '@/components/AppButton.vue';
	import AppHeader from '@/components/AppHeader.vue';
	import EpisodeCard from '@/components/EpisodeCard.vue';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();
	const playlists = usePlaylistStore();
	const player = usePlayerStore();

	const items = computed(() => playlists.items);

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

	async function move(index: number, delta: number) {
		const target = index + delta;
		if (target < 0 || target >= items.value.length) return;
		const next = [...items.value];
		const [moved] = next.splice(index, 1);
		next.splice(target, 0, moved);
		await playlists.reorder(next.map((episode) => episode.id));
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

		<div v-else class="flex w-full max-w-6xl flex-col gap-3">
			<div
				v-for="(episode, index) in items"
				:key="episode.id"
				class="flex w-full items-stretch gap-2"
			>
				<div class="flex shrink-0 flex-col items-center justify-center gap-1.5">
					<button
						type="button"
						class="flex h-9 w-9 items-center justify-center rounded-lg border border-outline text-text-muted transition-colors hover:text-text disabled:opacity-30 disabled:hover:text-text-muted"
						:disabled="index === 0"
						:aria-label="$t('playlist.moveUp')"
						@click="move(index, -1)"
					>
						<PhCaretUp class="h-4 w-4" weight="bold" />
					</button>
					<button
						type="button"
						class="flex h-9 w-9 items-center justify-center rounded-lg border border-outline text-text-muted transition-colors hover:text-text disabled:opacity-30 disabled:hover:text-text-muted"
						:disabled="index === items.length - 1"
						:aria-label="$t('playlist.moveDown')"
						@click="move(index, 1)"
					>
						<PhCaretDown class="h-4 w-4" weight="bold" />
					</button>
				</div>
				<EpisodeCard :episode="episode" :list="items" compact queue-source="playlist" />
			</div>
		</div>
	</main>
</template>
