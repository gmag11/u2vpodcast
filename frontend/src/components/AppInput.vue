<script setup lang="ts">
	import { useAttrs } from 'vue';

	withDefaults(
		defineProps<{
			modelValue: string | number;
			type?: string;
			placeholder?: string;
			id?: string;
			required?: boolean;
			leadingIcon?: boolean;
		}>(),
		{
			type: 'text',
			placeholder: '',
			id: undefined,
			required: false,
			leadingIcon: false
		}
	);
	const emit = defineEmits<{
		(e: 'update:modelValue', value: string | number): void;
	}>();
	const attrs = useAttrs();
</script>

<template>
	<div :class="leadingIcon ? 'relative' : ''">
		<div
			v-if="leadingIcon"
			class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3"
		>
			<slot name="icon" />
		</div>
		<input
			:id="id"
			:type="type"
			:value="modelValue"
			:placeholder="placeholder"
			:required="required"
			v-bind="attrs"
			:class="[
				'w-full rounded-xl border border-outline bg-surface-input px-4 py-3 text-text placeholder:text-text-muted/60 transition-all duration-150 focus:border-accent-500 focus:outline-none focus:ring-1 focus:ring-accent-500',
				leadingIcon ? 'pl-10' : ''
			]"
			@input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
		/>
	</div>
</template>
