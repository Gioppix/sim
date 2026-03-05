import { writable, type Readable } from 'svelte/store';
import type { BackToFront, FrontToBack, WorldMetadata, WorldSnapshot } from '$bindings/all';

export const ssr = false;

export function load() {
    const state = writable<WorldSnapshot | null>(null);
    const metadata = writable<WorldMetadata | null>(null);

    const ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
        const msg: FrontToBack = { type: 'Subscribe' };
        ws.send(JSON.stringify(msg));
    };

    ws.onmessage = (event) => {
        const msg: BackToFront = JSON.parse(event.data);
        if (msg.type === 'WorldMetadata') {
            metadata.set(msg.payload);
        } else if (msg.type === 'WorldStateUpdate') {
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
        state: { subscribe: state.subscribe } satisfies Readable<WorldSnapshot | null>,
        metadata: { subscribe: metadata.subscribe } satisfies Readable<WorldMetadata | null>
    };
}
