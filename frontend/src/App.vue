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
	watch(
		() => auth.isAuthenticated,
		(isAuthenticated) => {
			if (!isAuthenticated) player.halt();
			else playlists.load();
		}
	);
</script>

<template>
	<RouterView />
	<PersistentPlayer v-if="auth.isAuthenticated" />
	<AppLoading />
	<AppNotification />
</template>