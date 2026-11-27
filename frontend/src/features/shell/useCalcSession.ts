// The calculator session: the EV and MC queries of the store's document (frontend.md §5) and the effects that hang
// off the EV report: the URL code (urlSync.ts), the last build in prefs, and the live-region summary.
//
// Every caller of `useCalcSession` shares the same TanStack queries (same keys), so the results feature can call it
// too; the side effects run once, in `<CalcSessionEffects>` mounted by the shell.
import { useEffect, useMemo } from "react";
import { useRouter } from "@tanstack/react-router";
import { useIntl } from "react-intl";
import { useCalcEv, useCalcMc } from "../../api/queries";
import { useAppStore } from "../../state/store";
import { isComplete } from "../../state/defaults";
import { createUrlWriter, withBuildCode } from "../../state/urlSync";
import { usePrefsStore } from "../../state/prefs";
import { windowDps, windowDpsDelta } from "../../state/baseline";
import { formatDamage, formatDeltaPercent } from "../../i18n/format";
import { useAnnounce } from "./LiveRegion";

export function useCalcSession() {
  const doc = useAppStore((s) => s.doc);
  const ready = useAppStore((s) => s.ready);
  const autoMc = useAppStore((s) => s.ui.autoMc);
  const nonce = useAppStore((s) => s.ui.mcNonce);
  const enabled = ready && isComplete(doc);
  const ev = useCalcEv(enabled ? doc : undefined, { enabled });
  const mc = useCalcMc(enabled ? doc : undefined, { enabled: enabled && autoMc, nonce, parity: true });
  return { ev, mc, report: ev.data, mcReport: mc.data };
}

export function CalcSessionEffects() {
  const { report } = useCalcSession();
  const router = useRouter();
  const setPrefs = usePrefsStore((s) => s.setPrefs);
  const baseline = useAppStore((s) => s.baseline);
  const view = useAppStore((s) => s.ui.view);
  const attack = useAppStore((s) => s.ui.selectedAttack);
  const announce = useAnnounce();
  const intl = useIntl();

  const writer = useMemo(
    () =>
      createUrlWriter((code) => {
        const loc = router.state.location;
        if (loc.pathname !== "/") return;
        router.history.replace(withBuildCode(loc.href, code), router.history.location.state);
        setPrefs({ lastBuild: code });
      }),
    [router, setPrefs],
  );
  useEffect(() => () => writer.cancel(), [writer]);

  useEffect(() => {
    if (report?.code) writer.update(report.code);
  }, [report?.code, writer]);

  useEffect(() => {
    const dps = windowDps(report, attack, view);
    if (dps === null) return;
    const delta = windowDpsDelta(report, baseline, attack, view);
    const dpsText = formatDamage(dps, intl.locale);
    announce(
      delta === null
        ? intl.formatMessage({ id: "shell.live.result" }, { dps: dpsText })
        : intl.formatMessage(
            { id: "shell.live.resultDelta" },
            { dps: dpsText, delta: formatDeltaPercent(delta, intl.locale) },
          ),
    );
  }, [report, baseline, attack, view, announce, intl]);

  return null;
}
