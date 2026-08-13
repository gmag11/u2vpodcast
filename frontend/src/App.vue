<script setup lang="ts">
	import AppNotification from '@/components/AppNotification.vue';
	import AppLoading from '@/components/AppLoading.vue';
	import PersistentPlayer from '@/components/PersistentPlayer.vue';
</script>

<template>
	<RouterView />
	<PersistentPlayer />
	<AppLoading />
	<AppNotification />
</template>
