<script lang="ts">
	import type { PageData } from './$types';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import ChannelDialog from '$lib/components/ChannelDialog.svelte';
	import ChannelCard from '$lib/components/ChannelCard.svelte';
	import SearchInput from '$lib/components/SearchInput.svelte';
	import { PaginationItem } from 'flowbite-svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { ChevronLeftOutline, ChevronRightOutline } from 'flowbite-svelte-icons';
	import { GradientButton } from 'flowbite-svelte';
	import { CirclePlusSolid } from 'flowbite-svelte-icons';
	import type { Channel } from '$lib/types';
	import { base_endpoint } from '$lib/global';
	import { filterBySearchWords } from '$lib/utils/helpers/list.filter';
	export let data: PageData;
	let channel: Channel;
	let onDialogButtonClicked: () => void = () => {};
	let showConfirmDialog = false;
	let showChannelDialog = false;

	let channels: Channel[] = data.channels as Channel[];
	let perPage: number = data.per_page ?? 3;
	let searchQuery: string = '';

	async function deleteChannel(channelToDelete: Channel) {
		console.log('deleteChannel');
		console.log(channelToDelete);
		const request = await fetch(`${base_endpoint}/api/1.0/channels/${channelToDelete.slug}/`, {
			method: 'DELETE',
			headers: {
				Accept: 'application/json'
			}
		});
		const response = await request.json();
		if (response.status) {
			channels = channels.filter((item) => item.id != channelToDelete.id);
		}
	}

	async function updateChannel(channelToUpdate: Channel) {
		console.log('updateChannel');
		console.log(channelToUpdate);
		const request = await fetch(`${base_endpoint}/api/1.0/channels/${channelToUpdate.slug}/`, {
			method: 'PUT',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(channelToUpdate)
		});
		const response = await request.json();
		if (response.status) {
			channel = response.data;
		}
	}

	async function newChannel(newChannel: Channel) {
		console.log('newChannel');
		console.log(newChannel);
		const request = await fetch(`${base_endpoint}/api/1.0/channels/`, {
			method: 'POST',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(newChannel)
		});
		const response = await request.json();
		console.log(response);
		if (response.status) {
			channel = response.data as Channel;
			console.log(channel);
			channels = [...channels, channel];
		}
	}

	function onUpdateChannelButtonClicked(channelToUpdate: Channel) {
		console.log(channelToUpdate);
		channel = channelToUpdate;
		onDialogButtonClicked = () => updateChannel(channel);
		showChannelDialog = true;
	}
	function onDeleteChannelButtonClicked(channel: Channel) {
		console.log('onDeleteChannelButtonClicked');
		deleteChannel(channel);
	}
	function onNewChannelButtonClicked() {
		console.log('onNewChannelButtonClicked');
		channel = {
			id: 0,
			title: '',
			slug: '',
			description: '',
			image: '',
			active: true,
			url: '',
			max: 5,
			first: new Date(),
			created_at: new Date(),
			updated_at: new Date()
		};
		onDialogButtonClicked = () => newChannel(channel);
		showChannelDialog = true;
	}

	function getCurrentPage(currentPageString: string | null) {
		let currentPage = 1;
		if (currentPageString != null) {
			currentPage = parseInt(currentPageString);
		}
		if (!Number.isFinite(currentPage) || currentPage < 1) currentPage = 1;
		console.log(`currentPage: ${currentPage}`);
		return currentPage;
	}

	function getPaginatedChannels(pageNumber: number, channelList: Channel[]) {
		const total = channelList.length;
		let start = (pageNumber - 1) * perPage;
		if (start >= total) start = 0;
		let end = start + perPage;
		console.log(`start: ${start}, end: ${end}, total: ${total}`);
		return channelList.slice(start, end);
	}

	function pageUrl(pageNumber: number) {
		const url = new URL($page.url.href);
		url.searchParams.set('page', `${pageNumber}`);
		return `${url.pathname}${url.search}`;
	}

	function goToPage(pageNumber: number) {
		const maxPage = Math.max(1, Math.ceil(filteredChannels.length / perPage));
		if (pageNumber >= 1 && pageNumber <= maxPage) {
			goto(pageUrl(pageNumber));
		}
	}

	$: currentPage = getCurrentPage($page.url.searchParams.get('page'));
	$: filteredChannels = filterBySearchWords(channels, searchQuery, (c) =>
		[c.title, c.description, c.url, c.slug].join(' ')
	);
	$: paginatedChannels = getPaginatedChannels(currentPage, filteredChannels);
	$: maxPage = Math.max(1, Math.ceil(filteredChannels.length / perPage));
	$: pageNumbers = Array.from({ length: maxPage }, (_, idx) => idx + 1);
	$: noSearchResults = searchQuery.trim() !== '' && filteredChannels.length === 0;
</script>

<div id="channels" class="grid justify-items-center">
	<GradientButton onclick={onNewChannelButtonClicked} class="mb-4">
		<CirclePlusSolid />
	</GradientButton>
	<SearchInput bind:value={searchQuery} placeholder="Search channels…" />
	{#if noSearchResults}
		<p class="mt-4 text-gray-500 dark:text-gray-400">No results match your search.</p>
	{:else}
		{#each paginatedChannels as channel (channel.id)}
			<ChannelCard {channel} {onUpdateChannelButtonClicked} {onDeleteChannelButtonClicked} />
		{/each}
	{/if}
</div>
<div class="flex items-center justify-center gap-1 mt-4">
	<PaginationItem size="large" onclick={() => goToPage(currentPage - 1)} class="rounded-s-lg">
		<span class="sr-only">Previous</span>
		<ChevronLeftOutline class="w-5 h-5" />
	</PaginationItem>
	{#each pageNumbers as p (p)}
		<PaginationItem
			size="large"
			active={p === currentPage}
			onclick={() => goToPage(p)}
			class="rounded-lg"
		>
			{p}
		</PaginationItem>
	{/each}
	<PaginationItem size="large" onclick={() => goToPage(currentPage + 1)} class="rounded-e-lg">
		<span class="sr-only">Next</span>
		<ChevronRightOutline class="w-5 h-5" />
	</PaginationItem>
</div>

<ChannelDialog bind:open={showChannelDialog} {channel} onOkButtonClicked={onDialogButtonClicked} />

<ConfirmDialog
	bind:open={showConfirmDialog}
	title="Warning"
	message="Are you sure?"
	onOkButtonClicked={() => deleteChannel(channel)}
></ConfirmDialog>
