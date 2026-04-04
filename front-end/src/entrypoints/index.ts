import { io } from "socket.io-client";

import { State } from "../lib/state";
import { WebSocketHandler } from "../lib/web-socket-handler";

import "bootstrap";

const socket = new WebSocketHandler(
    new State(
        io({
            transports: ["websocket"], // webtransport does not work
        }),
        1,
    ),
);

// this attaches the handlers, and because the handlers are attached, the socket does not get GC'ed
socket.init();
