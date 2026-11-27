import { describe, expect, it, vi } from "vitest";
import { decodeTagged, openEventStream, openOptimizerEvents, SseParser } from "./sse";
import type { OptEvent } from "./generated";

function streamOf(chunks: string[], fail = false): ReadableStream<Uint8Array> {
  const enc = new TextEncoder();
  let i = 0;
  return new ReadableStream({
    pull(c) {
      const ch = chunks[i++];
      if (ch !== undefined) c.enqueue(enc.encode(ch));
      else if (fail) c.error(new Error("connection reset"));
      else c.close();
    },
  });
}

describe("SseParser", () => {
  it("parses events, ids, multi-line data, comments and retry across chunk boundaries", () => {
    const p = new SseParser();
    const out = [
      ...p.push(": keep-alive\n\nevent: progress\nid: 1"),
      ...p.push("7\ndata: {\"a\":1}\n\r\n"),
      ...p.push("data: line1\r\ndata: line2\r\rretry: 2500\n"),
      ...p.push("event: done\ndata:{}\n\n"),
    ];
    expect(out).toEqual([
      { event: "progress", data: '{"a":1}', id: "17" },
      { event: "message", data: "line1\nline2", id: "17" },
      { event: "done", data: "{}", id: "17" },
    ]);
    expect(p.lastEventId).toBe("17");
    expect(p.retry).toBe(2500);
  });

  it("decodes internally tagged events", () => {
    const e = decodeTagged<OptEvent>({ event: "log", data: '{"message":{"key":"opt.log.incumbent","args":{"score":1}}}' });
    expect(e).toEqual({ event: "log", message: { key: "opt.log.incumbent", args: { score: 1 } } });
  });
});

describe("openEventStream", () => {
  it("resumes with Last-Event-ID after a dropped connection and stops after a terminal event", async () => {
    const seen: string[] = [];
    const headers: (string | null)[] = [];
    let call = 0;
    const fetchImpl = vi.fn((_url: string | URL | Request, init?: RequestInit) => {
      headers.push(new Headers(init?.headers).get("Last-Event-ID"));
      call++;
      const body =
        call === 1
          ? streamOf(['event: phase\nid: 1\ndata: {"phase":"prepare","index":0,"total":1,"message":{"key":"opt.phase.prepare","args":{}}}\n\n'], true)
          : streamOf(['event: done\nid: 2\ndata: {"result":null}\n\n', "event: log\nid: 3\ndata: {}\n\n"]);
      return Promise.resolve(new Response(body, { status: 200, headers: { "Content-Type": "text/event-stream" } }));
    });
    const s = openOptimizerEvents("/api/optimize/j/events", (e, id) => seen.push(`${id}:${e.event}`), {
      fetchImpl: fetchImpl as unknown as typeof fetch,
      retryMs: 1,
    });
    await s.finished;
    expect(seen).toEqual(["1:phase", "2:done"]);
    expect(headers).toEqual([null, "1"]);
    expect(s.lastEventId()).toBe("2");
  });

  it("starts from a stored id and can be closed", async () => {
    const fetchImpl = vi.fn((_url: string | URL | Request, init?: RequestInit) => {
      expect(new Headers(init?.headers).get("Last-Event-ID")).toBe("41");
      return new Promise<Response>((_resolve, reject) => {
        init?.signal?.addEventListener("abort", () => reject(new DOMException("aborted", "AbortError")));
      });
    });
    const states: string[] = [];
    const s = openEventStream({
      url: "/x",
      decode: (m) => m,
      onEvent: () => undefined,
      lastEventId: "41",
      fetchImpl: fetchImpl,
      onState: (st) => states.push(st),
    });
    s.close();
    await s.finished;
    expect(states).toEqual(["closed"]);
  });

  it("stops when the server keeps ending the stream without an event (resume after the job's last event)", async () => {
    const headers: (string | null)[] = [];
    const fetchImpl = vi.fn((_url: string | URL | Request, init?: RequestInit) => {
      headers.push(new Headers(init?.headers).get("Last-Event-ID"));
      return Promise.resolve(new Response(streamOf([": keep-alive\n\n"]), { status: 200 }));
    });
    const onError = vi.fn();
    const states: string[] = [];
    const s = openOptimizerEvents("/api/optimize/j/events", () => undefined, {
      fetchImpl: fetchImpl as unknown as typeof fetch,
      lastEventId: "9",
      retryMs: 1,
      onError,
      onState: (st) => states.push(st),
    });
    await s.finished;
    expect(fetchImpl).toHaveBeenCalledTimes(3);
    expect(headers).toEqual(["9", "9", "9"]);
    expect(onError).toHaveBeenCalledTimes(3);
    expect(states.at(-1)).toBe("closed");
  });

  it("an open connection that delivers nothing does not reset the failure count", async () => {
    let call = 0;
    const fetchImpl = vi.fn(() => {
      call++;
      // Alternate: open-then-reset with no event, then refused.
      return Promise.resolve(call % 2 === 1 ? new Response(streamOf([], true), { status: 200 }) : new Response(null, { status: 503 }));
    });
    const s = openEventStream({ url: "/x", decode: (m) => m, onEvent: () => undefined, fetchImpl, retryMs: 1, maxAttempts: 4 });
    await s.finished;
    expect(fetchImpl).toHaveBeenCalledTimes(4);
  });

  it("gives up after maxAttempts failures", async () => {
    const fetchImpl = vi.fn(() => Promise.resolve(new Response(null, { status: 500 })));
    const onError = vi.fn();
    const s = openEventStream({ url: "/x", decode: (m) => m, onEvent: () => undefined, fetchImpl: fetchImpl, retryMs: 1, maxAttempts: 3, onError });
    await s.finished;
    expect(fetchImpl).toHaveBeenCalledTimes(3);
    expect(onError).toHaveBeenCalledTimes(3);
  });
});
