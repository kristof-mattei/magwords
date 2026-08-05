import { describe, expect, it } from "vitest";

import { outerHeight, outerWidth } from "./utilities";

function elementWithMargins(margins: Partial<CSSStyleDeclaration>): HTMLElement {
    const element = document.createElement("div");

    Object.assign(element.style, margins);

    return element;
}

describe("outerWidth", () => {
    it("adds px-suffixed margins to the element width", () => {
        const element = elementWithMargins({ marginLeft: "2px", marginRight: "3.5px" });

        Object.defineProperty(element, "offsetWidth", { value: 50 });

        expect(outerWidth(element)).toBe(55.5);
    });
});

describe("outerHeight", () => {
    it("adds px-suffixed margins to the element height", () => {
        const element = elementWithMargins({ marginTop: "1px", marginBottom: "0px" });

        Object.defineProperty(element, "offsetHeight", { value: 20 });

        expect(outerHeight(element)).toBe(21);
    });
});
