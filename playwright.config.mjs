// Root shim (owner T12, CONTRACTS.md): `npx --prefix frontend playwright test` runs in the repository root
// (work-breakdown.md T12 acceptance, `cargo xtask ci`), where Playwright looks for its configuration. The real
// configuration, which resolves every path from its own directory, is frontend/playwright.config.ts.
export { default } from "./frontend/playwright.config.ts";
