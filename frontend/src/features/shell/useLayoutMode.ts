// Layout breakpoints (frontend.md §3): ≥ 1280 px three panes, 900-1280 px two panes (Target becomes a tab next to
// Results), < 900 px one pane with a bottom tab bar.
import { useSyncExternalStore } from "react";

export type LayoutMode = "wide" | "medium" | "narrow";

export const WIDE_MIN_PX = 1280;
export const MEDIUM_MIN_PX = 900;

const WIDE = `(min-width: ${WIDE_MIN_PX}px)`;
const MEDIUM = `(min-width: ${MEDIUM_MIN_PX}px)`;

export function layoutModeFor(width: number): LayoutMode {
  return width >= WIDE_MIN_PX ? "wide" : width >= MEDIUM_MIN_PX ? "medium" : "narrow";
}

function subscribe(cb: () => void): () => void {
  if (typeof window === "undefined" || !window.matchMedia) return () => undefined;
  const lists = [window.matchMedia(WIDE), window.matchMedia(MEDIUM)];
  lists.forEach((l) => l.addEventListener("change", cb));
  return () => lists.forEach((l) => l.removeEventListener("change", cb));
}

function snapshot(): LayoutMode {
  if (typeof window === "undefined" || !window.matchMedia) return "wide";
  if (window.matchMedia(WIDE).matches) return "wide";
  if (window.matchMedia(MEDIUM).matches) return "medium";
  return "narrow";
}

export function useLayoutMode(): LayoutMode {
  return useSyncExternalStore(subscribe, snapshot, () => "wide");
}
