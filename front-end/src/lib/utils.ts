export function toHtmlWordId(word_id: number): string {
    return `#w-${word_id}`;
}

export function reload(): void {
    console.log("Reloading...");
    location.reload();
}
