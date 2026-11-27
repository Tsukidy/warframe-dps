import { describe, expect, it } from "vitest";
import { relativeDelta } from "./baseline";

describe("baseline deltas", () => {
  it("is (current − base) / |base| and null when undefined", () => {
    expect(relativeDelta(112, 100)).toBeCloseTo(0.12);
    expect(relativeDelta(90, 100)).toBeCloseTo(-0.1);
    expect(relativeDelta(-5, -10)).toBeCloseTo(0.5);
    expect(relativeDelta(1, 0)).toBeNull();
    expect(relativeDelta(null, 1)).toBeNull();
    expect(relativeDelta(1, Infinity)).toBeNull();
  });
});
