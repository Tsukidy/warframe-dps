// The app shell (work-breakdown.md T12 acceptance): renders with the mini-fixture data in the dark and the light
// theme (projects), layout per breakpoint (frontend.md §3), routes, theme switch, axe clean.
import { test, expect, expectAxeClean } from "./fixtures";

test.describe("app shell", () => {
  test("renders the calculator shell with data and a share code in the URL", async ({ page }, info) => {
    await page.goto("/");
    await expect(page.getByRole("banner")).toBeVisible();
    await expect(page.getByTestId("data-version")).toContainText(/Data \S+/);
    for (const pane of ["Loadout", "Results", "Target"]) {
      await expect(page.getByRole("region", { name: pane })).toBeVisible();
    }
    await expect(page.locator("[data-layout=wide]")).toBeVisible();
    // The document booted and the first report's share code reached the URL (urlSync, frontend.md §5).
    await expect(page).toHaveURL(/[?&]b=wfd1-/);
    // A fresh document has nothing to undo.
    await expect(page.getByRole("button", { name: "Undo (Ctrl+Z)" })).toBeDisabled();

    // The theme follows the project's colour scheme: the page background is the surface-0 token.
    const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(bg).toBe(info.project.name === "dark" ? "rgb(15, 18, 22)" : "rgb(246, 247, 249)");

    await expectAxeClean(page, "calculator");
  });

  test("switches themes explicitly and keeps contrast", async ({ page }) => {
    await page.goto("/");
    const toggle = page.getByTestId("theme-toggle");
    await toggle.click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
    await expectAxeClean(page, "dark theme");
    await toggle.click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    const bg = await page.evaluate(() => getComputedStyle(document.body).backgroundColor);
    expect(bg).toBe("rgb(246, 247, 249)");
    await expectAxeClean(page, "light theme");
    await toggle.click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "system");
  });

  test("navigates to the optimizer, data and about pages", async ({ page }) => {
    await page.goto("/");
    const nav = page.getByRole("navigation", { name: "Main" });
    await nav.getByRole("link", { name: "Optimizer" }).click();
    await expect(page).toHaveURL(/\/optimizer$/);
    await expect(page.getByRole("heading", { name: "Optimizer", level: 2 })).toBeVisible();
    await expectAxeClean(page, "optimizer");
    await nav.getByRole("link", { name: "Data" }).click();
    await expect(page.getByRole("heading", { name: "Game data", level: 2 })).toBeVisible();
    await expectAxeClean(page, "data");
    await nav.getByRole("link", { name: "About" }).click();
    await expect(page.getByRole("heading", { name: "About", level: 2 })).toBeVisible();
    await expectAxeClean(page, "about");
    await nav.getByRole("link", { name: "Calculator" }).click();
    await expect(page.getByRole("region", { name: "Results" })).toBeVisible();
    await expectAxeClean(page, "calculator again");
  });

  test("reports a build code that does not decode and starts a new build", async ({ page }) => {
    await page.goto("/?b=wfd1-garbage");
    const alert = page.getByRole("alert").filter({ hasText: "could not be read" });
    await expect(alert).toBeVisible();
    await expect(alert).toContainText("This is not a build code or a supported link.");
    await expect(page.getByRole("region", { name: "Results" })).toBeVisible();
    await expectAxeClean(page, "boot decode failure");
    await alert.getByRole("button", { name: "Dismiss" }).click();
    await expect(alert).toBeHidden();
  });

  test("uses two panes between 900 and 1280 px and a bottom tab bar on phones", async ({ page }) => {
    await page.setViewportSize({ width: 1000, height: 800 });
    await page.goto("/");
    await expect(page.locator("[data-layout=medium]")).toBeVisible();
    await expect(page.getByRole("region", { name: "Loadout" })).toBeVisible();
    await page.getByRole("tab", { name: "Target" }).click();
    await expect(page.getByRole("tabpanel")).toContainText("Enemy, options and simulation settings");
    await expectAxeClean(page, "medium layout");

    await page.setViewportSize({ width: 375, height: 740 });
    await expect(page.locator("[data-layout=narrow]")).toBeVisible();
    const tabs = page.getByRole("tablist", { name: "Panes" });
    await expect(tabs.getByRole("tab")).toHaveCount(3);
    // Share and Import stay reachable (icon-only) at phone width.
    await expect(page.getByRole("button", { name: "Share" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Import" })).toBeVisible();
    await tabs.getByRole("tab", { name: "Loadout" }).click();
    await expect(page.getByRole("tabpanel")).toContainText("Weapon, mods, arcanes and warframe");
    // No horizontal page scrolling at phone width (frontend.md §11).
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    await expectAxeClean(page, "narrow layout");
  });

  test("renders the pseudo-locale without crashing", async ({ page }) => {
    await page.goto("/?locale=en-XA");
    await expect(page.getByRole("region").first()).toBeVisible();
    await expect(page.getByTestId("data-version")).toContainText("[");
    await expectAxeClean(page, "pseudo-locale");
  });
});
