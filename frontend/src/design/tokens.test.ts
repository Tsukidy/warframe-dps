// Token contrast (frontend.md §10, §11; testing.md §8): text tokens ≥ 4.5:1 on every surface in both themes; the
// accent, statistics and status colours ≥ 3:1 (non-text UI, WCAG 1.4.11); text on filled accent ≥ 4.5:1.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

// Vitest runs from the frontend package root (npm scripts set the working directory).
const css = readFileSync(resolve(process.cwd(), "src/design/tokens.css"), "utf8");

type Theme = "light" | "dark";

function tokens(): Record<string, Record<Theme, string>> {
  const out: Record<string, Record<Theme, string>> = {};
  const re = /--([\w-]+):\s*light-dark\(\s*(#[0-9a-fA-F]{6})\s*,\s*(#[0-9a-fA-F]{6})\s*\)/g;
  for (const m of css.matchAll(re)) {
    out[m[1] as string] = { light: m[2] as string, dark: m[3] as string };
  }
  return out;
}

function luminance(hex: string): number {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const [r, g, b] = c.map((x) => (x <= 0.04045 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4)) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrast(a: string, b: string): number {
  const [la, lb] = [luminance(a), luminance(b)];
  return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
}

const T = tokens();
const SURFACES = ["surface-0", "surface-1", "surface-2", "surface-3"];
const THEMES: Theme[] = ["dark", "light"];

describe("design tokens", () => {
  it("parses every colour token as a light/dark pair", () => {
    for (const name of [...SURFACES, "text-1", "text-2", "text-3", "accent", "accent-fill", "on-accent", "stat", "success", "warning", "danger"]) {
      expect(T[name], name).toBeDefined();
    }
    expect(Object.keys(T).filter((k) => k.startsWith("dt-"))).toHaveLength(21);
  });

  it("keeps the frontend.md §10 surface and primary values", () => {
    expect(T["surface-0"]).toEqual({ light: "#f6f7f9", dark: "#0f1216" });
    expect(T["surface-3"]).toEqual({ light: "#e3e8ef", dark: "#252e3c" });
    expect(T["text-1"]).toEqual({ light: "#141a22", dark: "#e8ecf1" });
    expect(T.accent).toEqual({ light: "#a86a00", dark: "#f5b544" });
    expect(T.stat).toEqual({ light: "#0b7a93", dark: "#5cc8e0" });
  });

  for (const theme of THEMES) {
    for (const text of ["text-1", "text-2", "text-3"]) {
      for (const surface of SURFACES) {
        it(`${text} on ${surface} (${theme}) is at least 4.5:1`, () => {
          const fg = T[text]?.[theme] as string;
          const bg = T[surface]?.[theme] as string;
          expect(contrast(fg, bg)).toBeGreaterThanOrEqual(4.5);
        });
      }
    }
    it(`on-accent on accent-fill (${theme}) is at least 4.5:1`, () => {
      expect(contrast(T["on-accent"]?.[theme] as string, T["accent-fill"]?.[theme] as string)).toBeGreaterThanOrEqual(4.5);
    });
    for (const ui of ["accent", "stat", "success", "warning", "danger"]) {
      it(`${ui} (${theme}) is at least 3:1 on every surface`, () => {
        for (const surface of SURFACES) {
          expect(contrast(T[ui]?.[theme] as string, T[surface]?.[theme] as string), surface).toBeGreaterThanOrEqual(3);
        }
      });
    }
    it(`damage-type colours (${theme}) are at least 3:1 on the card surface`, () => {
      for (const [name, v] of Object.entries(T).filter(([k]) => k.startsWith("dt-"))) {
        expect(contrast(v[theme], T["surface-2"]?.[theme] as string), name).toBeGreaterThanOrEqual(3);
      }
    });
  }
});
