// The calculator's responsive pane layout (frontend.md §3): Loadout (420 px) · Results (fluid) · Target (360 px).
// Each pane scrolls on its own. 900-1280 px: Loadout plus a tabbed Results/Target column. < 900 px: one pane with
// a bottom tab bar.
import type { ReactNode } from "react";
import { useId } from "react";
import { useAppStore } from "../../state/store";
import { useT } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import { cx } from "../../design/primitives";
import { PaneTabs, type PaneId } from "./PaneTabs";
import { useLayoutMode, type LayoutMode } from "./useLayoutMode";

export interface AppLayoutProps {
  loadout: ReactNode;
  results: ReactNode;
  target: ReactNode;
  /** Force a mode (tests); defaults to the viewport's. */
  mode?: LayoutMode;
}

const TITLE: Record<PaneId, MessageId> = {
  loadout: "shell.pane.loadout",
  results: "shell.pane.results",
  target: "shell.pane.target",
};

export function Pane({ id, children, className, hideTitle }: { id: PaneId; children: ReactNode; className?: string; hideTitle?: boolean }) {
  const t = useT();
  const headingId = useId();
  return (
    <section
      aria-labelledby={headingId}
      data-pane={id}
      className={cx("flex min-h-0 min-w-0 flex-col overflow-y-auto bg-surface-0 p-3", className)}
    >
      <h2 id={headingId} className={cx("mt-0 mb-2 text-xs font-semibold tracking-wide text-fg-2 uppercase", hideTitle && "sr-only")}>
        {t(TITLE[id])}
      </h2>
      {children}
    </section>
  );
}

export function AppLayout({ loadout, results, target, mode }: AppLayoutProps) {
  const auto = useLayoutMode();
  const layout = mode ?? auto;
  const pane = useAppStore((s) => s.ui.narrowPane);
  const setUi = useAppStore((s) => s.setUi);
  const content: Record<PaneId, ReactNode> = { loadout, results, target };

  if (layout === "wide") {
    return (
      <div className="grid min-h-0 flex-1 grid-cols-[420px_minmax(0,1fr)_360px] divide-x divide-border" data-layout="wide">
        <Pane id="loadout">{loadout}</Pane>
        <Pane id="results">{results}</Pane>
        <Pane id="target">{target}</Pane>
      </div>
    );
  }
  if (layout === "medium") {
    const right: PaneId = pane === "target" ? "target" : "results";
    return (
      <div className="grid min-h-0 flex-1 grid-cols-[420px_minmax(0,1fr)] divide-x divide-border" data-layout="medium">
        <Pane id="loadout">{loadout}</Pane>
        <PaneTabs
          panes={["results", "target"]}
          selected={right}
          onSelect={(p) => setUi({ narrowPane: p })}
          render={(p) => (
            <Pane id={p} hideTitle>
              {content[p]}
            </Pane>
          )}
        />
      </div>
    );
  }
  return (
    <div className="flex min-h-0 flex-1 flex-col" data-layout="narrow">
      <PaneTabs
        panes={["loadout", "results", "target"]}
        selected={pane}
        onSelect={(p) => setUi({ narrowPane: p })}
        bottom
        render={(p) => (
          <Pane id={p} hideTitle>
            {content[p]}
          </Pane>
        )}
      />
    </div>
  );
}
