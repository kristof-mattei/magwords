export interface Word {
    id: number;
    word: string;
    x: number;
    y: number;
}

export interface Config {
    fridge_height: number;
    fridge_width: number;
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
    | { data: Config; type: "config" }
    | { data: Hup; type: "hup" }
    | { data: MoveEventParameters; type: "move" }
    | { data: Poets; type: "poets" }
    | { data: Record<string, never>; type: "goodbye" }
    | { data: Word[]; type: "words" };

export type ClientMessage = { data: { id: number }; type: "pong" } | { data: MoveEventParameters; type: "move" };
