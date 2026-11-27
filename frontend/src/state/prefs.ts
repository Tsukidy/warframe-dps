// Persisted preferences (frontend.md §5 "Persistence"): localStorage with try/catch fallbacks (private mode, quota,
// disabled storage all fall back to memory). Stored: last build (as a share code), prohibited mods (StableKeys plus
// the uniqueName of session-local keys, AD-18), theme, density, locale, collapsed sections, optimizer form.
import { create } from "zustand";
import type { Locale } from "../i18n/I18nProvider";

export type ThemePref = "system" | "dark" | "light";
export type Density = "comfortable" | "compact";

export interface ProhibitedEntry {
  key: number;
  /** uniqueName of a session-local key (≥ 2^31), so the entry survives a bundle change. */
  unique_name?: string;
}

export interface Prefs {
  theme: ThemePref;
  density: Density;
  locale: Locale;
  /** Last build as a share code (`wfd1-…`). */
  lastBuild?: string;
  prohibited: ProhibitedEntry[];
  /** Whether the inspiration's default prohibited list was offered once (PAR-38). */
  prohibitedDefaultsShown: boolean;
  collapsed: Record<string, boolean>;
  /** The optimizer form (owned by the optimizer feature; stored opaquely). */
  optimizerForm?: unknown;
}

export const PREFS_KEY = "wfd.prefs.v1";

export const DEFAULT_PREFS: Prefs = {
  theme: "system",
  density: "comfortable",
  locale: "en",
  prohibited: [],
  prohibitedDefaultsShown: false,
  collapsed: {},
};

/** The subset of the Storage API used here. */
export type KeyValueStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

function browserStorage(): KeyValueStorage | null {
  try {
    return typeof window !== "undefined" ? window.localStorage : null;
  } catch {
    return null;
  }
}

/** Reads the preferences; anything missing or malformed takes its default. */
export function loadPrefs(storage: KeyValueStorage | null = browserStorage()): Prefs {
  try {
    const raw = storage?.getItem(PREFS_KEY);
    if (!raw) return { ...DEFAULT_PREFS };
    const parsed = JSON.parse(raw) as Partial<Prefs>;
    if (typeof parsed !== "object" || parsed === null) return { ...DEFAULT_PREFS };
    const out: Prefs = { ...DEFAULT_PREFS, ...parsed };
    if (!["system", "dark", "light"].includes(out.theme)) out.theme = DEFAULT_PREFS.theme;
    if (!Array.isArray(out.prohibited)) out.prohibited = [];
    if (typeof out.collapsed !== "object" || out.collapsed === null) out.collapsed = {};
    return out;
  } catch {
    return { ...DEFAULT_PREFS };
  }
}

/** Writes the preferences; returns false when storage is unavailable (the values stay in memory). */
export function savePrefs(prefs: Prefs, storage: KeyValueStorage | null = browserStorage()): boolean {
  try {
    if (!storage) return false;
    storage.setItem(PREFS_KEY, JSON.stringify(prefs));
    return true;
  } catch {
    return false;
  }
}

export interface PrefsState {
  prefs: Prefs;
  setPrefs: (patch: Partial<Prefs>) => void;
}

export function createPrefsStore(storage: KeyValueStorage | null = browserStorage()) {
  return create<PrefsState>()((set, get) => ({
    prefs: loadPrefs(storage),
    setPrefs(patch) {
      const prefs = { ...get().prefs, ...patch };
      savePrefs(prefs, storage);
      set({ prefs });
    },
  }));
}

export const usePrefsStore = createPrefsStore();
