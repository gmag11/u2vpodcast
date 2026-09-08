<script setup lang="ts">
	import { PhCaretLeft, PhCaretRight } from '@phosphor-icons/vue';

	defineProps<{
		currentPage: number;
		maxPage: number;
		pageNumbers: number[];
	}>();
	const emit = defineEmits<{
		(e: 'page', page: number): void;
	}>();
</script>

<template>
	<div class="mt-6 flex items-center justify-center gap-1">
		<button
			type="button"
			aria-label="$t('pagination.previous')"
			:disabled="currentPage <= 1"
			class="rounded-lg border border-outline px-3 py-2 text-text-muted transition-colors hover:text-text disabled:opacity-40"
			@click="emit('page', currentPage - 1)"
		>
			<PhCaretLeft class="h-4 w-4" weight="regular" />
		</button>
		<button
			v-for="p in pageNumbers"
			:key="p"
			type="button"
			class="rounded-lg px-3 py-2 text-sm font-medium transition-colors"
			:class="
				p === currentPage
					? 'bg-primary-500 text-white'
					: 'border border-outline text-text-muted hover:text-text'
			"
			@click="emit('page', p)"
		>
			{{ p }}
		</button>
		<button
			type="button"
			aria-label="$t('pagination.next')"
			:disabled="currentPage >= maxPage"
			class="rounded-lg border border-outline px-3 py-2 text-text-muted transition-colors hover:text-text disabled:opacity-40"
			@click="emit('page', currentPage + 1)"
		>
			<PhCaretRight class="h-4 w-4" weight="regular" />
		</button>
	</div>
</template>
