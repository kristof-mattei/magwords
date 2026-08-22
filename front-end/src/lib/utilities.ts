/* oxlint-disable unicorn/prefer-number-coercion -- computed style values carry a px suffix, Number() yields NaN */

// element dimensions including margin, matching the space the element
// actually occupies in the container
export function outerWidth(element: HTMLElement): number {
    const style = getComputedStyle(element);

    return element.offsetWidth + Number.parseFloat(style.marginLeft) + Number.parseFloat(style.marginRight);
}

export function outerHeight(element: HTMLElement): number {
    const style = getComputedStyle(element);

    return element.offsetHeight + Number.parseFloat(style.marginTop) + Number.parseFloat(style.marginBottom);
}

export function toHtmlWordId(wordId: number): string {
    return `w-${wordId}`;
}

export function reload(): void {
    console.log("Reloading...");
    location.reload();
}
