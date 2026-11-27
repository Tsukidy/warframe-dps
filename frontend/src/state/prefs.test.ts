import { describe, expect, it } from "vitest";
import { createPrefsStore, DEFAULT_PREFS, loadPrefs, PREFS_KEY, savePrefs, type KeyValueStorage } from "./prefs";

function memory(): KeyValueStorage & { data: Map<string, string> } {
  const data = new Map<string, string>();
  return {
    data,
    getItem: (k) => data.get(k) ?? null,
    setItem: (k, v) => void data.set(k, v),
    removeItem: (k) => void data.delete(k),
  };
}

const throwing: KeyValueStorage = {
  getItem: () => {
    throw new Error("SecurityError");
  },
  setItem: () => {
    throw new Error("QuotaExceededError");
  },
  removeItem: () => undefined,
};

describe("prefs", () => {
  it("round-trips through storage", () => {
    const m = memory();
    expect(savePrefs({ ...DEFAULT_PREFS, theme: "light", lastBuild: "wfd1-x", prohibited: [{ key: 3301 }] }, m)).toBe(true);
    expect(loadPrefs(m)).toMatchObject({ theme: "light", lastBuild: "wfd1-x", prohibited: [{ key: 3301 }] });
  });

  it("falls back to defaults when storage throws or holds garbage", () => {
    expect(loadPrefs(throwing)).toEqual(DEFAULT_PREFS);
    expect(savePrefs(DEFAULT_PREFS, throwing)).toBe(false);
    expect(loadPrefs(null)).toEqual(DEFAULT_PREFS);
    const m = memory();
    m.data.set(PREFS_KEY, "{not json");
    expect(loadPrefs(m)).toEqual(DEFAULT_PREFS);
    m.data.set(PREFS_KEY, JSON.stringify({ theme: "neon", prohibited: "x" }));
    expect(loadPrefs(m)).toMatchObject({ theme: "system", prohibited: [] });
  });

  it("keeps working in memory when storage is unavailable", () => {
    const s = createPrefsStore(throwing);
    s.getState().setPrefs({ theme: "dark" });
    expect(s.getState().prefs.theme).toBe("dark");
  });
});
