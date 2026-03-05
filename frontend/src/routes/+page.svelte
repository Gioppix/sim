<script lang="ts">
    import WorldView from '$lib/WorldView.svelte';

    let { data } = $props();
    const snapshot = $derived(data.state);
</script>

{#if $snapshot === null}
    <p>Connecting...</p>
{:else}
    <WorldView snapshot={$snapshot} />
{/if}
