<script lang="ts">
    import WorldView from '$lib/WorldView.svelte';

    let { data } = $props();
    const snapshot = $derived(data.state);
    const metadata = $derived(data.metadata);
</script>

{#if $snapshot === null || $metadata === null}
    <p>Connecting...</p>
{:else}
    <WorldView snapshot={$snapshot} metadata={$metadata} />
{/if}
