// URL sync (frontend.md §5 "URL", §9): the page URL carries `?b=<share code>`. Every CalcReport carries the code of
// its inputs, so the URL is updated from reports (replaceState, debounced 300 ms) and Rust stays the only codec
// (AD-18). On load, a `?b=` code is decoded through `/api/build/decode`.

/** The search parameter holding the share code. */
export const BUILD_PARAM = "b";
export const URL_DEBOUNCE_MS = 300;

/** The build code in a search string (`?b=wfd1-…`), if any. */
export function readBuildCode(search: string): string | undefined {
  const v = new URLSearchParams(search).get(BUILD_PARAM);
  return v && v.length > 0 ? v : undefined;
}

/** `href` with its `b` parameter set to `code` (other parameters and the hash kept). */
export function withBuildCode(href: string, code: string): string {
  const url = new URL(href, "http://localhost");
  url.searchParams.set(BUILD_PARAM, code);
  const rel = `${url.pathname}${url.search}${url.hash}`;
  return /^[a-z]+:\/\//i.test(href) ? url.toString() : rel;
}

export interface UrlWriter {
  /** Schedules writing `code`; a later call within the debounce window replaces it. */
  update(code: string): void;
  /** Writes a pending code now. */
  flush(): void;
  cancel(): void;
}

/**
 * A debounced writer: `write` is called with the last code once the calls pause for `ms`, and never twice in a row
 * with the same code.
 */
export function createUrlWriter(write: (code: string) => void, ms = URL_DEBOUNCE_MS): UrlWriter {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: string | undefined;
  let last: string | undefined;
  const flush = () => {
    if (timer !== undefined) clearTimeout(timer);
    timer = undefined;
    if (pending !== undefined && pending !== last) {
      last = pending;
      write(pending);
    }
    pending = undefined;
  };
  return {
    update(code) {
      pending = code;
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(flush, ms);
    },
    flush,
    cancel() {
      if (timer !== undefined) clearTimeout(timer);
      timer = undefined;
      pending = undefined;
    },
  };
}

/** Writes the code into the current URL with replaceState (no history entry). */
export function replaceUrlCode(code: string, w: Pick<Window, "location" | "history"> = window): void {
  const next = withBuildCode(`${w.location.pathname}${w.location.search}${w.location.hash}`, code);
  w.history.replaceState(w.history.state, "", next);
}
