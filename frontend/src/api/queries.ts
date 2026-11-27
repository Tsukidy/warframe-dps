// TanStack Query hooks for every route (frontend.md §5 "Derived server state").
//
// - Catalogue, schema and details are fetched once per `data_version` (`staleTime: Infinity`; the version is in the
//   key, so an activated bundle refetches).
// - `useCalcEv`: debounced 120 ms, keyed by (data_version, stableHash(doc)), no MC. `useCalcMc`: debounced 400 ms,
//   `want.mc` set, a fresh seed per request when `eval.seed = "auto"`. Superseded requests are aborted through the
//   query's AbortSignal (the server cancels their work, server-api.md §11); the previous result stays visible while
//   the next one loads (`placeholderData`).
import { useEffect, useMemo, useState } from "react";
import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
  type QueryClient,
} from "@tanstack/react-query";
import { api } from "./client";
import { buildEvRequest, buildMcRequest, type McOptions } from "./seed";
import { stableHash } from "./stableHash";
import type {
  ActivateRequest,
  CalcInput,
  CandidatesRequest,
  ExplainRequest,
  OptRequest,
  RefreshRequest,
  TimelineRequest,
} from "./generated";

export const EV_DEBOUNCE_MS = 120;
export const MC_DEBOUNCE_MS = 400;

/** The value after it has been stable for `ms`. */
export function useDebounced<T>(value: T, ms: number): T {
  const [v, setV] = useState(value);
  useEffect(() => {
    if (ms <= 0) return;
    const t = setTimeout(() => setV(value), ms);
    return () => clearTimeout(t);
  }, [value, ms]);
  return ms <= 0 ? value : v;
}

/** Query keys, all prefixed by the data version where the answer depends on the bundle. */
export const queryKeys = {
  meta: ["meta"] as const,
  catalog: (dv: string) => ["catalog", dv] as const,
  schema: (dv: string) => ["schema", dv] as const,
  weapon: (dv: string, key: number) => ["weapon", dv, key] as const,
  item: (dv: string, key: number) => ["item", dv, key] as const,
  enemy: (dv: string, key: number) => ["enemy", dv, key] as const,
  candidates: (dv: string, h: string) => ["candidates", dv, h] as const,
  calcEv: (dv: string, h: string) => ["calc-ev", dv, h] as const,
  calcMc: (dv: string, h: string, nonce: number) => ["calc-mc", dv, h, nonce] as const,
  timeline: (dv: string, h: string) => ["timeline", dv, h] as const,
  explain: (dv: string, h: string) => ["explain", dv, h] as const,
  job: (job: string) => ["optimize-job", job] as const,
  dataStatus: ["data-status"] as const,
};

/** `GET /api/meta`. Refetched on focus: an activated bundle changes `data_version`. */
export function useMeta() {
  return useQuery({ queryKey: queryKeys.meta, queryFn: ({ signal }) => api.meta({ signal }), staleTime: 30_000 });
}

/** The active data version (undefined until `/api/meta` answered). */
export function useDataVersion(): string | undefined {
  return useMeta().data?.data_version;
}

export function useCatalog() {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.catalog(dv ?? ""),
    queryFn: ({ signal }) => api.catalog({ signal }),
    enabled: dv !== undefined,
    staleTime: Infinity,
  });
}

export function useSchema() {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.schema(dv ?? ""),
    queryFn: ({ signal }) => api.schema({ signal }),
    enabled: dv !== undefined,
    staleTime: Infinity,
  });
}

export function useWeaponDetail(key: number | undefined) {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.weapon(dv ?? "", key ?? 0),
    queryFn: ({ signal }) => api.weapon(key as number, { signal }),
    enabled: dv !== undefined && key !== undefined && key !== 0,
    staleTime: Infinity,
  });
}

export function useItemDetail(key: number | undefined) {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.item(dv ?? "", key ?? 0),
    queryFn: ({ signal }) => api.item(key as number, { signal }),
    enabled: dv !== undefined && key !== undefined && key !== 0,
    staleTime: Infinity,
  });
}

export function useEnemyDetail(key: number | undefined) {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.enemy(dv ?? "", key ?? 0),
    queryFn: ({ signal }) => api.enemy(key as number, { signal }),
    enabled: dv !== undefined && key !== undefined && key !== 0,
    staleTime: Infinity,
  });
}

/**
 * `POST /api/candidates` for one slot of the document (frontend.md §5 `useCandidates(doc, slot, query)`). `stat`
 * is the stat filter of `CandidatesRequest` (server-api.md §5.7). Disabled while `doc` or `slot` is undefined.
 */
export function useCandidates(
  doc: CalcInput | undefined,
  slot: CandidatesRequest["slot"] | undefined,
  query?: string,
  opts: { stat?: string; debounceMs?: number } = {},
) {
  const { stat } = opts;
  const req = useMemo<CandidatesRequest | undefined>(
    () =>
      doc && slot
        ? { input: doc, slot, ...(query ? { query } : {}), ...(stat !== undefined ? { stat } : {}) }
        : undefined,
    [doc, slot, query, stat],
  );
  return useCandidatesRequest(req, opts);
}

/** `POST /api/candidates` with a prebuilt request (the form `useCandidates` builds). */
export function useCandidatesRequest(req: CandidatesRequest | undefined, opts: { debounceMs?: number } = {}) {
  const dv = useDataVersion();
  const debounced = useDebounced(req, opts.debounceMs ?? 150);
  return useQuery({
    queryKey: queryKeys.candidates(dv ?? "", stableHash(debounced ?? null)),
    queryFn: ({ signal }) => api.candidates(debounced as CandidatesRequest, { signal }),
    enabled: dv !== undefined && debounced !== undefined,
    placeholderData: keepPreviousData,
    staleTime: Infinity,
  });
}

export interface CalcHookOptions extends McOptions {
  enabled?: boolean;
  /** Override the debounce (tests). */
  debounceMs?: number;
}

/** EV calculation of the document (no MC), debounced 120 ms. */
export function useCalcEv(doc: CalcInput | undefined, opts: CalcHookOptions = {}) {
  const dv = useDataVersion();
  const debounced = useDebounced(doc, opts.debounceMs ?? EV_DEBOUNCE_MS);
  const enabled = (opts.enabled ?? true) && dv !== undefined && debounced !== undefined;
  const { views, breakdown } = opts;
  return useQuery({
    queryKey: queryKeys.calcEv(dv ?? "", stableHash({ doc: debounced ?? null, views, breakdown })),
    queryFn: ({ signal }) => api.calc(buildEvRequest(debounced as CalcInput, { views, breakdown }), { signal }),
    enabled,
    placeholderData: keepPreviousData,
    staleTime: Infinity,
    retry: false,
  });
}

/**
 * MC calculation of the document, debounced 400 ms. With `eval.seed = "auto"` each request draws a new seed (in the
 * query function, so a refetch re-rolls); `nonce` (the "re-roll" button) forces a new request for the same document.
 */
export function useCalcMc(doc: CalcInput | undefined, opts: CalcHookOptions & { nonce?: number } = {}) {
  const dv = useDataVersion();
  const debounced = useDebounced(doc, opts.debounceMs ?? MC_DEBOUNCE_MS);
  const enabled = (opts.enabled ?? true) && dv !== undefined && debounced !== undefined;
  const { views, breakdown, parity } = opts;
  return useQuery({
    queryKey: queryKeys.calcMc(dv ?? "", stableHash({ doc: debounced ?? null, views, breakdown, parity }), opts.nonce ?? 0),
    queryFn: ({ signal }) => api.calc(buildMcRequest(debounced as CalcInput, { views, breakdown, parity }), { signal }),
    enabled,
    placeholderData: keepPreviousData,
    staleTime: Infinity,
    gcTime: 60_000,
    retry: false,
  });
}

export function useTimeline(req: TimelineRequest | undefined) {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.timeline(dv ?? "", stableHash(req ?? null)),
    queryFn: ({ signal }) => api.timeline(req as TimelineRequest, { signal }),
    enabled: dv !== undefined && req !== undefined,
    staleTime: Infinity,
  });
}

export function useExplain(req: ExplainRequest | undefined) {
  const dv = useDataVersion();
  return useQuery({
    queryKey: queryKeys.explain(dv ?? "", stableHash(req ?? null)),
    queryFn: ({ signal }) => api.explain(req as ExplainRequest, { signal }),
    enabled: dv !== undefined && req !== undefined,
    staleTime: Infinity,
  });
}

// Build codes (server-api.md §7).
export const useEncode = () => useMutation({ mutationFn: (input: CalcInput) => api.encode(input) });
export const useDecode = () => useMutation({ mutationFn: (text: string) => api.decode(text) });
export const useExportInspiration = () =>
  useMutation({ mutationFn: (input: CalcInput) => api.exportInspiration(input) });

// Optimizer (server-api.md §8). Events: `openOptimizerEvents` in sse.ts.
export const useStartOptimize = () => useMutation({ mutationFn: (req: OptRequest) => api.optimize(req) });
export const useCancelOptimize = () => useMutation({ mutationFn: (job: string) => api.optimizeCancel(job) });
export function useOptimizeJob(job: string | undefined, refetchMs?: number) {
  return useQuery({
    queryKey: queryKeys.job(job ?? ""),
    queryFn: ({ signal }) => api.optimizeStatus(job as string, { signal }),
    enabled: job !== undefined,
    refetchInterval: refetchMs,
  });
}

// Data routes (server-api.md §9). Events: `openDataEvents` in sse.ts.
export function useDataStatus() {
  return useQuery({ queryKey: queryKeys.dataStatus, queryFn: ({ signal }) => api.dataStatus({ signal }) });
}
export const useDataCheck = () => useMutation({ mutationFn: () => api.dataCheck() });
export const useDataRefresh = () => useMutation({ mutationFn: (req: RefreshRequest) => api.dataRefresh(req) });
export const useDataRefreshCancel = () => useMutation({ mutationFn: (job: string) => api.dataRefreshCancel(job) });

/** Activating a bundle changes the data version: meta and data status are refetched, so every keyed query follows. */
export function useDataActivate() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (req: ActivateRequest) => api.dataActivate(req),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: queryKeys.meta });
      await qc.invalidateQueries({ queryKey: queryKeys.dataStatus });
    },
  });
}

/** The app's query client defaults. */
export function makeQueryClientOptions(): ConstructorParameters<typeof QueryClient>[0] {
  return {
    defaultOptions: {
      queries: { refetchOnWindowFocus: false, retry: 1 },
      mutations: { retry: false },
    },
  };
}
