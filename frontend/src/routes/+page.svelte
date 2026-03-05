<script lang="ts">
    let { data } = $props();

    const current_state = $derived(data.state);
</script>

{#if $current_state === null}
    <p>Waiting...</p>
{:else if $current_state.type === 'valid'}
    <p>{$current_state.time}</p>
{:else}
    <p>Error: {$current_state.message}</p>
{/if}
