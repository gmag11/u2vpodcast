<script lang="ts">
	import type { Channel } from '$lib/utils/types';
	import { Modal, Button, Toggle, Label, Input } from 'flowbite-svelte';

	export let open: boolean = false;
	export let channel: Channel;
	export let onOkButtonClicked: () => void;

	function getDate() {
		const first: unknown = channel?.first;
		if (first == null) {
			console.log('No channel');
			return new Date().toISOString().split('T')[0];
		}
		if (typeof first == 'string') {
			return first.split('T')[0];
		}
		return (first as Date).toISOString().split('T')[0];
	}
	console.log(channel);

	function handleOkButtonClicked() {
		onOkButtonClicked();
		open = false;
	}
	function handleCancelButtonClicked() {
		open = false;
	}

	$: firstDate = getDate();
</script>

<Modal bind:open size="xs" class="w-full">
	<form class="flex flex-col space-y-6" action="">
		{#if channel.id == 0}
			<h3 class="mb-4 text-xl font-medium text-gray-900 dark:text-white">New Channel</h3>
		{:else}
			<h3 class="mb-4 text-xl font-medium text-gray-900 dark:text-white">Edit channel</h3>
		{/if}
		<Toggle bind:checked={channel.active}>Active</Toggle>
		<Label class="space-y-2">
			<span>Url</span>
			{#if channel.id == 0}
				<Input type="url" name="url" placeholder="url" bind:value={channel.url} required />
			{:else}
				<Input type="url" name="url" placeholder="url" bind:value={channel.url} readonly required />
			{/if}
		</Label>
		<Label class="space-y-2">
			<span>Max number of episodes</span>
			<Input
				type="number"
				name="max"
				placeholder="max"
				min="-1"
				bind:value={channel.max}
				oninput={(e) => {
					const el = e.currentTarget as HTMLInputElement;
					if (el != null && el.value != null) {
						channel.max = parseInt(el.value);
					}
				}}
				required
			/>
		</Label>
		<Label class="space-y-2">
			<span>First episode date</span>
			<Input
				type="date"
				name="first"
				placeholder="first"
				bind:value={firstDate}
				oninput={(e) => {
					const el = e.currentTarget as HTMLInputElement;
					if (el != null && el.value != null) {
						channel.first = new Date(el.value);
					}
				}}
				required
			/>
		</Label>
		<div class="flex flex-row md:space-y-0 md:space-x-4">
			{#if channel.id == 0}
				<Button onclick={handleOkButtonClicked}>Create channel</Button>
			{:else}
				<Button onclick={handleOkButtonClicked}>Update channel</Button>
			{/if}
			<Button onclick={handleCancelButtonClicked}>Cancel</Button>
		</div>
	</form>
</Modal>
