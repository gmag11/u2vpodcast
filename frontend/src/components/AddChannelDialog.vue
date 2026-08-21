<script setup lang="ts">
	import { ref, watch } from 'vue';
	import type { Channel } from '@/types';
	import AppButton from '@/components/AppButton.vue';
	import AppDialog from '@/components/AppDialog.vue';
	import AppInput from '@/components/AppInput.vue';
	import AppToggle from '@/components/AppToggle.vue';

	const props = defineProps<{
		open: boolean;
		channel?: Channel | null;
	}>();
	const emit = defineEmits<{
		(e: 'update:open', value: boolean): void;
		(e: 'save', channel: Channel): void;
		(e: 'cancel'): void;
	}>();

	const isEditing = ref(false);
	const title = ref('');
	const url = ref('');
	const active = ref(true);
	const max = ref(5);
	const first = ref('');

	function defaultFirst(): string {
		const d = new Date();
		d.setMonth(d.getMonth() - 1);
		const y = d.getFullYear();
		const m = String(d.getMonth() + 1).padStart(2, '0');
		const day = String(d.getDate()).padStart(2, '0');
		return `${y}-${m}-${day}`;
	}

	watch(
		() => props.open,
		(open) => {
			if (!open) return;
			isEditing.value = props.channel != null && props.channel.id > 0;
			title.value = props.channel?.title ?? '';
			url.value = props.channel?.url ?? '';
			active.value = props.channel?.active ?? true;
			max.value = props.channel?.max ?? 5;
			first.value = props.channel?.first
				? new Date(props.channel.first).toISOString().slice(0, 10)
				: defaultFirst();
		}
	);

	function handleSave() {
		if (!url.value.trim()) return;
		const channel: Channel = {
			...(props.channel ?? {}),
			id: props.channel?.id ?? 0,
			title: title.value || '',
			slug: props.channel?.slug ?? '',
			url: url.value.trim(),
			active: active.value,
			description: props.channel?.description ?? '',
			image: props.channel?.image ?? '',
			first: first.value ? new Date(first.value) : new Date(),
			max: Math.max(1, Number(max.value) || 5),
			created_at: props.channel?.created_at ?? new Date(),
			updated_at: new Date(),
			last_date: props.channel?.last_date ?? null,
			last_sync_at: props.channel?.last_sync_at ?? null,
			last_sync_ok: props.channel?.last_sync_ok ?? null,
			last_sync_error: props.channel?.last_sync_error ?? null
		};
		emit('save', channel);
	}

	function handleCancel() {
		emit('update:open', false);
		emit('cancel');
	}
</script>

<template>
	<AppDialog
		:open="open"
		:title="isEditing ? 'Edit Channel' : 'New Channel'"
		@update:open="(v: boolean) => emit('update:open', v)"
	>
		<form class="flex flex-col gap-5" @submit.prevent="handleSave">
			<div v-if="isEditing" class="flex flex-col gap-1.5">
				<label class="text-sm font-medium text-text" for="channel-title">Title</label>
				<AppInput id="channel-title" v-model="title" placeholder="Channel title" />
			</div>

			<div class="flex items-center gap-3">
				<AppToggle id="active-toggle" v-model="active" />
				<label class="text-sm font-medium text-text" for="active-toggle">Active</label>
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-sm font-medium text-text" for="channel-url">Url</label>
				<AppInput
					id="channel-url"
					v-model="url"
					type="url"
					placeholder="https://www.youtube.com/@channel"
				/>
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-sm font-medium text-text" for="max-episodes">
					Max number of episodes
				</label>
				<AppInput id="max-episodes" v-model="max" type="number" min="1" />
			</div>

			<div class="flex flex-col gap-1.5">
				<label class="text-sm font-medium text-text" for="first-episode-date">
					First episode date
				</label>
				<AppInput id="first-episode-date" v-model="first" type="date" />
			</div>

			<div class="mt-2 flex flex-col gap-3">
				<AppButton type="submit" class="w-full py-2.5">
					{{ isEditing ? 'Save changes' : 'Create channel' }}
				</AppButton>
				<AppButton type="button" variant="ghost" class="w-full py-2 text-sm" @click="handleCancel">
					Cancel
				</AppButton>
			</div>
		</form>
	</AppDialog>
</template>
