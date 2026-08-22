<script setup lang="ts">
	import { PhX } from '@phosphor-icons/vue';
	import { DialogContent, DialogOverlay, DialogPortal, DialogRoot, DialogTitle } from 'radix-vue';

	const props = defineProps<{
		open: boolean;
		title?: string;
		hideClose?: boolean;
	}>();
	const emit = defineEmits<{
		(e: 'update:open', value: boolean): void;
	}>();
</script>

<template>
	<DialogRoot :open="props.open" @update:open="(value: boolean) => emit('update:open', value)">
		<DialogPortal>
			<DialogOverlay
				class="fixed inset-0 z-10 bg-black/60 backdrop-blur-sm"
				:class="props.open ? 'data-[state=open]:animate-in' : ''"
			/>
			<DialogContent
				class="fixed inset-0 z-20 flex items-center justify-center p-4"
				@pointer-down-outside="() => emit('update:open', false)"
			>
				<div
					class="relative w-full max-w-md rounded-xl border border-outline bg-surface-card p-6 shadow-2xl"
					role="dialog"
					aria-modal="true"
				>
					<header class="mb-5 flex items-center justify-between">
						<div v-if="!hideClose" class="w-6" aria-hidden="true"></div>
						<DialogTitle class="text-lg font-semibold tracking-wide text-text">
							{{ props.title }}
						</DialogTitle>
						<button
							v-if="!hideClose"
							type="button"
							aria-label="$t('common.closeModal')"
							class="rounded-md p-1 text-text-muted transition-colors hover:text-text"
							@click="emit('update:open', false)"
						>
							<PhX class="h-5 w-5" weight="regular" />
						</button>
					</header>
					<slot />
				</div>
			</DialogContent>
		</DialogPortal>
	</DialogRoot>
</template>
