// Starts the MSW service worker in mock mode (`vite --mode mock`); the msw Vite plugin serves the worker script.
import { setupWorker } from "msw/browser";
import { handlers } from "./handlers";

export async function startMocks(): Promise<void> {
  const worker = setupWorker(...handlers);
  await worker.start({ onUnhandledFrame: "bypass", quiet: true });
}
