import { describe, expect, it } from "vitest";
import {
  extNumber,
  formatCompact,
  formatDamage,
  formatDeltaPercent,
  formatNumber,
  formatPercent,
  formatSeconds,
  roundU5,
} from "./format";

describe("formatters (frontend.md §8)", () => {
  it("rounds damage with floor(x + 0.5) (spec 00 U-5)", () => {
    expect(roundU5(0.5)).toBe(1);
    expect(roundU5(1.4999)).toBe(1);
    expect(roundU5(2.5)).toBe(3);
    expect(roundU5(-0.5)).toBe(0);
    expect(roundU5(-1.5)).toBe(-1);
    expect(roundU5(-1.6)).toBe(-2);
    expect(formatDamage(34380.5, "en")).toBe("34,381");
    expect(formatDamage(34380.49, "en")).toBe("34,380");
    expect(formatDamage(-0.4, "en")).toBe("0");
    expect(formatDamage(1234567.5, "de")).toBe("1.234.568");
  });

  it("shows infinity for the ext_f64 string forms", () => {
    expect(extNumber("Infinity")).toBe(Infinity);
    expect(extNumber("-Infinity")).toBe(-Infinity);
    expect(formatDamage("Infinity", "en")).toBe("∞");
    expect(formatNumber("-Infinity", "en")).toBe("−∞");
    expect(formatDamage(Number.NaN, "en")).toBe("–");
  });

  it("formats fractions as percents with one decimal", () => {
    expect(formatPercent(0.3, "en")).toBe("30.0%");
    expect(formatPercent(1.5, "en")).toBe("150.0%");
    expect(formatPercent(0.12345, "en")).toBe("12.3%");
    expect(formatPercent(0.25, "en", 0)).toBe("25%");
    expect(formatDeltaPercent(0.12, "en")).toBe("+12.0%");
    expect(formatDeltaPercent(-0.035, "en")).toBe("-3.5%");
    expect(formatDeltaPercent(0, "en")).toBe("0.0%");
  });

  it("formats times with three decimals (08 §8.12)", () => {
    expect(formatSeconds(1.2, "en")).toBe("1.200 sec");
    expect(formatSeconds(0.0004, "en")).toBe("0.000 sec");
  });

  it("uses compact notation only on request, after U-5 rounding", () => {
    expect(formatCompact(34381, "en")).toBe("34.4K");
    expect(formatCompact(999.5, "en")).toBe("1K");
    expect(formatNumber(1.23456, "en")).toBe("1.23");
  });
});
