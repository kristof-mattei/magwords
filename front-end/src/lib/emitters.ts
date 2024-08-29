import { state, version } from "@/entrypoints/index";

export function sendMove(id: string, x: number, y: number): void {
    state.socket.emit("move", {
        id: Number.parseInt(id.slice(2), 10),
        v: version,
        x,
        y,
    });

    // ga("send", "event", {
    //     eventCategory: "word",
    //     eventAction: "move",
    // });
}
