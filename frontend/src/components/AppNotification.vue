<script setup lang="ts">
	import { PhCheckCircle, PhInfo, PhWarning } from '@phosphor-icons/vue';
	import { useNotificationStore } from '@/stores/notification';

	const store = useNotificationStore();
</script>

<template>
	<Transition
		enter-active-class="transition ease-out duration-300"
		enter-from-class="translate-y-2 opacity-0"
		enter-to-class="translate-y-0 opacity-100"
		leave-active-class="transition ease-in duration-200"
		leave-from-class="translate-y-0 opacity-100"
		leave-to-class="translate-y-2 opacity-0"
	>
		<div
			v-if="store.current"
			class="fixed bottom-6 right-6 z-50 flex items-center gap-3 rounded-xl border border-outline bg-surface-card px-4 py-3 shadow-card"
			:class="{
				'text-error': store.current.type === 'error',
				'text-success': store.current.type === 'success',
				'text-accent-600': store.current.type === 'info'
			}"
		>
			<PhWarning v-if="store.current.type === 'error'" class="h-5 w-5" weight="fill" />
			<PhCheckCircle v-else-if="store.current.type === 'success'" class="h-5 w-5" weight="fill" />
			<PhInfo v-else class="h-5 w-5" weight="fill" />
			<span class="text-sm font-medium text-text">{{ store.current.message }}</span>
		</div>
	</Transition>
</template>
