import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// vitest globals are off, so Testing Library cannot auto-register its cleanup
afterEach(() => {
    cleanup();
});
