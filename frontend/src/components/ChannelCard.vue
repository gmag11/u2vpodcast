<script setup lang="ts">
	import {
		PhImage,
		PhLinkSimple,
		PhMicrophoneStage,
		PhPencilSimple,
		PhTrashSimple,
		PhYoutubeLogo
	} from '@phosphor-icons/vue';
	import { useRouter } from 'vue-router';
	import { baseEndpoint } from '@/lib/api/client';
	import type { Channel } from '@/types';

	const props = defineProps<{
		channel: Channel;
		refreshing?: boolean;
	}>();
	const emit = defineEmits<{
		(e: 'update', channel: Channel): void;
		(e: 'delete', channel: Channel): void;
		(e: 'cover-refresh', channel: Channel): void;
	}>();
	const router = useRouter();

	function openEpisodes() {
		router.push({ name: 'episodes', params: { channelId: String(props.channel.id) } });
	}
</script>

<template>
	<div
		class="glass-card flex h-full min-h-[300px] cursor-pointer flex-col rounded-3xl p-8"
		@click="openEpisodes"
	>
		<div class="mb-6 flex gap-6">
			<div
				class="h-[140px] w-[140px] shrink-0 overflow-hidden rounded-2xl border border-outline bg-surface-input shadow-lg"
			>
				<img
					v-if="channel.image"
					:src="channel.image"
					:alt="channel.title"
					class="h-full w-full object-cover"
				/>
				<div v-else class="flex h-full w-full items-center justify-center">
					<PhMicrophoneStage class="h-10 w-10 text-text-muted" weight="regular" />
				</div>
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
			<div class="group relative">
				<a
					:href="channel.url"
					target="_blank"
					rel="noopener noreferrer"
					aria-label="YouTube"
					class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
					@click.stop
				>
					<PhYoutubeLogo class="h-5 w-5" weight="regular" />
				</a>
				<span
					class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					Open on YouTube
				</span>
			</div>
			<div class="group relative">
				<a
					:href="`${baseEndpoint}/channels/${channel.slug}/feed.xml`"
					target="_blank"
					rel="noopener noreferrer"
					aria-label="Link"
					class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
					@click.stop
				>
					<PhLinkSimple class="h-5 w-5" weight="regular" />
				</a>
				<span
					class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					Get RSS feed
				</span>
			</div>
			<div class="group relative">
				<button
					type="button"
					aria-label="Refresh cover image"
					class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400 disabled:cursor-not-allowed disabled:opacity-50"
					:disabled="refreshing"
					@click.stop="emit('cover-refresh', channel)"
				>
					<PhImage :class="refreshing ? 'h-5 w-5 animate-spin' : 'h-5 w-5'" weight="regular" />
				</button>
				<span
					class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					Reload cover
				</span>
			</div>
			<div class="group relative">
				<button
					type="button"
					aria-label="Edit"
					class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
					@click.stop="emit('update', channel)"
				>
					<PhPencilSimple class="h-5 w-5" weight="regular" />
				</button>
				<span
					class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					Edit channel
				</span>
			</div>
			<div class="group relative">
				<button
					type="button"
					aria-label="Delete"
					class="cursor-pointer text-error transition-colors hover:opacity-80"
					@click.stop="emit('delete', channel)"
				>
					<PhTrashSimple class="h-5 w-5" weight="regular" />
				</button>
				<span
					class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					Delete channel and audio files
				</span>
			</div>
		</div>
	</div>
</template>
