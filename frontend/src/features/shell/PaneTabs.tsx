// Pane tabs for narrow screens (frontend.md §3): a tab list over the panes, at the bottom below 900 px.
import type { ReactNode } from "react";
import { Tab, TabList, TabPanel, Tabs, type Key } from "react-aria-components";
import { useT } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import { Icon, type IconName } from "../../design/Icon";
import { cx, INTERACTIVE } from "../../design/primitives/cx";

export type PaneId = "loadout" | "results" | "target";

const LABEL: Record<PaneId, MessageId> = {
  loadout: "shell.pane.loadout",
  results: "shell.pane.results",
  target: "shell.pane.target",
};
const ICON: Record<PaneId, IconName> = { loadout: "loadout", results: "results", target: "target" };

export interface PaneTabsProps {
  panes: readonly PaneId[];
  selected: PaneId;
  onSelect: (p: PaneId) => void;
  render: (p: PaneId) => ReactNode;
  /** Tab bar at the bottom (phone layout). */
  bottom?: boolean;
}

export function PaneTabs({ panes, selected, onSelect, render, bottom = false }: PaneTabsProps) {
  const t = useT();
  const list = (
    <TabList
      aria-label={t("shell.paneTabs.label")}
      className={cx(
        "flex shrink-0 border-border bg-surface-1",
        bottom ? "order-last justify-around border-t" : "gap-1 border-b px-2",
      )}
    >
      {panes.map((p) => (
        <Tab
          key={p}
          id={p}
          className={cx(
            "flex min-h-11 items-center justify-center gap-1.5 px-3 text-sm text-fg-2 data-[hovered]:text-fg-1 data-[selected]:text-fg-1",
            bottom ? "flex-1 flex-col gap-0.5 text-xs data-[selected]:text-accent" : "border-b-2 border-transparent data-[selected]:border-accent",
            INTERACTIVE,
          )}
        >
          <Icon name={ICON[p]} size={bottom ? 18 : 14} />
          {t(LABEL[p])}
        </Tab>
      ))}
    </TabList>
  );
  return (
    <Tabs
      selectedKey={selected}
      onSelectionChange={(k: Key) => onSelect(String(k) as PaneId)}
      className="flex min-h-0 min-w-0 flex-1 flex-col"
    >
      {list}
      {panes.map((p) => (
        <TabPanel key={p} id={p} className="flex min-h-0 flex-1 flex-col outline-none">
          {render(p)}
        </TabPanel>
      ))}
    </Tabs>
  );
}
