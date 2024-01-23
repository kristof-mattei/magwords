import "bootstrap";

import { io } from "socket.io-client";

import { setupHandlers } from "@/lib/listeners";

export const version = 1; // version

export const state = {
    socket: io("http://localhost:3000/", {
        transports: ["websocket"], // webtransport
    }),
    poets: 0,
};

setupHandlers();
