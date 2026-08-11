<script setup lang="ts">
	withDefaults(
		defineProps<{
			as?: string;
		}>(),
		{
			as: 'div'
		}
	);
</script>

<template>
	<component :is="as" class="glass-card rounded-3xl p-8">
		<slot />
	</component>
</template>
