export function toHtmlWordId(wordId: number): string {
    return `#w-${wordId}`;
}

export function reload(): void {
    console.log("Reloading...");
    location.reload();
}
