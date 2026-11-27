// Boot sequence of the calculator document (frontend.md §5, §9): a `?b=` code in the URL wins, then the last build
// from prefs, then a default document on the first weapon and enemy of the catalogue. Codes decode through
// `/api/build/decode` (Rust is the only codec, AD-18).
import { useEffect, useRef, useState } from "react";
import { api } from "../../api/client";
import { useCatalog, useMeta } from "../../api/queries";
import type { Catalog, ImportReport } from "../../api/generated";
import { useAppStore } from "../../state/store";
import { newDocument } from "../../state/defaults";
import { readBuildCode } from "../../state/urlSync";
import { usePrefsStore } from "../../state/prefs";

/** The default weapon and enemy of a new document: the first primary weapon and the first enemy by name. */
export function defaultPicks(catalog: Catalog): { weapon: number; enemy: number } {
  const byName = <T extends { name: string }>(a: T, b: T) => a.name.localeCompare(b.name);
  const weapons = [...catalog.weapons].sort(byName);
  const weapon = (weapons.find((w) => w.slot === "primary") ?? weapons[0])?.key ?? 0;
  const enemy = [...catalog.enemies].sort(byName)[0]?.key ?? 0;
  return { weapon, enemy };
}

export interface BootState {
  /** The decode report of a URL or stored code (shown by the import feature as a dismissible report). */
  report?: ImportReport;
  failed: boolean;
}

export function useBoot(): BootState {
  const ready = useAppStore((s) => s.ready);
  const replaceDoc = useAppStore((s) => s.replaceDoc);
  const meta = useMeta();
  const catalog = useCatalog();
  const started = useRef(false);
  const [state, setState] = useState<BootState>({ failed: false });

  useEffect(() => {
    if (ready || started.current || !meta.data || !catalog.data) return;
    started.current = true;
    const cat = catalog.data;
    const mcRuns = meta.data.config.mc_runs_default;
    const code = readBuildCode(window.location.search) ?? usePrefsStore.getState().prefs.lastBuild;
    const fallback = () => {
      const { weapon, enemy } = defaultPicks(cat);
      replaceDoc(newDocument(weapon, enemy, mcRuns), { record: false });
    };
    if (!code) {
      fallback();
      return;
    }
    api
      .decode(code)
      .then((res) => {
        replaceDoc(res.input, { record: false });
        setState({ report: res.report, failed: false });
      })
      .catch(() => {
        fallback();
        setState({ failed: true });
      });
  }, [ready, meta.data, catalog.data, replaceDoc]);

  return state;
}
