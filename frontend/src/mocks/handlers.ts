// MSW handlers for every route of server-api.md §3, answering from the mini fixture (fixtures.ts). Used by
// `vite --mode mock` (browser worker), component tests (msw/node) and the e2e shell test until the server exists.
// Feature tasks add handlers in `src/mocks/extra/<task>.ts` (exporting `handlers`); they take precedence.
import { http, HttpResponse, type RequestHandler } from "msw";
import type {
  CalcInput,
  CalcRequest,
  Candidate,
  CandidatesRequest,
  DataEvent,
  DecodeResponse,
  ErrorResponse,
  ExplainRequest,
  JobStatus,
  OptEvent,
  OptResult,
  TimelineRequest,
} from "../api/generated";
import {
  calcReport,
  catalog,
  decodeMock,
  digest,
  encodeMock,
  enemyDetail,
  itemDetail,
  meta,
  schema,
  weaponDetail,
} from "./fixtures";

const A = "/api";

function notFound(what: string, id: string) {
  const body: ErrorResponse = { error: { key: "api.not_found", args: { what, id } } };
  return HttpResponse.json(body, { status: 404 });
}

function sse(events: { event: string; data: unknown }[], startId = 1): HttpResponse<string> {
  const text = events
    .map((e, i) => {
      const { event: _tag, ...rest } = e.data as Record<string, unknown>;
      return `event: ${e.event}\nid: ${startId + i}\ndata: ${JSON.stringify(rest)}\n\n`;
    })
    .join("");
  return new HttpResponse(text, { headers: { "Content-Type": "text/event-stream", "Cache-Control": "no-cache" } });
}

const jobs = new Map<string, JobStatus<OptResult>>();
let jobSeq = 0;

function optEvents(job: string): OptEvent[] {
  return [
    { event: "started", job, plan: { positions: [{ group: "main", pool: digest.items.length }], composition_sets: 0, excluded: [] } },
    { event: "phase", phase: "prepare", index: 0, total: 1, message: { key: "opt.phase.prepare", args: {} } },
    { event: "progress", phase: "prepare", fraction: 1, evaluated: 0, bounded: 0, pruned: 0, rate_per_s: 0, elapsed_ms: 1, eta_s: 0, gap: null },
    { event: "log", message: { key: "opt.failed.internal", args: { error_id: "mock" } } },
    { event: "failed", error: { key: "opt.failed.internal", args: { error_id: "mock" } } },
  ];
}

const dataEvents: DataEvent[] = [
  { event: "failed", error: { key: "data.refresh_unavailable", args: {} } },
];

const baseHandlers: RequestHandler[] = [
  http.get(`${A}/meta`, () => HttpResponse.json(meta())),
  http.get(`${A}/catalog`, () => HttpResponse.json(catalog(), { headers: { ETag: `"${digest.meta.data_version}"` } })),
  http.get(`${A}/schema`, () => HttpResponse.json(schema())),
  http.get(`${A}/weapons/:key`, ({ params }) => {
    const d = weaponDetail(Number(params.key));
    return d ? HttpResponse.json(d) : notFound("weapon", String(params.key));
  }),
  http.get(`${A}/items/:key`, ({ params }) => {
    const d = itemDetail(Number(params.key));
    return d ? HttpResponse.json(d) : notFound("item", String(params.key));
  }),
  http.get(`${A}/enemies/:key`, ({ params }) => {
    const d = enemyDetail(Number(params.key));
    return d ? HttpResponse.json(d) : notFound("enemy", String(params.key));
  }),
  http.post(`${A}/candidates`, async ({ request }) => {
    const req = (await request.json()) as CandidatesRequest;
    const q = req.query?.toLowerCase();
    const used = new Set(req.input.build.slots.map((s) => ("item" in s.content ? s.content.item.item : -1)));
    const out: Candidate[] = digest.items
      .filter((i) => i.kind === "weapon_mod" && (!q || i.name.toLowerCase().includes(q)) && (!req.stat || i.stats.includes(req.stat)))
      .map((i) =>
        used.has(i.key)
          ? { key: i.key, legal: false as const, reason: { key: "dup.mod", args: { item: i.key } } }
          : { key: i.key, legal: true as const },
      );
    return HttpResponse.json(out);
  }),
  http.post(`${A}/calc`, async ({ request }) => {
    const req = (await request.json()) as CalcRequest;
    return HttpResponse.json(calcReport(req, encodeMock(req.input)));
  }),
  http.post(`${A}/timeline`, async ({ request }) => {
    const req = (await request.json()) as TimelineRequest;
    return HttpResponse.json({
      duration: req.input.scenario.eval.window_s,
      hits: [],
      reloads: [],
      heat_episodes: [],
      truncated: false,
    });
  }),
  http.post(`${A}/explain`, async ({ request }) => {
    const req = (await request.json()) as ExplainRequest;
    return HttpResponse.json(
      { error: { key: "engine.probe_unknown", args: { probe: req.probe } } } satisfies ErrorResponse,
      { status: 404 },
    );
  }),
  http.post(`${A}/build/encode`, async ({ request }) => {
    const input = (await request.json()) as CalcInput;
    const code = encodeMock(input);
    return HttpResponse.json({ code, path: `/?b=${code}` });
  }),
  http.post(`${A}/build/decode`, async ({ request }) => {
    const { text } = (await request.json()) as { text: string };
    const input = decodeMock(text);
    if (!input) {
      return HttpResponse.json({ error: { key: "import.unrecognized", args: {} } } satisfies ErrorResponse, { status: 422 });
    }
    const res: DecodeResponse = {
      input,
      source: "wfd1",
      report: { unmapped: [], converted: [], defaults_applied: [], retired_items: [], warnings: [] },
    };
    return HttpResponse.json(res);
  }),
  http.post(`${A}/build/export-inspiration`, () =>
    HttpResponse.json({ error: { key: "import.export_unsupported", args: { feature: "mock" } } } satisfies ErrorResponse, {
      status: 422,
    }),
  ),
  http.post(`${A}/optimize`, () => {
    const job = `opt-mock-${++jobSeq}`;
    jobs.set(job, { job, state: "failed", started_at: new Date(0).toISOString(), error: { key: "opt.failed.internal", args: { error_id: "mock" } } });
    return HttpResponse.json({ job }, { status: 202 });
  }),
  http.get(`${A}/optimize/:job/events`, ({ params, request }) => {
    const job = String(params.job);
    const after = Number(request.headers.get("Last-Event-ID") ?? 0);
    const all = optEvents(job).map((e) => ({ event: e.event, data: e }));
    return sse(all.slice(after), after + 1);
  }),
  http.get(`${A}/optimize/:job`, ({ params }) => {
    const j = jobs.get(String(params.job));
    return j ? HttpResponse.json(j) : notFound("job", String(params.job));
  }),
  http.delete(`${A}/optimize/:job`, () => new HttpResponse(null, { status: 202 })),
  http.get(`${A}/data/status`, () =>
    HttpResponse.json({
      active: {
        id: "embedded",
        source: "embedded",
        data_version: digest.meta.data_version,
        game_baseline: digest.meta.game_baseline,
        created_at: digest.meta.created_at,
      },
      bundles: [],
    }),
  ),
  http.post(`${A}/data/check`, () =>
    HttpResponse.json({
      changed: false,
      pe_build_label: "mock",
      wiki_changed_modules: [],
      wfcd_latest_tag: "mock",
      checked_at: new Date(0).toISOString(),
    }),
  ),
  http.post(`${A}/data/refresh`, () =>
    HttpResponse.json({ error: { key: "data.refresh_unavailable", args: {} } } satisfies ErrorResponse, { status: 409 }),
  ),
  http.get(`${A}/data/refresh/:job/events`, () => sse(dataEvents.map((e) => ({ event: e.event, data: e })))),
  http.delete(`${A}/data/refresh/:job`, () => new HttpResponse(null, { status: 202 })),
  http.post(`${A}/data/activate`, () =>
    HttpResponse.json({ error: { key: "api.not_found", args: { what: "bundle", id: "mock" } } } satisfies ErrorResponse, {
      status: 404,
    }),
  ),
  // No images in mock mode (as with `images.fetch = false`): the UI shows glyph tiles.
  http.get(`${A}/img/:key`, () => new HttpResponse(null, { status: 204 })),
];

const extra = import.meta.glob<{ handlers?: RequestHandler[] }>("./extra/*.ts", { eager: true });

/** Every mock handler: feature extras first (they win), then the base set. */
export const handlers: RequestHandler[] = [
  ...Object.keys(extra)
    .sort()
    .flatMap((k) => extra[k]?.handlers ?? []),
  ...baseHandlers,
];
