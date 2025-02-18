import { sendMove } from "@/lib/emitters";
import type { State } from "@/lib/state";
import { toHtmlWordId } from "@/lib/utilities";

export function setupMovable(state: State, element: HTMLElement): void {
    let diffX = 0;
    let diffY = 0;
    let elementWidth = 0;
    let elementHeight = 0;
    let containerHeight = 0;
    let containerWidth = 0;
    let newLeft = 0;
    let newTop = 0;

    let startScrollX = 0;
    let startScrollY = 0;

    let lastScrollX = 0;
    let lastScrollY = 0;

    function mouseDown(event: MouseEvent): void {
        // get initial mousedown coordinated
        const mouseX = event.clientX;
        const mouseY = event.clientY;

        // get element top and left positions
        const elementOffsetLeft = element.offsetLeft;
        const elementOffsetTop = element.offsetTop;

        // get element
        elementWidth = element.offsetWidth;
        elementHeight = element.offsetHeight;
        // elementWidth = element.clientWidth;
        // elementHeight = element.clientHeight;

        // get container dimensions
        const container = element.offsetParent;

        if (container === null) {
            throw new Error("Element does not have valid parent");
        }

        // 2 px on each side
        containerWidth = container.clientWidth - 4;
        containerHeight = container.clientHeight - 4;

        // get diff from (0,0) to mousedown point
        diffX = mouseX - elementOffsetLeft;
        diffY = mouseY - elementOffsetTop;

        startScrollX = window.scrollX;
        startScrollY = window.scrollY;

        lastScrollX = window.scrollX;
        lastScrollY = window.scrollY;

        // we add these in the context of the current element
        document.addEventListener("mousemove", mouseMove);
        document.addEventListener("mouseup", mouseUp);

        document.addEventListener("scroll", scroll);
    }

    function scroll(_event: Event): void {
        const currentScrollX = window.scrollX;
        const currentScrollY = window.scrollY;

        const scrollX = currentScrollX - lastScrollX;
        const scrollY = currentScrollY - lastScrollY;

        diffX -= scrollX;
        diffY -= scrollY;

        moveElement(newLeft + diffX + (startScrollX + scrollX), newTop + diffY + (startScrollY + scrollY));

        lastScrollX = currentScrollX;
        lastScrollY = currentScrollY;
    }

    function mouseMove(event: MouseEvent): void {
        // get new mouse coordinates
        moveElement(event.clientX, event.clientY);
    }

    function moveElement(newMouseX: number, newMouseY: number): void {
        // calc new top, left pos of element
        newLeft = newMouseX - diffX;
        newTop = newMouseY - diffY;

        // calc new bottom, right pos of element
        const newRight = newLeft + elementWidth;
        const newBottom = newTop + elementHeight;

        if (
            newLeft < 0 ||
            newTop < 0 ||
            newLeft + elementWidth > containerWidth ||
            newTop + elementHeight > containerHeight
        ) {
            // if element is being dragged off left of the container...
            if (newLeft < 0) {
                newLeft = 0;
            }

            // if element is being dragged off top of the container...
            if (newTop < 0) {
                newTop = 0;
            }

            // if element is being dragged off right of the container...
            if (newRight > containerWidth) {
                newLeft = containerWidth - elementWidth;
            }

            // if element is being dragged off bottom of the container...
            if (newBottom > containerHeight) {
                newTop = containerHeight - elementHeight;
            }
        }

        element.style.left = `${newLeft}px`;
        element.style.top = `${newTop}px`;
    }

    function mouseUp(): void {
        // message.innerHTML = "plz move me";

        document.removeEventListener("mousemove", mouseMove);
        document.removeEventListener("mouseup", mouseUp);
        document.removeEventListener("scroll", scroll);

        sendMove(state, element.id, Math.round(newLeft), Math.round(newTop));
    }

    element.addEventListener("mousedown", mouseDown);
}

export function purgeWords(wordIds: number[]): void {
    for (const id of wordIds) {
        const htmlId = `#${toHtmlWordId(id)}`;

        document.querySelector(htmlId)?.remove();
    }
}
