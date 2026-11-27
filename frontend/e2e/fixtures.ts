// Shared e2e fixtures: the base URL of the chosen target (playwright.config.ts) and an axe check that allows no
// serious or critical violation (frontend.md §11; testing.md §8).
import { test as base, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

export const test = base.extend({
  baseURL: async ({ baseURL }, provide) => {
    const port = process.env.WFD_PORT;
    await provide(port ? `http://127.0.0.1:${port}` : baseURL);
  },
});

export { expect };

/** Runs axe (WCAG 2.2 A/AA rules) and fails on serious or critical violations. */
export async function expectAxeClean(page: Page, context?: string): Promise<void> {
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
    .analyze();
  const bad = result.violations
    .filter((v) => v.impact === "serious" || v.impact === "critical")
    .map((v) => `${v.id} (${v.impact}): ${v.help} — ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
  expect(bad, context).toEqual([]);
}
