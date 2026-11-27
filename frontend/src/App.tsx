// The provider stack (frontend.md §4 <App>): I18nProvider · QueryClientProvider · RouterProvider · theme · live
// region.
import { useState } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { I18nProvider, isLocale, type Locale } from "./i18n/I18nProvider";
import { makeQueryClientOptions } from "./api/queries";
import { LiveRegionProvider } from "./features/shell/LiveRegion";
import { usePrefsStore } from "./state/prefs";
import { makeRouter } from "./router";

/** `?locale=en-XA` overrides the stored locale (pseudo-locale e2e run, frontend.md §8). */
function urlLocale(): Locale | undefined {
  const v = new URLSearchParams(window.location.search).get("locale");
  return isLocale(v) ? v : undefined;
}

export function App() {
  const [queryClient] = useState(() => new QueryClient(makeQueryClientOptions()));
  const [router] = useState(makeRouter);
  const stored = usePrefsStore((s) => s.prefs.locale);
  const [override] = useState(urlLocale);
  return (
    <I18nProvider locale={override ?? stored}>
      <QueryClientProvider client={queryClient}>
        <LiveRegionProvider>
          <RouterProvider router={router} />
        </LiveRegionProvider>
      </QueryClientProvider>
    </I18nProvider>
  );
}
