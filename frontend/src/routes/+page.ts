import { writable, type Readable } from 'svelte/store';
import type { BackToFront, FrontToBack } from '$bindings/all';

export const ssr = false;

type State = { type: 'valid'; time: string } | { type: 'error'; message: string };

export function load() {
    const state = writable<State | null>(null);

    const ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
        const msg: FrontToBack = { type: 'Subscribe' };
        ws.send(JSON.stringify(msg));
    };

    ws.onmessage = (event) => {
        const msg: BackToFront = JSON.parse(event.data);
        if (msg.type === 'Time') {
            state.set({ type: 'valid', time: msg.payload });
        }
    };

    ws.onerror = (event) => {
        state.set({ type: 'error', message: event.type });
    };

    ws.onclose = () => {
        state.set({ type: 'error', message: 'WS Closed' });
    };

    return {
        state: { subscribe: state.subscribe } satisfies Readable<State | null>
    };
}
