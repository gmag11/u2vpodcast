<script setup lang="ts">
	import { PhLinkSimple, PhPencilSimple, PhTrashSimple } from '@phosphor-icons/vue';
	import { baseEndpoint } from '@/lib/api/client';
	import type { Channel } from '@/types';

	defineProps<{
		channel: Channel;
	}>();
	const emit = defineEmits<{
		(e: 'update', channel: Channel): void;
		(e: 'delete', channel: Channel): void;
	}>();
</script>

<template>
	<div class="glass-card flex h-full min-h-[300px] flex-col rounded-3xl p-8">
		<div class="mb-6 flex gap-6">
			<div
				class="h-[140px] w-[140px] shrink-0 overflow-hidden rounded-2xl border border-outline shadow-lg"
			>
				<img
					v-if="channel.image"
					:src="channel.image"
					:alt="channel.title"
					class="h-full w-full object-cover"
				/>
			</div>
			<div class="flex flex-col">
				<h2 class="mb-2 line-clamp-2 font-display text-2xl font-semibold text-text">
					{{ channel.title }}
				</h2>
				<p class="line-clamp-3 text-base leading-relaxed text-text-muted">
					{{ channel.description }}
				</p>
			</div>
		</div>
		<div class="mt-auto flex justify-end gap-4 border-t border-outline pt-4">
			<a
				:href="`${baseEndpoint}/channels/${channel.slug}/feed.xml`"
				target="_blank"
				rel="noopener noreferrer"
				aria-label="Link"
				class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
			>
				<PhLinkSimple class="h-5 w-5" weight="regular" />
			</a>
			<button
				type="button"
				aria-label="Edit"
				class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
				@click="emit('update', channel)"
			>
				<PhPencilSimple class="h-5 w-5" weight="regular" />
			</button>
			<button
				type="button"
				aria-label="Delete"
				class="cursor-pointer text-error transition-colors hover:opacity-80"
				@click="emit('delete', channel)"
			>
				<PhTrashSimple class="h-5 w-5" weight="regular" />
			</button>
		</div>
	</div>
</template>
