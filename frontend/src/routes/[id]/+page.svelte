<script lang="ts">
    import type { PageData } from "./$types";
    import { LinkOutline } from 'flowbite-svelte-icons';
    import { base_endpoint } from '$lib/global';
    import type { Episode } from '$lib/types';
    import EpisodeCard from '$lib/components/EpisodeCard.svelte';
    import SearchInput from '$lib/components/SearchInput.svelte';
    import { filterBySearchWords } from '$lib/utils/helpers/list.filter';
    export let data: PageData;
    let searchQuery: string = '';

    $: filteredEpisodes = filterBySearchWords(
        data.episodes,
        searchQuery,
        (e) => [e.title, e.description, e.yt_id].join(' ')
    );
    $: noSearchResults = searchQuery.trim() !== '' && filteredEpisodes.length === 0;
</script>

<div class="flex justify-end p-4">
    <a href={`${base_endpoint}/channels/${data.channel_slug}/feed.xml`} target="_blank" rel="noopener noreferrer">
        <LinkOutline class="w-8 h-8 text-sky-400" />
    </a>
</div>
<div class="grid justify-items-center">
    <SearchInput bind:value={searchQuery} placeholder="Search episodes…" />
    {#if noSearchResults}
        <p class="mt-4 text-gray-500 dark:text-gray-400">No results match your search.</p>
    {:else}
        {#each filteredEpisodes as episode}
            <EpisodeCard {episode}>
            </EpisodeCard>
        {/each}
    {/if}
</div>

