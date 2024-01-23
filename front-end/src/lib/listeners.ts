import { state, version } from "@/entrypoints/index";
import { setupMovable } from "@/lib/handlers";
import { type Word } from "@/lib/types";
import { reload, toHtmlWordId } from "@/lib/utils";

export function setupHandlers(): void {
    state.socket.on("move", onMove);
    state.socket.on("reload", onReload);
    state.socket.on("poets", onPoets);
    state.socket.on("hup", onHup);
    state.socket.on("words", onWords);
}

function onWords(words: Word[]): void {
    const fridge = document.getElementById("fridge");

    if (fridge === null) {
        return;
    }

    for (const word of words) {
        addWord(fridge, word);
    }
}

function onHup(
    data: { id: number; v: number },
    _callback: ({ id }: { id: number }) => void,
): void {
    if (data?.id === undefined) {
        console.log("Invalid ping");
        return;
    }
    if (data.v !== version) {
        reload();
    }

    // pong
    // callback({ id: data.id });
}

function onPoets(data: { count: number }): void {
    if (data.count !== state.poets) {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        document.getElementById("odo")!.innerHTML = data.count.toString(10);
    }

    state.poets = data.count;
}

function onMove({ id, x, y }: { id: number; x: number; y: number }): void {
    const time = 1500;
    const wordId = toHtmlWordId(id);

    // source: https://easings.net/
    // left
    const easeInOutQuad = "cubic-bezier(0.45, 0, 0.55, 1)";
    const easeOutCirc = "cubic-bezier(0.0, 0.55, 0.45, 1)";

    // top
    const easeInOutExpo = "cubic-bezier(0.87, 0, 0.13, 1)";
    const easeOutBack = "cubic-bezier(0.34, 1.56, 0.64, 1)";

    const element = document.getElementById(wordId);

    if (element !== null) {
        const left: string = Math.round(Math.random())
            ? easeInOutQuad
            : easeOutCirc;

        const top: string = Math.round(Math.random())
            ? easeInOutExpo
            : easeOutBack;

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

function addWord(fridge: HTMLElement, word: Word): void {
    const wordId = toHtmlWordId(word.id);

    // Delete any existing element with the same ID before adding the new one.
    document.getElementById(wordId)?.remove();

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

    setupMovable(wordElement);

    fridge.appendChild(wordElement);
}
