import { purgeWords, setupMovable } from "./handlers";
import { coordinateToPixel } from "./shared";
import type { State } from "./state";
import type { ClientMessage, Config, Hup, MoveEventParameters, Poets, ServerMessage, Word } from "./types";
import { outerHeight, outerWidth, reload, toHtmlWordId } from "./utilities";

export class WebSocketHandler {
    private readonly state: State;
    private readonly wordIds: number[] = [];

    public constructor(state: State) {
        this.state = state;
    }

    public init(): void {
        this.state.socket.addEventListener("message", (event: MessageEvent<string>) => {
            const message = ((): ServerMessage | undefined => {
                try {
                    // eslint-disable-next-line @typescript-eslint/no-unsafe-return -- we trust the server protocol
                    return JSON.parse(event.data);
                } catch (error) {
                    console.error(`failed to parse message: ${String(error)}`);
                    return undefined;
                }
            })();

            if (message === undefined) {
                return;
            }

            switch (message.type) {
                case "config": {
                    this.onConfig(message.data);
                    break;
                }
                case "words": {
                    this.onWords(message.data);
                    break;
                }
                case "poets": {
                    this.onPoets(message.data);
                    break;
                }
                case "move": {
                    this.onMove(message.data);
                    break;
                }
                case "hup": {
                    this.onHup(message.data);
                    break;
                }
                case "goodbye": {
                    break;
                }
            }
        });
    }

    public onConfig(data: Config): void {
        this.state.fridgeWidth = data.fridge_width;
        this.state.fridgeHeight = data.fridge_height;

        const fridge = document.querySelector<HTMLElement>("#fridge");

        if (fridge !== null) {
            fridge.style.width = `${data.fridge_width}px`;
            fridge.style.height = `${data.fridge_height}px`;
        }
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

    public onMove({ id, x, y }: MoveEventParameters): void {
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
            const targetLeft = coordinateToPixel(x, outerWidth(element), this.state.fridgeWidth);
            const targetTop = coordinateToPixel(y, outerHeight(element), this.state.fridgeHeight);

            const left: string = Math.round(Math.random()) === 0 ? easeInOutQuad : easeOutCirc;

            const top: string = Math.round(Math.random()) === 0 ? easeInOutExpo : easeOutBack;

            const transition = `left ${time}ms ${left}, top ${time}ms ${top}`;

            element.style.setProperty("transition", transition);
            element.style.setProperty("left", `${targetLeft}px`);
            element.style.setProperty("top", `${targetTop}px`);

            element.addEventListener("transitionend", () => {
                element.style.setProperty("transition", "");
            });
        }
    }

    public onHup(data: Hup): void {
        if (data.id === undefined) {
            console.log("Invalid ping");
            return;
        }

        if (data.v !== this.state.version) {
            reload();
        }

        // pong
        const pong: ClientMessage = { type: "pong", data: { id: data.id } };
        this.state.socket.send(JSON.stringify(pong));
    }
}

function addWord(state: State, fridge: Element, word: Word): void {
    console.log(`Adding word: ${word.word}`);
    const wordId = toHtmlWordId(word.id);

    // Delete any existing element with the same ID before adding the new one.
    document.querySelector(wordId)?.remove();

    const wordElement = document.createElement("span");

    wordElement.id = wordId;
    wordElement.classList.add("draggable", "ui-widget-content", "ui-draggable", "ui-draggable-handle", "word");

    wordElement.append(word.word);

    setupMovable(state, wordElement);

    // append before positioning so we can read rendered dimensions
    fridge.append(wordElement);

    wordElement.style.left = `${coordinateToPixel(word.x, outerWidth(wordElement), state.fridgeWidth)}px`;
    wordElement.style.top = `${coordinateToPixel(word.y, outerHeight(wordElement), state.fridgeHeight)}px`;
}
