export class State {
    public socket: WebSocket;
    public readonly version: number;
    public poets: number;
    public fridgeWidth: number;
    public fridgeHeight: number;

    public constructor(socket: WebSocket, version: number) {
        this.socket = socket;
        this.version = version;
        this.poets = 0;
        this.fridgeWidth = 0;
        this.fridgeHeight = 0;
    }
}
