import type { Socket } from "socket.io-client";

export class State {
    public socket: Socket;
    public readonly version: number;
    public poets: number;

    public constructor(socket: Socket, version: number) {
        this.socket = socket;
        this.version = version;
        this.poets = 0;
    }
}
