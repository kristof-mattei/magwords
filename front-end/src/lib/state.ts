export class State {
    public socket: WebSocket;
    public readonly version: number;
    public poets: number;

    public constructor(socket: WebSocket, version: number) {
        this.socket = socket;
        this.version = version;
        this.poets = 0;
    }
}
