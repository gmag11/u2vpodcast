<script setup lang="ts">
	import { watch } from 'vue';
	import AppNotification from '@/components/AppNotification.vue';
	import AppLoading from '@/components/AppLoading.vue';
	import PersistentPlayer from '@/components/PersistentPlayer.vue';
	import { useAuthStore } from '@/stores/auth';
	import { usePlayerStore } from '@/stores/player';
	import { usePlaylistStore } from '@/stores/playlists';

	const auth = useAuthStore();
	const player = usePlayerStore();
	const playlists = usePlaylistStore();

	// The player is only meant to be used after logging in: hide it on the
	// login screen and stop any playback the moment the session disappears.
	// `immediate` also covers the reload case: the session is restored in
	// main.ts *before* this component mounts, so without it the playlist would
	// never load on a fresh page load and the card icons would stay unmarked
	// until the playlist page is visited.
	watch(
		() => auth.isAuthenticated,
		(isAuthenticated) => {
			if (!isAuthenticated) player.halt();
			else playlists.load();
		},
		{ immediate: true }
	);
</script>

<template>
	<RouterView />
	<PersistentPlayer v-if="auth.isAuthenticated" />
	<AppLoading />
	<AppNotification />
</template>
