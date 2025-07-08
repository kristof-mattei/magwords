import { createRoot } from "react-dom/client";

import { App } from "@/components/app.tsx";

const container = document.querySelector("#root");

// eslint-disable-next-line @typescript-eslint/no-non-null-assertion -- We control html, there is a #root
const root = createRoot(container!);

root.render(<App />);
