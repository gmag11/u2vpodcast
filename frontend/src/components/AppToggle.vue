<script setup lang="ts">
	import { SwitchRoot, SwitchThumb } from 'radix-vue';

	withDefaults(
		defineProps<{
			modelValue: boolean;
			id?: string;
		}>(),
		{
			id: undefined
		}
	);
	const emit = defineEmits<{
		(e: 'update:modelValue', value: boolean): void;
	}>();
</script>

<template>
	<SwitchRoot
		:id="id"
		:checked="modelValue"
		class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-300 focus:outline-none focus:ring-2 focus:ring-accent-500 focus:ring-offset-2"
		:class="modelValue ? 'bg-accent-600' : 'bg-surface-input border border-outline'"
		@update:checked="emit('update:modelValue', $event)"
	>
		<SwitchThumb
			class="block h-5 w-5 rounded-full bg-white shadow transition-transform duration-300"
			:class="modelValue ? 'translate-x-[22px]' : 'translate-x-[2px]'"
		/>
	</SwitchRoot>
</template>
