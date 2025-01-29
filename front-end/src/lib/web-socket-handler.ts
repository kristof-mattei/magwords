import { purgeWords, setupMovable } from "@/lib/handlers";
import type { State } from "@/lib/state";
import type { Hup, MoveEventParameters, Poets, Word } from "@/lib/types";
import { reload, toHtmlWordId } from "@/lib/utils";

export class WebSocketHandler {
    private readonly state: State;
    private readonly wordIds: number[] = [];

    public constructor(state: State) {
        this.state = state;

        state.socket.on("move", onMove);
        state.socket.on("reload", onReload);
        state.socket.on("poets", this.onPoets.bind(this));
        state.socket.on("hup", this.onHup.bind(this));
        state.socket.on("words", this.onWords.bind(this));
    }

    public onWords(words: Word[]): void {
        const fridge = document.querySelector("#fridge");

        if (fridge === null) {
            return;
        }

        purgeWords(this.wordIds);

        // clear array, nasty, but this is how JavaScript wants to do it
        this.wordIds.length = 0;

        for (const word of words) {
            addWord(this.state, fridge, word);
            this.wordIds.push(word.id);
        }
    }

    public onPoets(data: Poets): void {
        if (data.count !== this.state.poets) {
            // eslint-disable-next-line @typescript-eslint/no-non-null-assertion -- we control html, element is there
            document.querySelector("#odo")!.innerHTML = data.count.toString(10);
        }

        this.state.poets = data.count;
    }

    public onHup(data: Hup, callback: ({ id }: { id: number }) => void): void {
        if (data.id === undefined) {
            console.log("Invalid ping");
            return;
        }

        if (data.v !== this.state.version) {
            reload();
        }

        // pong
        callback({ id: data.id });
    }
}

function onMove({ id, x, y }: MoveEventParameters): void {
    const time = 1500;
    const wordHtmlId = `#${toHtmlWordId(id)}`;

    // source: https://easings.net/
    // left
    const easeInOutQuad = "cubic-bezier(0.45, 0, 0.55, 1)";
    const easeOutCirc = "cubic-bezier(0.0, 0.55, 0.45, 1)";

    // top
    const easeInOutExpo = "cubic-bezier(0.87, 0, 0.13, 1)";
    const easeOutBack = "cubic-bezier(0.34, 1.56, 0.64, 1)";

    const element: HTMLElement | null = document.querySelector(wordHtmlId);

    if (element !== null) {
        const left: string = Math.round(Math.random()) === 0 ? easeInOutQuad : easeOutCirc;

        const top: string = Math.round(Math.random()) === 0 ? easeInOutExpo : easeOutBack;

        const transition = `left ${time}ms ${left}, top ${time}ms ${top}`;

        element.style.setProperty("transition", transition);
        element.style.setProperty("left", `${x}px`);
        element.style.setProperty("top", `${y}px`);

        element.addEventListener("transitionend", () => {
            element.style.setProperty("transition", "");
        });
    }
}

function onReload(_data: unknown): void {
    reload();
}

function addWord(state: State, fridge: Element, word: Word): void {
    console.log(`Adding word: ${word.word}`);
    const wordId = toHtmlWordId(word.id);

    // Delete any existing element with the same ID before adding the new one.
    document.querySelector(wordId)?.remove();

    const wordElement = document.createElement("span");

    wordElement.id = wordId;
    wordElement.classList.add("draggable");
    wordElement.classList.add("ui-widget-content");
    wordElement.classList.add("ui-draggable");
    wordElement.classList.add("ui-draggable-handle");
    wordElement.classList.add("word");
    wordElement.style.left = `${word.x}px`;
    wordElement.style.top = `${word.y}px`;

    wordElement.append(word.word);

    setupMovable(state, wordElement);

    fridge.append(wordElement);
}
