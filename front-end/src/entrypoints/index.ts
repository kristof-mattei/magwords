import { State } from "../lib/state";
import { WebSocketHandler } from "../lib/web-socket-handler";

import "bootstrap";

function connect(): void {
    const protocol = location.protocol === "https:" ? "wss:" : "ws:";
    const ws = new WebSocket(`${protocol}//${location.host}/ws`);

    ws.addEventListener("open", () => {
        const state = new State(ws, 1);
        const handler = new WebSocketHandler(state);

        handler.init();
    });

    ws.addEventListener("close", () => {
        setTimeout(connect, 2000);
    });

    ws.addEventListener("error", () => {
        ws.close();
    });
}

connect();
