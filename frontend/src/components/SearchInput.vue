<script setup lang="ts">
	import { PhMagnifyingGlass } from '@phosphor-icons/vue';

	withDefaults(
		defineProps<{
			modelValue: string;
			placeholder?: string;
		}>(),
		{
			placeholder: 'Search…'
		}
	);
	const emit = defineEmits<{
		(e: 'update:modelValue', value: string): void;
	}>();
</script>

<template>
	<div class="relative w-full">
		<PhMagnifyingGlass
			class="pointer-events-none absolute inset-y-0 left-4 flex items-center self-center text-sm text-text-muted"
			weight="regular"
		/>
		<input
			:value="modelValue"
			:placeholder="placeholder"
			class="w-full rounded-full border border-outline bg-surface-input py-3 pl-12 pr-4 text-text shadow-inner transition-all duration-150 placeholder:text-text-muted/60 focus:border-accent-500 focus:outline-none focus:ring-1 focus:ring-accent-500"
			@input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
		/>
	</div>
</template>
