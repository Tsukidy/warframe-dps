// The polite live region (frontend.md §11): announces a throttled summary after recalculation ("Window DPS 34,381,
// plus 12 %"). Features call `useAnnounce()`.
import { createContext, useCallback, useContext, useEffect, useRef, useState, type ReactNode } from "react";

type Announce = (text: string) => void;

const AnnounceContext = createContext<Announce>(() => undefined);

/** Minimum time between two announcements. */
export const ANNOUNCE_THROTTLE_MS = 1500;

export function LiveRegionProvider({ children }: { children: ReactNode }) {
  const [text, setText] = useState("");
  const last = useRef(0);
  const pending = useRef<string | undefined>(undefined);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const announce = useCallback((t: string) => {
    const now = Date.now();
    const wait = last.current + ANNOUNCE_THROTTLE_MS - now;
    if (wait <= 0) {
      last.current = now;
      setText(t);
      return;
    }
    pending.current = t;
    if (timer.current === undefined) {
      timer.current = setTimeout(() => {
        timer.current = undefined;
        last.current = Date.now();
        if (pending.current !== undefined) setText(pending.current);
        pending.current = undefined;
      }, wait);
    }
  }, []);

  useEffect(() => () => clearTimeout(timer.current), []);

  return (
    <AnnounceContext.Provider value={announce}>
      {children}
      <div aria-live="polite" aria-atomic="true" role="status" className="sr-only" data-testid="live-region">
        {text}
      </div>
    </AnnounceContext.Provider>
  );
}

export function useAnnounce(): Announce {
  return useContext(AnnounceContext);
}
