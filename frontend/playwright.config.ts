// Playwright (frontend.md §2; testing.md §8; work-breakdown.md T12).
//
// Two targets, chosen by WFD_E2E_TARGET:
// - "server": the real binary, `warframe-dps --no-browser --port 0` (server-api.md §1.1) with image fetching off and
//   a throw-away data directory; the port the server prints at startup (§1.2 step 5) is captured into WFD_PORT.
//   Binary: WFD_BIN (default ../target/release/warframe-dps); bundle: WFD_E2E_BUNDLE (passed as --bundle) when set.
// - "mock" (default until the server exists, T11): the Vite dev server in mock mode, MSW answering /api from the
//   mini fixture in the browser, on a per-checkout port (see MOCK_PORT).
//
// Every run sets the browser's colour scheme per project (dark, light). The config resolves its paths from its own
// directory, so `npx --prefix frontend playwright test …` works from the repository root.
import { defineConfig, devices } from "@playwright/test";
import { chromium } from "@playwright/test";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const target = process.env.WFD_E2E_TARGET === "server" ? "server" : "mock";
/**
 * The mock server's port: WFD_MOCK_PORT, else 5200-5999 derived from this checkout's path, so parallel worktrees
 * use different ports. The server is never reused unless WFD_E2E_REUSE=1: a run that finds the port taken fails
 * (`--strictPort`) instead of silently testing another checkout's code.
 */
function defaultMockPort(dir: string): number {
  let h = 0;
  for (const ch of dir) h = (Math.imul(h, 31) + ch.charCodeAt(0)) >>> 0;
  return 5200 + (h % 800);
}
const MOCK_PORT = Number(process.env.WFD_MOCK_PORT ?? defaultMockPort(here));

/**
 * The Chromium build @playwright/test expects, or the preinstalled one at /opt/pw-browsers/chromium (or
 * WFD_CHROMIUM) when the expected revision is not installed (no `playwright install` in CI containers).
 */
function chromiumExecutable(): string | undefined {
  if (process.env.WFD_CHROMIUM) return process.env.WFD_CHROMIUM;
  try {
    if (existsSync(chromium.executablePath())) return undefined;
  } catch {
    // fall through
  }
  return existsSync("/opt/pw-browsers/chromium") ? "/opt/pw-browsers/chromium" : undefined;
}

const serverBin = process.env.WFD_BIN ?? resolve(here, "../target/release/warframe-dps");
const serverArgs = [
  "--no-browser",
  "--port 0",
  `--data-dir ${resolve(here, "test-results/server-data")}`,
  ...(process.env.WFD_E2E_BUNDLE ? [`--bundle ${process.env.WFD_E2E_BUNDLE}`] : []),
].join(" ");

const webServer =
  target === "server"
    ? {
        command: `${serverBin} ${serverArgs}`,
        cwd: here,
        // server-api.md §1.2 step 5: "warframe-dps <version> — http://127.0.0.1:<port>/ (data <data_version>)".
        wait: { stdout: /http:\/\/127\.0\.0\.1:(?<wfd_port>\d+)\// },
        env: { WFD_IMAGES_FETCH: "false", WFD_OPEN_BROWSER: "false" },
        reuseExistingServer: false,
        timeout: 60_000,
      }
    : {
        command: `npm run dev:mock -- --host 127.0.0.1 --port ${MOCK_PORT} --strictPort`,
        cwd: here,
        url: `http://127.0.0.1:${MOCK_PORT}/`,
        reuseExistingServer: process.env.WFD_E2E_REUSE === "1",
        timeout: 120_000,
      };

const executablePath = chromiumExecutable();

export default defineConfig({
  testDir: resolve(here, "e2e"),
  outputDir: resolve(here, "test-results"),
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["list"]],
  timeout: 60_000,
  use: {
    // The server target's port is only known at startup; tests read it through e2e/fixtures.ts.
    baseURL: `http://127.0.0.1:${MOCK_PORT}`,
    trace: "off",
    viewport: { width: 1440, height: 900 },
    launchOptions: executablePath ? { executablePath } : {},
  },
  projects: [
    { name: "dark", use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 900 }, colorScheme: "dark" } },
    { name: "light", use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 900 }, colorScheme: "light" } },
  ],
  webServer,
  metadata: { target },
});
