import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./main";

const container = document.getElementById("app");
if (!container) throw new Error("missing #app container");

createRoot(container).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
