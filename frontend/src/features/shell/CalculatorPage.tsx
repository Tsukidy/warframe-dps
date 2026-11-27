// Route "/" (frontend.md §4 <CalculatorPage>): the three panes from the feature registry, placeholders until the
// loadout (T30), results and target (T31) features exist.
import { useT } from "../../i18n/I18nProvider";
import { EmptyState, Skeleton } from "../../design/primitives";
import { LoadoutPane, ResultsPane, TargetPane } from "../registry";
import { AppLayout } from "./AppLayout";
import { useBoot } from "./useBoot";
import { useAppStore } from "../../state/store";

export function CalculatorPage() {
  const t = useT();
  useBoot();
  const ready = useAppStore((s) => s.ready);
  const pending = <Skeleton lines={4} />;
  return (
    <AppLayout
      loadout={!ready ? pending : LoadoutPane ? <LoadoutPane /> : <EmptyState icon="loadout" title={t("shell.placeholder.loadout")} />}
      results={!ready ? pending : ResultsPane ? <ResultsPane /> : <EmptyState icon="results" title={t("shell.placeholder.results")} />}
      target={!ready ? pending : TargetPane ? <TargetPane /> : <EmptyState icon="target" title={t("shell.placeholder.target")} />}
    />
  );
}
