import type { State } from "./state";
import type { ClientMessage } from "./types";

export interface Move {
    id: string;
    x: number;
    y: number;
}

export function sendMove(state: State, { id, x, y }: Move): void {
    const message: ClientMessage = {
        type: "move",
        data: {
            id: Number(id.slice(2)),
            v: state.version,
            x,
            y,
        },
    };

    state.socket.send(JSON.stringify(message));
}
