// Routes (frontend.md §3, §4): "/" calculator with the typed `?b=` search param, "/optimizer", "/data", "/about"
// (lazy pages of their features). Unknown non-API paths render the not-found page (the server returns index.html
// for them, server-api.md §10).
import { createRootRoute, createRoute, createRouter } from "@tanstack/react-router";
import { RootLayout, NotFound, RouteError } from "./features/shell/RootLayout";
import { CalculatorPage } from "./features/shell/CalculatorPage";
import { PageFrame } from "./features/shell/PlaceholderPage";
import { AboutPage, DataPage, OptimizerPage } from "./features/registry";
import { BUILD_PARAM } from "./state/urlSync";

/** Search params of the calculator route. */
export interface CalculatorSearch {
  b?: string;
}

export function validateCalculatorSearch(search: Record<string, unknown>): CalculatorSearch {
  const b = search[BUILD_PARAM];
  return typeof b === "string" && b.length > 0 ? { b } : {};
}

const rootRoute = createRootRoute({
  component: RootLayout,
  notFoundComponent: NotFound,
  errorComponent: ({ error }) => <RouteError error={error} />,
});

const calculatorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  validateSearch: validateCalculatorSearch,
  component: CalculatorPage,
});

const optimizerRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/optimizer",
  component: () => <PageFrame titleId="shell.page.optimizer" page={OptimizerPage} />,
});

const dataRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/data",
  component: () => <PageFrame titleId="shell.page.data" page={DataPage} />,
});

const aboutRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/about",
  component: () => <PageFrame titleId="shell.page.about" page={AboutPage} />,
});

export const routeTree = rootRoute.addChildren([calculatorRoute, optimizerRoute, dataRoute, aboutRoute]);

export function makeRouter() {
  return createRouter({ routeTree, defaultPreload: "intent" });
}

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof makeRouter>;
  }
}
