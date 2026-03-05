import { writable, type Readable } from 'svelte/store';
import type { BackToFront, FrontToBack, WorldSnapshot } from '$lib/bindings/all';

export const ssr = false;

export function load() {
    const state = writable<WorldSnapshot | null>(null);

    const ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
        const msg: FrontToBack = { type: 'Subscribe' };
        ws.send(JSON.stringify(msg));
    };

    ws.onmessage = (event) => {
        const msg: BackToFront = JSON.parse(event.data);
        if (msg.type === 'WorldStateUpdate') {
            state.set(msg.payload);
        }
    };

    ws.onerror = (event) => {
        state.set(null);
        console.error('WS error', event);
    };

    ws.onclose = () => {
        state.set(null);
    };

    return {
        state: { subscribe: state.subscribe } satisfies Readable<WorldSnapshot | null>
    };
}
