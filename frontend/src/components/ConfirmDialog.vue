<script setup lang="ts">
	import { useI18n } from 'vue-i18n';
	import AppButton from '@/components/AppButton.vue';
	import AppDialog from '@/components/AppDialog.vue';

	defineProps<{
		open: boolean;
		title?: string;
		message?: string;
	}>();
	const emit = defineEmits<{
		(e: 'update:open', value: boolean): void;
		(e: 'confirm'): void;
	}>();

	const { t } = useI18n();
</script>

<template>
	<AppDialog
		:open="open"
		:title="title || t('common.warning')"
		hide-close
		@update:open="(v: boolean) => emit('update:open', v)"
	>
		<p class="mb-6 text-sm text-text-muted">{{ message }}</p>
		<div class="flex flex-col gap-3">
			<AppButton type="button" class="w-full py-2.5" variant="primary" @click="emit('confirm')">
				{{ $t('common.ok') }}
			</AppButton>
			<AppButton
				type="button"
				variant="ghost"
				class="w-full py-2 text-sm"
				@click="emit('update:open', false)"
			>
				{{ $t('common.cancel') }}
			</AppButton>
		</div>
	</AppDialog>
</template>
