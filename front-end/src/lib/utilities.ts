// element dimensions including margin, matching the space the element
// actually occupies in the container
export function outerWidth(element: HTMLElement): number {
    const style = getComputedStyle(element);

    return element.offsetWidth + Number(style.marginLeft) + Number(style.marginRight);
}

export function outerHeight(element: HTMLElement): number {
    const style = getComputedStyle(element);

    return element.offsetHeight + Number(style.marginTop) + Number(style.marginBottom);
}

export function toHtmlWordId(wordId: number): string {
    return `w-${wordId}`;
}

export function reload(): void {
    console.log("Reloading...");
    location.reload();
}
