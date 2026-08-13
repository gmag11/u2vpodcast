<script setup lang="ts">
	import { useLoadingStore } from '@/stores/loading';

	const store = useLoadingStore();
</script>

<template>
	<Transition
		enter-active-class="transition-opacity duration-200"
		enter-from-class="opacity-0"
		leave-active-class="transition-opacity duration-200"
		leave-to-class="opacity-0"
	>
		<div
			v-if="store.isLoading"
			class="fixed inset-0 z-40 flex items-center justify-center bg-black/40 backdrop-blur-sm"
		>
			<div
				class="flex flex-col items-center gap-3 rounded-xl bg-surface-card px-8 py-6 shadow-card"
			>
				<svg class="h-8 w-8 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
					<circle
						class="opacity-25"
						cx="12"
						cy="12"
						r="10"
						stroke="currentColor"
						stroke-width="4"
					></circle>
					<path
						class="opacity-75"
						fill="currentColor"
						d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"
					></path>
				</svg>
				<p v-if="store.message" class="text-sm font-medium text-text">{{ store.message }}</p>
			</div>
		</div>
	</Transition>
</template>
