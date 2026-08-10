<script lang="ts">
    import type { PageData } from "./$types";
    import { LinkOutline } from 'flowbite-svelte-icons';
    import { base_endpoint } from '$lib/global';
    import type { Episode } from '$lib/types';
    import EpisodeCard from '$lib/components/EpisodeCard.svelte';
    export let data: PageData;

</script>

<div class="flex justify-end p-4">
    <a href={`${base_endpoint}/channels/${data.channel_slug}/feed.xml`} target="_blank" rel="noopener noreferrer">
        <LinkOutline class="w-8 h-8 text-sky-400" />
    </a>
</div>
<div class="grid justify-items-center">
{#each data.episodes as episode}
    <EpisodeCard {episode}>
    </EpisodeCard>
{/each}
</div>



