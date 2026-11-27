// Typed fetch wrapper for every route of server-api.md §3. Errors become `ApiError` carrying a `Message`
// (server-api.md §2: non-2xx bodies are `{ error: Message, issues?: Message[] }`), so the UI renders them through
// i18n; network failures and non-JSON errors get client-side message ids.
import type {
  ActivateRequest,
  CalcInput,
  CalcReport,
  CalcRequest,
  Candidate,
  CandidatesRequest,
  Catalog,
  DataCheck,
  DataStatus,
  DecodeResponse,
  EncodeResponse,
  EnemyDetail,
  ErrorResponse,
  Explanation,
  ExplainRequest,
  ExportInspirationResponse,
  ItemDetail,
  JobAccepted,
  JobStatus,
  Message,
  Meta,
  OptRequest,
  OptResult,
  RefreshRequest,
  ScenarioSchema,
  Timeline,
  TimelineRequest,
  WeaponDetail,
} from "./generated";

/** Base path of the API (same origin; the dev server proxies it). */
export const API_BASE = "/api";

/** A failed API call. `message` is the server's (or a client-side) `Message`. */
export class ApiError extends Error {
  readonly status: number;
  readonly msg: Message;
  readonly issues: Message[];

  constructor(status: number, msg: Message, issues: Message[] = []) {
    super(`${status} ${msg.key}`);
    this.name = "ApiError";
    this.status = status;
    this.msg = msg;
    this.issues = issues;
  }
}

/** Client-side message for an unreachable server. */
export const NETWORK_ERROR: Message = { key: "common.error.network", args: {} };

function isErrorResponse(x: unknown): x is ErrorResponse {
  return (
    typeof x === "object" &&
    x !== null &&
    "error" in x &&
    typeof (x).error === "object" &&
    typeof (x as { error: { key?: unknown } }).error.key === "string"
  );
}

export function isAbortError(e: unknown): boolean {
  return e instanceof DOMException && e.name === "AbortError";
}

export interface RequestOptions {
  signal?: AbortSignal;
  /** Extra headers (`If-None-Match`, `Last-Event-ID`). */
  headers?: Record<string, string>;
}

/** Performs one request; resolves with the parsed JSON body, or `undefined` for 202/204 without a body. */
export async function request<T>(method: string, path: string, body?: unknown, opts: RequestOptions = {}): Promise<T> {
  let res: Response;
  try {
    res = await fetch(`${API_BASE}${path}`, {
      method,
      headers: {
        Accept: "application/json",
        ...(body !== undefined ? { "Content-Type": "application/json" } : {}),
        ...opts.headers,
      },
      body: body !== undefined ? JSON.stringify(body) : undefined,
      signal: opts.signal,
    });
  } catch (e) {
    if (isAbortError(e)) throw e;
    throw new ApiError(0, NETWORK_ERROR);
  }
  const text = await res.text();
  let json: unknown = undefined;
  if (text.length > 0) {
    try {
      json = JSON.parse(text);
    } catch {
      json = undefined;
    }
  }
  if (!res.ok) {
    if (isErrorResponse(json)) throw new ApiError(res.status, json.error, json.issues ?? []);
    throw new ApiError(res.status, { key: "common.error.status", args: { status: res.status } });
  }
  return json as T;
}

const enc = encodeURIComponent;

/** One function per route of server-api.md §3. */
export const api = {
  meta: (o?: RequestOptions) => request<Meta>("GET", "/meta", undefined, o),
  catalog: (o?: RequestOptions) => request<Catalog>("GET", "/catalog", undefined, o),
  weapon: (key: number, o?: RequestOptions) => request<WeaponDetail>("GET", `/weapons/${key}`, undefined, o),
  item: (key: number, o?: RequestOptions) => request<ItemDetail>("GET", `/items/${key}`, undefined, o),
  enemy: (key: number, o?: RequestOptions) => request<EnemyDetail>("GET", `/enemies/${key}`, undefined, o),
  schema: (o?: RequestOptions) => request<ScenarioSchema>("GET", "/schema", undefined, o),
  candidates: (req: CandidatesRequest, o?: RequestOptions) => request<Candidate[]>("POST", "/candidates", req, o),
  calc: (req: CalcRequest, o?: RequestOptions) => request<CalcReport>("POST", "/calc", req, o),
  timeline: (req: TimelineRequest, o?: RequestOptions) => request<Timeline>("POST", "/timeline", req, o),
  explain: (req: ExplainRequest, o?: RequestOptions) => request<Explanation>("POST", "/explain", req, o),
  encode: (input: CalcInput, o?: RequestOptions) => request<EncodeResponse>("POST", "/build/encode", input, o),
  decode: (text: string, o?: RequestOptions) => request<DecodeResponse>("POST", "/build/decode", { text }, o),
  exportInspiration: (input: CalcInput, o?: RequestOptions) =>
    request<ExportInspirationResponse>("POST", "/build/export-inspiration", input, o),
  optimize: (req: OptRequest, o?: RequestOptions) => request<JobAccepted>("POST", "/optimize", req, o),
  optimizeStatus: (job: string, o?: RequestOptions) =>
    request<JobStatus<OptResult>>("GET", `/optimize/${enc(job)}`, undefined, o),
  optimizeCancel: (job: string, o?: RequestOptions) => request<void>("DELETE", `/optimize/${enc(job)}`, undefined, o),
  optimizeEventsUrl: (job: string) => `${API_BASE}/optimize/${enc(job)}/events`,
  dataStatus: (o?: RequestOptions) => request<DataStatus>("GET", "/data/status", undefined, o),
  dataCheck: (o?: RequestOptions) => request<DataCheck>("POST", "/data/check", undefined, o),
  dataRefresh: (req: RefreshRequest, o?: RequestOptions) => request<JobAccepted>("POST", "/data/refresh", req, o),
  dataRefreshCancel: (job: string, o?: RequestOptions) =>
    request<void>("DELETE", `/data/refresh/${enc(job)}`, undefined, o),
  dataRefreshEventsUrl: (job: string) => `${API_BASE}/data/refresh/${enc(job)}/events`,
  dataActivate: (req: ActivateRequest, o?: RequestOptions) => request<DataStatus>("POST", "/data/activate", req, o),
  imgUrl: (key: number) => `${API_BASE}/img/${key}`,
};
