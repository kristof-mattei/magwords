import type { State } from "./state";
import type { ClientMessage } from "./types";

export function sendMove(state: State, id: string, x: number, y: number): void {
    const message: ClientMessage = {
        type: "move",
        data: {
            id: Number.parseInt(id.slice(2), 10),
            v: state.version,
            x,
            y,
        },
    };

    state.socket.send(JSON.stringify(message));
}
