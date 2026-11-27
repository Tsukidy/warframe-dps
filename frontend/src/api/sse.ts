// Server-sent events with Last-Event-ID resume (server-api.md §8.2, §9).
//
// The optimizer and data-refresh streams send one `OptEvent` / `DataEvent` per SSE message: `event:` is the variant
// name, `data:` the variant's other fields as JSON, `id:` a per-job sequence number. The server keeps the last 1,000
// events of a job; a client reconnecting with `Last-Event-ID` receives everything after that id. The browser's
// EventSource only sends the header on its own automatic reconnects, so this client reads the stream with `fetch`
// and sets the header itself on every reconnect (and on a fresh open with a stored id).
import type { DataEvent, OptEvent } from "./generated";

/** One dispatched SSE message. */
export interface SseMessage {
  event: string;
  data: string;
  id?: string;
}

/**
 * Incremental parser of the `text/event-stream` format (WHATWG HTML §9.2): lines end in LF, CRLF or CR; blank line
 * dispatches; `:` starts a comment (keep-alives); `retry:` sets the reconnection delay.
 */
export class SseParser {
  private buffer = "";
  private data: string[] = [];
  private event = "";
  private id: string | undefined;
  /** The last `id:` seen (persists across messages, per the format). */
  lastEventId: string | undefined;
  /** The last `retry:` value in milliseconds. */
  retry: number | undefined;

  /** Feeds a decoded chunk; returns the messages it completed. */
  push(chunk: string): SseMessage[] {
    this.buffer += chunk;
    const out: SseMessage[] = [];
    for (;;) {
      const m = /\r\n|\r|\n/.exec(this.buffer);
      if (!m) break;
      // A lone CR at the very end may be the first half of CRLF: wait for more input.
      if (m[0] === "\r" && m.index === this.buffer.length - 1) break;
      const line = this.buffer.slice(0, m.index);
      this.buffer = this.buffer.slice(m.index + m[0].length);
      const msg = this.line(line);
      if (msg) out.push(msg);
    }
    return out;
  }

  private line(line: string): SseMessage | undefined {
    if (line === "") {
      if (this.data.length === 0) {
        this.event = "";
        return undefined;
      }
      const msg: SseMessage = { event: this.event || "message", data: this.data.join("\n") };
      if (this.id !== undefined) msg.id = this.id;
      this.data = [];
      this.event = "";
      return msg;
    }
    if (line.startsWith(":")) return undefined;
    const colon = line.indexOf(":");
    const field = colon < 0 ? line : line.slice(0, colon);
    let value = colon < 0 ? "" : line.slice(colon + 1);
    if (value.startsWith(" ")) value = value.slice(1);
    switch (field) {
      case "data":
        this.data.push(value);
        break;
      case "event":
        this.event = value;
        break;
      case "id":
        if (!value.includes("\0")) {
          this.id = value;
          this.lastEventId = value;
        }
        break;
      case "retry":
        if (/^\d+$/.test(value)) this.retry = Number(value);
        break;
    }
    return undefined;
  }
}

/** Decodes an SSE message of an internally tagged event union (`OptEvent`, `DataEvent`). */
export function decodeTagged<E extends { event: string }>(msg: SseMessage): E {
  const body = msg.data.length > 0 ? (JSON.parse(msg.data) as Record<string, unknown>) : {};
  return { ...body, event: msg.event } as E;
}

const OPT_TERMINAL = new Set(["done", "cancelled", "failed"]);

export function isTerminalOptEvent(e: OptEvent): boolean {
  return OPT_TERMINAL.has(e.event);
}

export function isTerminalDataEvent(e: DataEvent): boolean {
  return OPT_TERMINAL.has(e.event);
}

export interface EventStreamOptions<E> {
  url: string;
  decode: (msg: SseMessage) => E;
  onEvent: (event: E, id: string | undefined) => void;
  /** Called after a terminal event; the stream is closed and not reopened. */
  isTerminal?: (event: E) => boolean;
  /** Resume after this id (e.g. stored before a page reload). */
  lastEventId?: string;
  /** Reconnection delay in ms when the server sends no `retry:` (default 1000). */
  retryMs?: number;
  /** Connection state changes, for the UI ("reconnecting…"). */
  onState?: (state: "open" | "reconnecting" | "closed") => void;
  /** Called on a failure that will be retried, with the attempt number. */
  onError?: (error: unknown, attempt: number) => void;
  /** Give up after this many consecutive failed attempts (default 20). */
  maxAttempts?: number;
  fetchImpl?: typeof fetch;
}

export interface EventStream {
  close(): void;
  /** The id of the last event received (for resuming later). */
  lastEventId(): string | undefined;
  /** Resolves when the stream ends (terminal event, close, or attempts exhausted). */
  finished: Promise<void>;
}

/** Opens a resumable event stream. */
export function openEventStream<E>(opts: EventStreamOptions<E>): EventStream {
  const controller = new AbortController();
  const doFetch = opts.fetchImpl ?? fetch;
  let lastId = opts.lastEventId;
  let closed = false;
  const maxAttempts = opts.maxAttempts ?? 20;

  const sleep = (ms: number) =>
    new Promise<void>((resolve) => {
      const t = setTimeout(resolve, ms);
      controller.signal.addEventListener("abort", () => {
        clearTimeout(t);
        resolve();
      });
    });

  const run = async () => {
    let attempt = 0;
    let retry = opts.retryMs ?? 1000;
    while (!closed) {
      let terminal = false;
      try {
        const res = await doFetch(opts.url, {
          headers: { Accept: "text/event-stream", ...(lastId !== undefined ? { "Last-Event-ID": lastId } : {}) },
          signal: controller.signal,
        });
        if (!res.ok || !res.body) throw new Error(`SSE ${res.status}`);
        opts.onState?.("open");
        attempt = 0;
        const parser = new SseParser();
        const reader = res.body.pipeThrough(new TextDecoderStream()).getReader();
        for (;;) {
          const { value, done } = await reader.read();
          if (done) break;
          for (const msg of parser.push(value)) {
            if (msg.id !== undefined) lastId = msg.id;
            const ev = opts.decode(msg);
            opts.onEvent(ev, msg.id);
            if (opts.isTerminal?.(ev)) {
              terminal = true;
              break;
            }
          }
          if (parser.retry !== undefined) retry = parser.retry;
          if (terminal) {
            await reader.cancel().catch(() => undefined);
            break;
          }
        }
      } catch (e) {
        if (closed || controller.signal.aborted) break;
        attempt++;
        opts.onError?.(e, attempt);
        if (attempt >= maxAttempts) break;
      }
      if (terminal || closed) break;
      opts.onState?.("reconnecting");
      await sleep(retry);
    }
    closed = true;
    opts.onState?.("closed");
  };

  const finished = run();
  return {
    close() {
      closed = true;
      controller.abort();
    },
    lastEventId: () => lastId,
    finished,
  };
}

/** The optimizer job's event stream (server-api.md §8.2). */
export function openOptimizerEvents(
  url: string,
  onEvent: (e: OptEvent, id: string | undefined) => void,
  extra: Partial<EventStreamOptions<OptEvent>> = {},
): EventStream {
  return openEventStream<OptEvent>({ url, decode: decodeTagged, onEvent, isTerminal: isTerminalOptEvent, ...extra });
}

/** The data-refresh job's event stream (server-api.md §9). */
export function openDataEvents(
  url: string,
  onEvent: (e: DataEvent, id: string | undefined) => void,
  extra: Partial<EventStreamOptions<DataEvent>> = {},
): EventStream {
  return openEventStream<DataEvent>({ url, decode: decodeTagged, onEvent, isTerminal: isTerminalDataEvent, ...extra });
}
