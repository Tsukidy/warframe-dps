import { afterEach, describe, expect, it, vi } from "vitest";
import { createUrlWriter, readBuildCode, replaceUrlCode, withBuildCode } from "./urlSync";

afterEach(() => {
  vi.useRealTimers();
});

describe("URL sync", () => {
  it("reads the b parameter", () => {
    expect(readBuildCode("?b=wfd1-abc")).toBe("wfd1-abc");
    expect(readBuildCode("?x=1&b=wfd1-q_-")).toBe("wfd1-q_-");
    expect(readBuildCode("?b=")).toBeUndefined();
    expect(readBuildCode("")).toBeUndefined();
  });

  it("sets the b parameter and keeps the others", () => {
    expect(withBuildCode("/", "wfd1-a")).toBe("/?b=wfd1-a");
    expect(withBuildCode("/?locale=en-XA&b=old#top", "wfd1-new")).toBe("/?locale=en-XA&b=wfd1-new#top");
    expect(withBuildCode("http://127.0.0.1:7878/?b=x", "y")).toBe("http://127.0.0.1:7878/?b=y");
  });

  it("debounces writes by 300 ms and writes only the last code", () => {
    vi.useFakeTimers();
    const write = vi.fn();
    const w = createUrlWriter(write);
    w.update("a");
    vi.advanceTimersByTime(200);
    w.update("b");
    vi.advanceTimersByTime(299);
    expect(write).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(write).toHaveBeenCalledTimes(1);
    expect(write).toHaveBeenCalledWith("b");
  });

  it("skips a write of the code already written", () => {
    vi.useFakeTimers();
    const write = vi.fn();
    const w = createUrlWriter(write, 10);
    w.update("a");
    vi.advanceTimersByTime(10);
    w.update("a");
    vi.advanceTimersByTime(10);
    w.update("b");
    w.flush();
    expect(write.mock.calls).toEqual([["a"], ["b"]]);
    w.update("c");
    w.cancel();
    vi.advanceTimersByTime(20);
    expect(write).toHaveBeenCalledTimes(2);
  });

  it("replaces the URL without adding history entries", () => {
    window.history.replaceState(null, "", "/?locale=en");
    const before = window.history.length;
    replaceUrlCode("wfd1-xyz");
    expect(window.location.search).toBe("?locale=en&b=wfd1-xyz");
    expect(window.history.length).toBe(before);
  });
});
