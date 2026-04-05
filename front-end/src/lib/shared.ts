// convert an abstract coordinate (0..fridgeSize) to a CSS pixel position.
// the anchor point within the element slides proportionally from the
// leading edge (at coordinate=0) to the trailing edge (at coordinate=fridgeSize),
// so the element never overflows regardless of its rendered size.
export function coordinateToPixel(coordinate: number, elementSize: number, fridgeSize: number): number {
    if (elementSize >= fridgeSize) {
        return 0;
    }

    return (coordinate * (fridgeSize - elementSize)) / fridgeSize;
}

// inverse of coordinateToPixel
export function pixelToCoordinate(pixel: number, elementSize: number, fridgeSize: number): number {
    if (elementSize >= fridgeSize) {
        return 0;
    }

    return (pixel * fridgeSize) / (fridgeSize - elementSize);
}
