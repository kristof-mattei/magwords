import { describe, expect, it, vi } from "vitest";

vi.setConfig({ testTimeout: 1000 });

describe("description", () => {
    it("is real?", () => {
        expect(true).toBe(true);
    });
});
