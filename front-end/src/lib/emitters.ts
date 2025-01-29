import type { State } from "@/lib/state";

export function sendMove(state: State, id: string, x: number, y: number): void {
    state.socket.emit("move", {
        id: Number.parseInt(id.slice(2), 10),
        v: state.version,
        x,
        y,
    });

    // ga("send", "event", {
    //     eventCategory: "word",
    //     eventAction: "move",
    // });
}
