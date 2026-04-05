export interface Word {
    id: number;
    word: string;
    x: number;
    y: number;
}

export interface Config {
    fridge_width: number;
    fridge_height: number;
}

export interface Poets {
    count: number;
}

export interface Hup {
    id: number | undefined;
    v: number;
}

export interface MoveEventParameters {
    id: number;
    v: number;
    x: number;
    y: number;
}

export type ServerMessage =
    | { type: "config"; data: Config }
    | { type: "goodbye"; data: Record<string, never> }
    | { type: "hup"; data: Hup }
    | { type: "move"; data: MoveEventParameters }
    | { type: "poets"; data: Poets }
    | { type: "words"; data: Word[] };

export type ClientMessage = { type: "move"; data: MoveEventParameters } | { type: "pong"; data: { id: number } };
