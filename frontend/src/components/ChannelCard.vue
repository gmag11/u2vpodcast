<script setup lang="ts">
	import {
		PhImage,
		PhMicrophoneStage,
		PhPencilSimple,
		PhRss,
		PhTrashSimple,
		PhYoutubeLogo
	} from '@phosphor-icons/vue';
	import { useRouter } from 'vue-router';
	import { computed } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { baseEndpoint } from '@/lib/api/client';
	import { lastEpisodeAge } from '@/lib/utils/channel.age';
	import { lastSyncAge } from '@/lib/utils/channel.sync.age';
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
	const { t } = useI18n();

	function openEpisodes() {
		router.push({ name: 'episodes', params: { channelId: String(props.channel.id) } });
	}

	const ageLabel = computed(() => lastEpisodeAge(props.channel.last_date));
	const syncAgeLabel = computed(() => lastSyncAge(props.channel.last_sync_at));
	const hasSyncStatus = computed(
		() => props.channel.last_sync_ok === true || props.channel.last_sync_ok === false
	);
	const syncStatusTooltip = computed(() =>
		t('card.syncTooltip', {
			age: syncAgeLabel.value,
			status: props.channel.last_sync_ok ? t('status.ok') : t('status.error')
		})
	);
	const episodeAgeTooltip = computed(() => t('card.lastEpisodeTooltip', { age: ageLabel.value }));
</script>

<template>
	<div
		class="glass-card relative flex h-full min-h-[300px] cursor-pointer flex-col rounded-3xl p-8"
		@click="openEpisodes"
	>
		<div class="group absolute right-4 top-4 z-10">
			<span
				v-if="ageLabel"
				class="block rounded-full bg-surface-high px-2.5 py-1 text-xs font-semibold text-text shadow"
			>
				{{ ageLabel }}
			</span>
			<span
				v-if="ageLabel"
				class="pointer-events-none absolute top-full right-0 z-20 mt-2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
			>
				{{ episodeAgeTooltip }}
			</span>
		</div>

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
			<div class="flex min-w-0 flex-col">
				<h2 class="mb-2 line-clamp-2 break-words font-display text-2xl font-semibold text-text">
					{{ channel.title }}
				</h2>
				<p class="line-clamp-3 break-words text-base leading-relaxed text-text-muted">
					{{ channel.description }}
				</p>
			</div>
		</div>
		<div class="mt-auto flex items-center justify-between gap-4 border-t border-outline pt-4">
			<div v-if="hasSyncStatus || syncAgeLabel" class="group relative flex items-center gap-1.5">
				<span
					v-if="hasSyncStatus"
					class="h-2.5 w-2.5 rounded-full shadow"
					:class="channel.last_sync_ok ? 'bg-emerald-500' : 'bg-error'"
				></span>
				<span
					v-if="syncAgeLabel"
					class="rounded-full bg-surface-high px-2.5 py-1 text-xs font-semibold text-text shadow"
				>
					{{ syncAgeLabel }}
				</span>
				<span
					class="pointer-events-none absolute bottom-full left-0 z-20 mb-2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
				>
					{{ syncStatusTooltip }}
				</span>
			</div>
			<div class="flex justify-end gap-4">
				<div class="group relative">
					<a
						:href="channel.url"
						target="_blank"
						rel="noopener noreferrer"
						:aria-label="$t('common.youtube')"
						class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
						@click.stop
					>
						<PhYoutubeLogo class="h-5 w-5" weight="regular" />
					</a>
					<span
						class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
					>
						{{ $t('card.openOnYouTube') }}
					</span>
				</div>
				<div class="group relative">
					<a
						:href="`${baseEndpoint}/channels/${channel.slug}/feed.xml`"
						target="_blank"
						rel="noopener noreferrer"
						:aria-label="$t('card.rssAria')"
						class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
						@click.stop
					>
						<PhRss class="h-5 w-5" weight="regular" />
					</a>
					<span
						class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
					>
						{{ $t('card.rssFeed') }}
					</span>
				</div>
				<div class="group relative">
					<button
						type="button"
						:aria-label="$t('card.refreshCoverAria')"
						class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400 disabled:cursor-not-allowed disabled:opacity-50"
						:disabled="refreshing"
						@click.stop="emit('cover-refresh', channel)"
					>
						<PhImage :class="refreshing ? 'h-5 w-5 animate-spin' : 'h-5 w-5'" weight="regular" />
					</button>
					<span
						class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
					>
						{{ $t('card.reloadCover') }}
					</span>
				</div>
				<div class="group relative">
					<button
						type="button"
						:aria-label="$t('card.edit')"
						class="cursor-pointer text-accent-500 transition-colors hover:text-accent-400"
						@click.stop="emit('update', channel)"
					>
						<PhPencilSimple class="h-5 w-5" weight="regular" />
					</button>
					<span
						class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
					>
						{{ $t('card.editLabel') }}
					</span>
				</div>
				<div class="group relative">
					<button
						type="button"
						:aria-label="$t('card.delete')"
						class="cursor-pointer text-error transition-colors hover:opacity-80"
						@click.stop="emit('delete', channel)"
					>
						<PhTrashSimple class="h-5 w-5" weight="regular" />
					</button>
					<span
						class="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 -translate-x-1/2 whitespace-nowrap rounded-md bg-surface-high px-2 py-1 text-xs text-text shadow-lg opacity-0 transition-opacity group-hover:opacity-100"
					>
						{{ $t('card.deleteLabel') }}
					</span>
				</div>
			</div>
		</div>
	</div>
</template>
