import { io } from "socket.io-client";

import { State } from "@/lib/state.ts";
import { WebSocketHandler } from "@/lib/web-socket-handler.ts";

import "bootstrap";

// @ts-expect-error 6133
// eslint-disable-next-line @typescript-eslint/no-unused-vars -- keep reference
const _handler = new WebSocketHandler(
    new State(
        io({
            transports: ["websocket"], // webtransport does not work
        }),
        1,
    ),
);

// the socket from `io` keeps the handler alive
