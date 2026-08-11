<script setup lang="ts">
	import AppNotification from '@/components/AppNotification.vue';
	import AppLoading from '@/components/AppLoading.vue';
</script>

<template>
	<RouterView />
	<AppLoading />
	<AppNotification />
</template>
