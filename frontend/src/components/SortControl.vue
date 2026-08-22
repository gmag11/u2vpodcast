<script setup lang="ts">
	import { computed } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { PhArrowDown, PhArrowUp, PhCalendarBlank } from '@phosphor-icons/vue';
	import type { ChannelSortKey, SortDirection } from '@/lib/utils/channel.sort';

	const props = defineProps<{
		modelValue: ChannelSortKey;
		direction: SortDirection;
	}>();

	const emit = defineEmits<{
		(e: 'update:modelValue', value: ChannelSortKey): void;
		(e: 'update:direction', value: SortDirection): void;
	}>();

	const { t } = useI18n();

	const keys = computed<
		Array<{ value: ChannelSortKey; label: string; tooltip: string; icon?: boolean }>
	>(() => [
		{
			value: 'last_date',
			label: t('sort.lastEpisode'),
			tooltip: t('sort.byLastEpisode'),
			icon: true
		},
		{ value: 'title', label: t('sort.az'), tooltip: t('sort.byTitle') },
		{ value: 'id', label: t('sort.id'), tooltip: t('sort.byId') }
	]);

	const directionAria = computed(() =>
		props.direction === 'asc' ? t('sort.ascending') : t('sort.descending')
	);
</script>

<template>
	<div role="group" :aria-label="$t('sort.channels')" class="flex flex-nowrap items-center gap-2">
		<div
			class="inline-flex items-center rounded-full border border-outline bg-surface-input p-1 shadow-inner"
		>
			<button
				v-for="key in keys"
				:key="key.value"
				type="button"
				:aria-pressed="modelValue === key.value"
				:aria-label="key.label"
				:title="key.tooltip"
				class="rounded-full px-3 py-1.5 text-sm font-medium transition-colors focus:outline-none focus:ring-1 focus:ring-accent-500"
				:class="
					modelValue === key.value ? 'bg-accent-500 text-white' : 'text-text-muted hover:text-text'
				"
				@click="emit('update:modelValue', key.value)"
			>
				<PhCalendarBlank v-if="key.icon" class="h-4 w-4" weight="regular" />
				<span v-else>{{ key.label }}</span>
			</button>
		</div>

		<button
			type="button"
			:aria-pressed="direction === 'asc'"
			:aria-label="directionAria"
			:title="directionAria"
			class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-outline bg-surface-input text-text-muted shadow-inner transition-colors hover:text-text focus:outline-none focus:ring-1 focus:ring-accent-500"
			@click="emit('update:direction', direction === 'asc' ? 'desc' : 'asc')"
		>
			<PhArrowUp v-if="direction === 'asc'" class="h-4 w-4" weight="bold" />
			<PhArrowDown v-else class="h-4 w-4" weight="bold" />
		</button>
	</div>
</template>
