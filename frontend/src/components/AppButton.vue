<script setup lang="ts">
	withDefaults(
		defineProps<{
			variant?: 'primary' | 'secondary' | 'ghost';
			type?: 'button' | 'submit';
		}>(),
		{
			variant: 'primary',
			type: 'button'
		}
	);
</script>

<template>
	<button
		:type="type"
		class="inline-flex items-center justify-center gap-2 rounded-lg px-5 py-2.5 text-sm font-medium transition-all duration-200 active:scale-95 disabled:opacity-50"
		:class="{
			'bg-primary-500 text-white hover:bg-primary-600 hover:shadow-glow': variant === 'primary',
			'border border-outline text-text-muted hover:text-text': variant === 'secondary',
			'text-text-muted hover:text-text': variant === 'ghost'
		}"
	>
		<slot />
	</button>
</template>
