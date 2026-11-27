import { describe, expect, it } from "vitest";
import { buildEvRequest, buildMcRequest, drawSeed, MAX_SEED } from "./seed";
import { newDocument } from "../state/defaults";

describe("seed auto (frontend.md §5, T-29)", () => {
  it("draws seeds in 0 … 2^53 − 1", () => {
    expect(drawSeed(() => [0, 0])).toBe(0);
    expect(drawSeed(() => [0xffffffff, 0xffffffff])).toBe(MAX_SEED);
    expect(drawSeed(() => [1 << 11, 5])).toBe(2 ** 32 + 5);
    for (let i = 0; i < 100; i++) {
      const s = drawSeed();
      expect(Number.isSafeInteger(s)).toBe(true);
      expect(s).toBeGreaterThanOrEqual(0);
    }
  });

  it("puts a fresh seed in want.mc when the document says auto, and keeps auto in the document", () => {
    const doc = newDocument(1204, 9001);
    expect(doc.scenario.eval.seed).toBe("auto");
    let n = 0;
    const words = (): [number, number] => [0, ++n];
    const a = buildMcRequest(doc, {}, words);
    const b = buildMcRequest(doc, {}, words);
    expect(a.want.mc?.seed).toBe(1);
    expect(b.want.mc?.seed).toBe(2);
    expect(a.input.scenario.eval.seed).toBe("auto");
    expect(doc.scenario.eval.seed).toBe("auto");
    expect(a.want.mc?.runs).toBe(1000);
    expect(a.want.parity).toBe(true);
  });

  it("re-rolls on every request with the crypto source", () => {
    const doc = newDocument(1204, 9001);
    const seeds = new Set(Array.from({ length: 20 }, () => buildMcRequest(doc).want.mc?.seed));
    expect(seeds.size).toBeGreaterThan(1);
  });

  it("sends no seed in want for a fixed seed (the document carries it)", () => {
    const doc = newDocument(1204, 9001);
    doc.scenario.eval.seed = 12345;
    const req = buildMcRequest(doc);
    expect(req.want.mc).toBeDefined();
    expect(req.want.mc?.seed).toBeUndefined();
    expect(req.input.scenario.eval.seed).toBe(12345);
  });

  it("EV requests carry no MC", () => {
    const req = buildEvRequest(newDocument(1204, 9001), { breakdown: true });
    expect(req.want).toEqual({ breakdown: true });
  });
});
