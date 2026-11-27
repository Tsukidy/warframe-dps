/// <reference types="vitest/config" />
// Vite + Vitest configuration (frontend.md §1-§2, server-api.md §10).
//
// Modes:
// - default (`npm run dev`): the dev server proxies `/api` to the Rust server (`WFD_API`, default
//   http://127.0.0.1:7878, server-api.md §12.1 port).
// - `mock` (`npm run dev:mock`, the e2e default until the server exists): MSW answers `/api` in the browser from the
//   mini fixture bundle (src/mocks). The msw Vite plugin serves the worker script in this mode only, so production
//   builds contain neither MSW nor the worker.
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { msw } from "msw/vite";

export default defineConfig(({ mode }) => {
  const mock = mode === "mock";
  return {
    plugins: [react(), tailwindcss(), ...(mock ? [msw({ mode: "worker-only" })] : [])],
    define: {
      __WFD_MOCKS__: JSON.stringify(mock),
    },
    server: {
      port: 5173,
      proxy: mock
        ? undefined
        : { "/api": { target: process.env.WFD_API ?? "http://127.0.0.1:7878", changeOrigin: false } },
    },
    build: {
      target: "es2023",
      sourcemap: false,
      chunkSizeWarningLimit: 700,
    },
    test: {
      environment: "jsdom",
      setupFiles: ["./src/test/setup.ts"],
      include: ["src/**/*.test.{ts,tsx}"],
      restoreMocks: true,
      css: false,
    },
  };
});
