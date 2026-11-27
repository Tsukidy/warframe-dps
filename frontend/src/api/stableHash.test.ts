import { describe, expect, it } from "vitest";
import { stableHash, stableStringify } from "./stableHash";

describe("stableHash", () => {
  it("ignores key order and undefined members", () => {
    expect(stableStringify({ b: 1, a: { d: [1, 2], c: undefined } })).toBe('{"a":{"d":[1,2]},"b":1}');
    expect(stableHash({ a: 1, b: [1, { y: 2, x: 1 }] })).toBe(stableHash({ b: [1, { x: 1, y: 2 }], a: 1 }));
  });

  it("separates different documents", () => {
    expect(stableHash({ a: 1 })).not.toBe(stableHash({ a: 2 }));
    expect(stableHash([1, 2])).not.toBe(stableHash([2, 1]));
    expect(stableHash({ a: 1 })).toMatch(/^[0-9a-f]{16}$/);
  });
});
