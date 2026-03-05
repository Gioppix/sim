<script lang="ts">
    import type { WorldSnapshot } from '$lib/bindings/all';

    const WORLD_SIZE = 100;

    let { snapshot }: { snapshot: WorldSnapshot } = $props();
</script>

<div class="world">
    {#each snapshot.food as food (food.position.x + '_' + food.position.y)}
        <div
            class="food"
            style="left: {(food.position.x / WORLD_SIZE) * 100}%; top: {(food.position.y /
                WORLD_SIZE) *
                100}%; opacity: {Math.min(1, food.amount / 10)};"
        ></div>
    {/each}

    {#each snapshot.ants as ant (ant.id)}
        <div
            class="ant"
            class:queen={ant.queen}
            style="left: {(ant.position.x / WORLD_SIZE) * 100}%; top: {(ant.position.y /
                WORLD_SIZE) *
                100}%;"
        ></div>
    {/each}
</div>

<style>
    .world {
        position: relative;
        width: 600px;
        height: 600px;
        background: #f5f0e8;
        border: 1px solid #ccc;
        overflow: hidden;
    }

    .ant,
    .food {
        position: absolute;
        transform: translate(-50%, -50%);
        border-radius: 50%;
    }

    .ant {
        width: 8px;
        height: 8px;
        background: black;
    }

    .ant.queen {
        width: 12px;
        height: 12px;
        background: purple;
    }

    .food {
        width: 10px;
        height: 10px;
        background: saddlebrown;
    }
</style>
