<script setup lang="ts">
	import { computed, onMounted, ref } from 'vue';
	import { useRoute, useRouter } from 'vue-router';
	import { PhMicrophoneStage, PhPlus } from '@phosphor-icons/vue';
	import { api } from '@/lib/api/client';
	import { useAuthStore } from '@/stores/auth';
	import { useNotificationStore } from '@/stores/notification';
	import { filterBySearchWords } from '@/lib/utils/list.filter';
	import type { Channel } from '@/types';
	import AppButton from '@/components/AppButton.vue';
	import AppHeader from '@/components/AppHeader.vue';
	import ChannelCard from '@/components/ChannelCard.vue';
	import AddChannelDialog from '@/components/AddChannelDialog.vue';
	import ConfirmDialog from '@/components/ConfirmDialog.vue';
	import Pagination from '@/components/Pagination.vue';
	import SearchInput from '@/components/SearchInput.vue';

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

	const filteredChannels = computed(() =>
		filterBySearchWords(channels.value, searchQuery.value, (c) =>
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
				if (idx >= 0) channels.value[idx] = channel;
				notification.show('Channel updated', 'success');
			} else {
				notification.show(result.message || 'Failed to update channel', 'error');
			}
		} else {
			const result = await api.createChannel(channel);
			if (result.ok && result.data) {
				channels.value = [...channels.value, result.data];
				notification.show('Channel created', 'success');
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

	function goToPage(page: number) {
		if (page < 1 || page > maxPage.value) return;
		router.push({ query: { ...route.query, page: String(page) } });
	}

	onMounted(async () => {
		await loadConfig();
		await load();
	});
</script>

<template>
	<AppHeader>
		<template #brand-icon>
			<PhMicrophoneStage class="h-5 w-5" weight="fill" />
		</template>
		<template #search>
			<SearchInput v-model="searchQuery" placeholder="Search" />
		</template>
		<template #actions>
			<AppButton type="button" @click="openNewDialog">
				Create New
				<PhPlus class="h-4 w-4" weight="regular" />
			</AppButton>
		</template>
	</AppHeader>

	<main class="mx-auto max-w-[1440px] px-5 pb-28 pt-20 md:px-16">
		<div class="mb-12 pt-8">
			<h1 class="mb-2 font-display text-4xl font-semibold text-text">Dashboard</h1>
			<p class="text-lg text-text-muted">Manage your recent podcast episodes and content.</p>
		</div>

		<p v-if="noSearchResults" class="mt-4 text-text-muted">No results match your search.</p>

		<div v-else class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
			<ChannelCard
				v-for="channel in paginatedChannels"
				:key="channel.id"
				:channel="channel"
				@update="openEditDialog"
				@delete="openDeleteDialog"
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
