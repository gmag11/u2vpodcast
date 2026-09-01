<script setup lang="ts">
	withDefaults(
		defineProps<{
			id: string;
			text: string;
			align?: 'left' | 'center' | 'right';
		}>(),
		{ align: 'center' }
	);
</script>

<template>
	<span class="group relative inline-flex">
		<slot :describedby="id" />
		<span
			:id="id"
			role="tooltip"
			class="pointer-events-none absolute bottom-full z-50 mb-2 w-max whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs font-medium text-text opacity-0 shadow-card transition-opacity group-hover:opacity-100 group-focus-within:opacity-100"
			:class="{
				'left-0': align === 'left',
				'left-1/2 -translate-x-1/2': align === 'center',
				'right-0': align === 'right'
			}"
		>
			{{ text }}
		</span>
	</span>
</template>
