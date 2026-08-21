<script setup lang="ts">
	import { computed, onMounted, ref } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhPlus } from '@phosphor-icons/vue';
	import { api } from '@/lib/api/client';
	import { useAuthStore } from '@/stores/auth';
	import { useNotificationStore } from '@/stores/notification';
	import { filterBySearchWords } from '@/lib/utils/list.filter';
	import { DEFAULT_SORT_DIRECTION, DEFAULT_SORT_KEY, sortChannels } from '@/lib/utils/channel.sort';
	import type { ChannelSortKey, SortDirection } from '@/lib/utils/channel.sort';
	import type { Channel } from '@/types';
	import AppButton from '@/components/AppButton.vue';
	import AppHeader from '@/components/AppHeader.vue';
	import ChannelCard from '@/components/ChannelCard.vue';
	import AddChannelDialog from '@/components/AddChannelDialog.vue';
	import ConfirmDialog from '@/components/ConfirmDialog.vue';
	import Pagination from '@/components/Pagination.vue';
	import SearchInput from '@/components/SearchInput.vue';
	import SortControl from '@/components/SortControl.vue';

	const route = useRoute();
	const router = useRouter();
	const auth = useAuthStore();
	const notification = useNotificationStore();

	const channels = ref<Channel[]>([]);
	const searchQuery = ref('');
	const perPage = ref(3);
	const showAddDialog = ref(false);
	const showConfirmDialog = ref(false);
	const editingChannel = ref<Channel | null>(null);
	const pendingDelete = ref<Channel | null>(null);
	const refreshingSlug = ref<string | null>(null);

	const SORT_KEY = 'channel-sort';

	const sortKey = ref<ChannelSortKey>(DEFAULT_SORT_KEY);
	const sortDirection = ref<SortDirection>(DEFAULT_SORT_DIRECTION);

	function resolveInitialSort(): { key: ChannelSortKey; direction: SortDirection } {
		const saved = localStorage.getItem(SORT_KEY);
		if (!saved) return { key: DEFAULT_SORT_KEY, direction: DEFAULT_SORT_DIRECTION };
		try {
			const parsed = JSON.parse(saved) as { key?: unknown; direction?: unknown };
			const key =
				parsed.key === 'title' || parsed.key === 'id' || parsed.key === 'last_date'
					? parsed.key
					: DEFAULT_SORT_KEY;
			const direction =
				parsed.direction === 'asc' || parsed.direction === 'desc'
					? parsed.direction
					: DEFAULT_SORT_DIRECTION;
			return { key, direction };
		} catch {
			return { key: DEFAULT_SORT_KEY, direction: DEFAULT_SORT_DIRECTION };
		}
	}

	function persistSort() {
		localStorage.setItem(
			SORT_KEY,
			JSON.stringify({ key: sortKey.value, direction: sortDirection.value })
		);
	}

	function setSortKey(key: ChannelSortKey) {
		sortKey.value = key;
		persistSort();
	}

	function setSortDirection(direction: SortDirection) {
		sortDirection.value = direction;
		persistSort();
	}

	const sortedChannels = computed(() =>
		sortChannels(channels.value, sortKey.value, sortDirection.value)
	);

	const filteredChannels = computed(() =>
		filterBySearchWords(sortedChannels.value, searchQuery.value, (c) =>
			[c.title, c.description, c.url, c.slug].join(' ')
		)
	);

	const currentPage = computed(() => {
		const raw = route.query.page;
		const page = raw ? parseInt(String(raw), 10) : 1;
		return Number.isFinite(page) && page >= 1 ? page : 1;
	});

	const maxPage = computed(() =>
		Math.max(1, Math.ceil(filteredChannels.value.length / perPage.value))
	);

	const pageNumbers = computed(() => Array.from({ length: maxPage.value }, (_, idx) => idx + 1));

	const paginatedChannels = computed(() => {
		const start = (currentPage.value - 1) * perPage.value;
		if (start >= filteredChannels.value.length)
			return filteredChannels.value.slice(0, perPage.value);
		return filteredChannels.value.slice(start, start + perPage.value);
	});

	const noSearchResults = computed(
		() => searchQuery.value.trim() !== '' && filteredChannels.value.length === 0
	);

	async function load() {
		const result = await api.getChannels();
		if (!result.ok || result.user == null) {
			auth.setUser(null);
			router.push({ name: 'login', query: { next: route.fullPath } });
			return;
		}
		auth.setUser(result.user);
		if (result.data) {
			channels.value = result.data as Array<Channel>;
		}
	}

	async function loadConfig() {
		const result = await api.getConfig();
		if (result.data && typeof result.data === 'object' && 'per_page' in (result.data as object)) {
			const per = (result.data as { per_page?: number }).per_page;
			if (per != null) perPage.value = per;
		}
	}

	function openNewDialog() {
		editingChannel.value = null;
		showAddDialog.value = true;
	}

	function openEditDialog(channel: Channel) {
		editingChannel.value = channel;
		showAddDialog.value = true;
	}

	function openDeleteDialog(channel: Channel) {
		pendingDelete.value = channel;
		showConfirmDialog.value = true;
	}

	async function saveChannel(channel: Channel) {
		if (channel.id > 0) {
			const result = await api.updateChannel(channel.slug, channel);
			if (result.ok) {
				const idx = channels.value.findIndex((c) => c.id === channel.id);
				// Apply the server response so fields the backend rejected or
				// normalized (e.g. empty title -> 400) are never flashed as saved.
				if (idx >= 0 && result.data) channels.value[idx] = result.data;
				notification.show('Channel updated', 'success');
			} else {
				notification.show(result.message || 'Failed to update channel', 'error');
			}
		} else {
			const result = await api.createChannel(channel);
			if (result.ok && result.data) {
				channels.value = [...channels.value, result.data];
				notification.show('Channel created. Update started.', 'success');
			} else {
				notification.show(result.message || 'Failed to create channel', 'error');
			}
		}
		showAddDialog.value = false;
	}

	async function deletePendingChannel() {
		if (!pendingDelete.value) return;
		const result = await api.deleteChannel(pendingDelete.value.slug);
		if (result.ok) {
			channels.value = channels.value.filter((c) => c.id !== pendingDelete.value?.id);
			notification.show('Channel deleted', 'success');
		} else {
			notification.show(result.message || 'Failed to delete channel', 'error');
		}
		pendingDelete.value = null;
		showConfirmDialog.value = false;
	}

	async function refreshChannelCover(channel: Channel) {
		refreshingSlug.value = channel.slug;
		const result = await api.refreshChannelImage(channel.slug);
		refreshingSlug.value = null;
		if (result.ok && result.data) {
			const idx = channels.value.findIndex((c) => c.id === channel.id);
			if (idx >= 0) channels.value[idx] = result.data;
			notification.show('Cover image updated', 'success');
		} else {
			notification.show(result.message || 'Failed to refresh cover image', 'error');
		}
	}

	function goToPage(page: number) {
		if (page < 1 || page > maxPage.value) return;
		router.push({ query: { ...route.query, page: String(page) } });
	}

	onMounted(async () => {
		const initial = resolveInitialSort();
		sortKey.value = initial.key;
		sortDirection.value = initial.direction;
		await loadConfig();
		await load();
	});
</script>

<template>
	<AppHeader>
		<template #brand-icon>
			<svg
				class="h-5 w-5"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				viewbox="0 0 24 24"
				xmlns="http://www.w3.org/2000/svg"
			>
				<path
					d="M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z"
					stroke-linecap="round"
					stroke-linejoin="round"
				></path>
			</svg>
		</template>
		<template #actions>
			<AppButton type="button" @click="openNewDialog">
				<span class="hidden sm:inline">Create New</span>
				<PhPlus class="h-4 w-4" weight="regular" />
			</AppButton>
		</template>
	</AppHeader>

	<main class="mx-auto max-w-[1440px] overflow-x-clip px-5 pb-28 pt-20 md:px-16">
		<div class="mb-12 pt-8">
			<h1 class="mb-2 font-display text-4xl font-semibold text-text">Dashboard</h1>
			<p class="text-lg text-text-muted">Manage your recent podcast episodes and content.</p>
		</div>

		<div class="mb-10 flex w-full flex-col gap-3 md:flex-row md:items-center">
			<SearchInput
				v-model="searchQuery"
				placeholder="Search channels…"
				class="min-w-0 flex-1 md:max-w-3xl md:mx-auto"
			/>
			<SortControl
				:model-value="sortKey"
				:direction="sortDirection"
				class="shrink-0 self-end"
				@update:model-value="setSortKey"
				@update:direction="setSortDirection"
			/>
		</div>

		<p v-if="noSearchResults" class="mt-4 text-text-muted">No results match your search.</p>

		<div v-else class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
			<ChannelCard
				v-for="channel in paginatedChannels"
				:key="channel.id"
				:channel="channel"
				:refreshing="refreshingSlug === channel.slug"
				@update="openEditDialog"
				@delete="openDeleteDialog"
				@cover-refresh="refreshChannelCover"
			/>
		</div>

		<Pagination
			:current-page="currentPage"
			:max-page="maxPage"
			:page-numbers="pageNumbers"
			@page="goToPage"
		/>
	</main>

	<AddChannelDialog v-model:open="showAddDialog" :channel="editingChannel" @save="saveChannel" />

	<ConfirmDialog
		v-model:open="showConfirmDialog"
		title="Warning"
		message="Are you sure you want to delete this channel?"
		@confirm="deletePendingChannel"
	/>
</template>
