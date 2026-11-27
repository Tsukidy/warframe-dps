import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { App } from "./App";

async function start() {
  if (__WFD_MOCKS__) {
    const { startMocks } = await import("./mocks/browser");
    await startMocks();
  }
  const el = document.getElementById("root");
  if (!el) throw new Error("#root missing");
  createRoot(el).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

void start();
