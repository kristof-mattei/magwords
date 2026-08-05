import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { App } from "./app";

describe("app", () => {
    it("renders", () => {
        render(<App />);

        expect(screen.getByText("Hello!")).toBeDefined();
    });
});
