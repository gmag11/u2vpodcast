<script lang="ts">
	import { Input } from 'flowbite-svelte';
	export let value: string;
	export let placeholder: string = 'Search…';
</script>

<div class="flex justify-center py-2">
	<Input type="text" bind:value {placeholder} class="w-full md:max-w-xl" />
</div>
