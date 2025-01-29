export interface Word {
    id: number;
    word: string;
    x: number;
    y: number;
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
