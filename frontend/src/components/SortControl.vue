<script setup lang="ts">
	import { PhArrowDown, PhArrowUp } from '@phosphor-icons/vue';
	import type { ChannelSortKey, SortDirection } from '@/lib/utils/channel.sort';

	defineProps<{
		modelValue: ChannelSortKey;
		direction: SortDirection;
	}>();

	const emit = defineEmits<{
		(e: 'update:modelValue', value: ChannelSortKey): void;
		(e: 'update:direction', value: SortDirection): void;
	}>();

	const keys: Array<{ value: ChannelSortKey; label: string }> = [
		{ value: 'last_date', label: 'Last episode' },
		{ value: 'title', label: 'Title' },
		{ value: 'id', label: 'Id' }
	];
</script>

<template>
	<div role="group" aria-label="Sort channels" class="flex flex-wrap items-center gap-2">
		<div
			class="inline-flex items-center rounded-full border border-outline bg-surface-input p-1 shadow-inner"
		>
			<button
				v-for="key in keys"
				:key="key.value"
				type="button"
				:aria-pressed="modelValue === key.value"
				class="rounded-full px-3 py-1.5 text-sm font-medium transition-colors focus:outline-none focus:ring-1 focus:ring-accent-500"
				:class="
					modelValue === key.value ? 'bg-accent-500 text-white' : 'text-text-muted hover:text-text'
				"
				@click="emit('update:modelValue', key.value)"
			>
				{{ key.label }}
			</button>
		</div>

		<button
			type="button"
			:aria-pressed="direction === 'asc'"
			:aria-label="direction === 'asc' ? 'Sort ascending' : 'Sort descending'"
			class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline bg-surface-input text-text-muted shadow-inner transition-colors hover:text-text focus:outline-none focus:ring-1 focus:ring-accent-500"
			@click="emit('update:direction', direction === 'asc' ? 'desc' : 'asc')"
		>
			<PhArrowUp v-if="direction === 'asc'" class="h-4 w-4" weight="bold" />
			<PhArrowDown v-else class="h-4 w-4" weight="bold" />
		</button>
	</div>
</template>
