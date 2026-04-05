// element dimensions including margin, matching the space the element
// actually occupies in the container
export function outerWidth(element: HTMLElement): number {
    const style = getComputedStyle(element);
    return element.offsetWidth + Number.parseInt(style.marginLeft, 10) + Number.parseInt(style.marginRight, 10);
}

export function outerHeight(element: HTMLElement): number {
    const style = getComputedStyle(element);
    return element.offsetHeight + Number.parseInt(style.marginTop, 10) + Number.parseInt(style.marginBottom, 10);
}

export function toHtmlWordId(wordId: number): string {
    return `w-${wordId}`;
}

export function reload(): void {
    console.log("Reloading...");
    location.reload();
}
